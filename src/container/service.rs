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
use tokio::sync::mpsc::{self, UnboundedReceiver, unbounded_channel};
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
    let (tx, rx) = mpsc::unbounded_channel();

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
