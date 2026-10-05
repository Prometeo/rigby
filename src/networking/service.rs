use crate::networking::models::{DockerNetwork, DockerNetworkDetail};
use bollard::{Docker, models::Network, query_parameters};
use color_eyre::Result;

pub async fn list_networks(client: &Docker) -> Result<Vec<DockerNetwork>> {
    let networks: Vec<Network> = client
        .list_networks(None::<query_parameters::ListNetworksOptions>)
        .await?;
    networks.into_iter().map(DockerNetwork::try_from).collect()
}
pub async fn inspect_network(
    client: &Docker,
    network: &DockerNetwork,
) -> Result<DockerNetworkDetail> {
    let network_info = client
        .inspect_network(
            &network.name,
            Some(query_parameters::InspectNetworkOptions::default()),
        )
        .await?;
    Ok(DockerNetworkDetail::new(network, network_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_converts_network_to_docker_network() {
        let network = Network {
            name: Some("network-test".into()),
            ..Default::default()
        };

        let docker_network = DockerNetwork::try_from(network).unwrap();
        assert_eq!(docker_network.name, "network-test");
    }
}
