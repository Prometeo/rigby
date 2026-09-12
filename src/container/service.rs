use crate::container::models::{DockerContainer, DockerContainerDetail};
use bollard::{
    Docker,
    container::LogOutput,
    query_parameters::{ListContainersOptionsBuilder, LogsOptions},
};
use color_eyre::Result;
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, UnboundedReceiver};
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

            let text = match output {
                LogOutput::StdOut { message } | LogOutput::Console { message } => {
                    String::from_utf8_lossy(&message).to_string()
                }
                LogOutput::StdErr { message } => {
                    format!("[ERR] {}", String::from_utf8_lossy(&message))
                }
                _ => continue,
            };

            for line in text.lines() {
                if tx.send(line.to_string()).is_err() {
                    return;
                }
            }
        }
    });

    (rx, task.abort_handle())
}
