use bollard::models::{ContainerSummary, PortSummary};
use color_eyre::Report;
use std::fmt;

pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub created: String,
    pub state: String,
    pub status: String,
    pub ports: String,
}

impl fmt::Display for DockerContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<ContainerSummary> for DockerContainer {
    type Error = Report;

    fn try_from(container: ContainerSummary) -> Result<Self, Self::Error> {
        let name: String = container
            .names
            .as_ref()
            .and_then(|names| names.first())
            .map(|name| name.trim_start_matches("/").to_string())
            .unwrap_or_default();
        let ports: PortSummary = container
            .ports
            .as_ref()
            .and_then(|port_object| port_object.first())
            .cloned()
            .unwrap_or_default();
        let ports = format!(
            "{}:{}",
            ports.private_port,
            ports.public_port.unwrap_or_default()
        );

        Ok(Self {
            id: container.id.unwrap_or_default(),
            name: name,
            image: container.image.unwrap_or_default(),
            created: "10/10/2020".into(),
            state: container.state.unwrap().to_string(),
            status: container.status.unwrap_or_default(),
            ports: ports,
        })
    }
}
