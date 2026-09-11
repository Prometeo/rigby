use crate::container::models::{DockerContainer, DockerContainerDetail};
use bollard::{
    Docker,
    container::LogOutput,
    query_parameters::{ListContainersOptionsBuilder, LogsOptions},
};
use color_eyre::Result;
use futures_util::StreamExt;

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

pub async fn get_container_logs(
    client: &Docker,
    container_id_or_name: &str,
    tail: usize,
) -> Result<Vec<String>> {
    let options = LogsOptions {
        stdout: true,
        stderr: true,
        tail: tail.to_string(),
        timestamps: false,
        ..Default::default()
    };

    let mut stream = client.logs(container_id_or_name, Some(options));
    let mut logs = Vec::new();

    while let Some(output) = stream.next().await {
        match output? {
            LogOutput::StdOut { message } | LogOutput::Console { message } => {
                let text = String::from_utf8_lossy(&message);
                for line in text.lines() {
                    logs.push(line.to_string());
                }
            }
            LogOutput::StdErr { message } => {
                let text = String::from_utf8_lossy(&message);
                for line in text.lines() {
                    logs.push(format!("[ERR] {line}"));
                }
            }
            _ => {}
        }
    }

    Ok(logs)
}
