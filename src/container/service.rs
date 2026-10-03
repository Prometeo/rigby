use crate::container::models::{DockerContainer, DockerContainerDetail};
use bollard::{
    Docker,
    container::LogOutput,
    query_parameters::{
        ListContainersOptionsBuilder, LogsOptions, StartContainerOptions,
        StopContainerOptionsBuilder,
    },
};
use color_eyre::Result;
use futures_util::StreamExt;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::task::AbortHandle;

pub async fn list_containers(client: &Docker) -> Result<Vec<DockerContainer>> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    let containers = client.list_containers(Some(options)).await?;

    let container_list = containers
        .into_iter()
        .map(DockerContainer::try_from)
        .collect::<Result<Vec<DockerContainer>, _>>()?;
    Ok(container_list)
}

pub async fn inspect_container(
    client: &Docker,
    container: &DockerContainer,
) -> Result<DockerContainerDetail> {
    let container_info = client.inspect_container(&container.id, None).await?;
    Ok(DockerContainerDetail::new(&container_info, container))
}

pub fn get_container_logs(
    client: Docker,
    container_id: String,
    tail: usize,
) -> (UnboundedReceiver<String>, AbortHandle) {
    let (tx, rx) = unbounded_channel();

    let task = tokio::spawn(async move {
        let options = LogsOptions {
            stdout: true,
            stderr: true,
            follow: true,
            tail: tail.to_string(),
            timestamps: false,
            ..Default::default()
        };

        let mut stream = client.logs(&container_id, Some(options));

        while let Some(chunk) = stream.next().await {
            let Ok(output) = chunk else { break };
            if let Some(lines) = format_log_chunk(output) {
                for line in lines {
                    if tx.send(line).is_err() {
                        return;
                    }
                }
            }
        }
    });

    (rx, task.abort_handle())
}

pub fn format_log_chunk(output: LogOutput) -> Option<Vec<String>> {
    let text = match output {
        LogOutput::StdOut { message } | LogOutput::Console { message } => {
            String::from_utf8_lossy(&message).to_string()
        }
        LogOutput::StdErr { message } => {
            format!("[ERR] {}", String::from_utf8_lossy(&message))
        }
        _ => return None,
    };

    Some(text.lines().map(ToString::to_string).collect())
}

pub async fn stop_container(client: &Docker, container_id: &str) {
    let options = StopContainerOptionsBuilder::default().t(0).build();
    match client.stop_container(container_id, Some(options)).await {
        Ok(_) => {}
        Err(_) => {}
    }
}

pub async fn start_container(client: &Docker, container_id: &str) {
    let options: StartContainerOptions = Default::default();
    match client.start_container(container_id, Some(options)).await {
        Ok(_) => {}
        Err(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_format_log_chunk_stdout() {
        let chunk = LogOutput::StdOut {
            message: Bytes::from_static(b"Application listening on :8000\n"),
        };
        let lines = format_log_chunk(chunk).expect("Expected stdout lines");
        assert_eq!(lines, vec!["Application listening on :8000"]);
    }

    #[test]
    fn test_format_log_chunk_console() {
        let chunk = LogOutput::Console {
            message: Bytes::from_static(b"Interactive console output\n"),
        };
        let lines = format_log_chunk(chunk).expect("Expected console lines");
        assert_eq!(lines, vec!["Interactive console output"]);
    }

    #[test]
    fn test_format_log_chunk_stderr_adds_err_prefix() {
        let chunk = LogOutput::StdErr {
            message: Bytes::from_static(b"Connection refused"),
        };
        let lines = format_log_chunk(chunk).expect("Expected stderr lines");
        assert_eq!(lines, vec!["[ERR] Connection refused"]);
    }

    #[test]
    fn test_format_log_chunk_multiline_splitting() {
        let raw = b"first line\nsecond line\r\nthird line";
        let chunk = LogOutput::StdOut {
            message: Bytes::from_static(raw),
        };
        let lines = format_log_chunk(chunk).expect("Expected multiline parse");
        assert_eq!(lines, vec!["first line", "second line", "third line"]);
    }
}
