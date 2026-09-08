use bollard::{Docker, models::Network, query_parameters};
use color_eyre::Result;

use crate::networking::models::DockerNetwork;

pub async fn list_networks(client: &Docker) -> Result<Vec<DockerNetwork>> {
    let networks: Vec<Network> = client
        .list_networks(None::<query_parameters::ListNetworksOptions>)
        .await?;
    networks.into_iter().map(DockerNetwork::try_from).collect()
}
