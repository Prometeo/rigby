use bollard::models::Network;
use color_eyre::Report;
use std::fmt;

pub struct DockerNetwork {
    name: String,
}

impl fmt::Display for DockerNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<Network> for DockerNetwork {
    type Error = Report;

    fn try_from(network: Network) -> Result<Self, Self::Error> {
        Ok(Self {
            name: network.name.unwrap_or("<none>".into()),
        })
    }
}
