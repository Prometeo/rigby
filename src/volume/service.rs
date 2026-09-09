use crate::volume::models::DockerVolume;
use bollard::{Docker, query_parameters::ListVolumesOptions};
use color_eyre::Result;

pub async fn list_volumes(client: &Docker) -> Result<Vec<DockerVolume>> {
    let docker_response = client.list_volumes(None::<ListVolumesOptions>).await?;
    let volumes = docker_response.volumes.unwrap_or_default();
    let volume_list = volumes
        .into_iter()
        .map(DockerVolume::try_from)
        .collect::<Result<Vec<DockerVolume>, _>>()?;
    Ok(volume_list)
}
