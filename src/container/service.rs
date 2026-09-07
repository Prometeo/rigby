use crate::container::models::DockerContainer;
use bollard::Docker;
use bollard::query_parameters::ListContainersOptionsBuilder;
use color_eyre::Result;

pub async fn list_containers(client: Docker) -> Result<Vec<DockerContainer>> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    let containers = client.list_containers(Some(options)).await?;

    let container_list = containers
        .into_iter()
        .map(DockerContainer::try_from)
        .collect::<Result<Vec<DockerContainer>, _>>()?;
    Ok(container_list)
}
