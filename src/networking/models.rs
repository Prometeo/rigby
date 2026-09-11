use bollard::models::{EndpointResource, Network, NetworkInspect};
use color_eyre::Report;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub struct DockerNetwork {
    pub name: String,
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

pub struct DockerNetworkIpam {
    driver: String,
    configs: Vec<DockerNetworkIpamConfig>,
}

pub struct DockerNetworkIpamConfig {
    subnet: Option<String>,
    gateway: Option<String>,
    ip_range: Option<String>,
}

pub struct DockerNetworkDetail {
    id: String,
    name: String,
    created: String,
    scope: String,
    driver: String,
    enable_ipv6: bool,
    internal: bool,
    attachable: bool,
    ingress: bool,
    containers: HashMap<String, EndpointResource>,
    options: HashMap<String, String>,
    labels: HashMap<String, String>,
    ipam: DockerNetworkIpam,
}

impl DockerNetworkDetail {
    pub fn new(network: &DockerNetwork, network_info: NetworkInspect) -> Self {
        let ipam_info = network_info.ipam.unwrap();
        let configs = ipam_info
            .config
            .unwrap_or_default()
            .into_iter()
            .map(|config| DockerNetworkIpamConfig {
                subnet: config.subnet,
                gateway: config.gateway,
                ip_range: config.ip_range,
            })
            .collect();
        let ipam = DockerNetworkIpam {
            driver: ipam_info.driver.unwrap_or("<none>".into()),
            configs,
        };

        Self {
            id: network_info.id.unwrap_or("<none>".into()),
            name: network.name.clone(),
            created: network_info.created.unwrap_or("<none>".into()),
            scope: network_info.scope.unwrap_or("<none>".into()),
            driver: network_info.driver.unwrap_or("<none>".into()),
            enable_ipv6: network_info.enable_ipv6.unwrap_or_default(),
            internal: network_info.internal.unwrap_or_default(),
            attachable: network_info.attachable.unwrap_or_default(),
            ingress: network_info.ingress.unwrap_or_default(),
            containers: network_info.containers.unwrap_or_default(),
            options: network_info.options.unwrap_or_default(),
            labels: network_info.labels.unwrap_or_default(),
            ipam,
        }
    }
}

impl fmt::Display for DockerNetworkDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field_width: usize = 16;
        let containers = self
            .containers
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        let options = self
            .options
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ");
        let labels = self
            .labels
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(f, "{:<width$}{}", "ID", self.id, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Name", self.name, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "Created",
            self.created,
            width = field_width
        )?;
        writeln!(f, "{:<width$}{}", "Scope", self.scope, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "Driver",
            self.driver,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "EnableIPv6",
            self.enable_ipv6,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Internal",
            self.internal,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Attachable",
            self.attachable,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Ingress",
            self.ingress,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Containers:",
            containers,
            width = field_width
        )?;
        writeln!(f, "{:<width$}{}", "Options", options, width = field_width)?;
        // IPAM
        writeln!(f, "{:<width$}{}", "Labels", labels, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "IPAM:",
            self.ipam.driver,
            width = field_width
        )?;
        for config in &self.ipam.configs {
            writeln!(
                f,
                "{:<width$}Subnet: {}",
                "",
                config.subnet.as_deref().unwrap_or("<none>"),
                width = field_width
            )?;

            writeln!(
                f,
                "{:<width$}Gateway: {}",
                "",
                config.gateway.as_deref().unwrap_or("<none>"),
                width = field_width
            )?;

            writeln!(
                f,
                "{:<width$} IPRange: {}",
                "",
                config.ip_range.as_deref().unwrap_or("<none>"),
                width = field_width
            )?;
        }
        Ok(())
    }
}
