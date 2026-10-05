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
        let ipam_info = network_info.ipam.unwrap_or_default();
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

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::{EndpointResource, Ipam, IpamConfig, Network, NetworkInspect};
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use std::collections::HashMap;

    // -------------------- fixtures -------------------- //
    #[fixture]
    fn default_docker_network() -> DockerNetwork {
        DockerNetwork {
            name: "bridge".to_string(),
        }
    }

    #[fixture]
    fn full_network_inspect() -> NetworkInspect {
        let mut labels = HashMap::new();
        labels.insert(
            "com.docker.network.bridge.name".to_string(),
            "docker0".to_string(),
        );

        let mut options = HashMap::new();
        options.insert("icc".to_string(), "true".to_string());

        let mut containers = HashMap::new();
        containers.insert(
            "container-abc-123".to_string(),
            EndpointResource {
                name: Some("web-app".to_string()),
                endpoint_id: Some("ep-1".to_string()),
                mac_address: Some("02:42:ac:11:00:02".to_string()),
                ipv4_address: Some("172.17.0.2/16".to_string()),
                ipv6_address: Some("".to_string()),
            },
        );

        let ipam_config = IpamConfig {
            subnet: Some("172.17.0.0/16".to_string()),
            gateway: Some("172.17.0.1".to_string()),
            ip_range: Some("172.17.0.0/24".to_string()),
            auxiliary_addresses: None,
        };

        let ipam = Ipam {
            driver: Some("default".to_string()),
            config: Some(vec![ipam_config]),
            options: None,
        };

        NetworkInspect {
            id: Some("network-id-xyz789".to_string()),
            name: Some("bridge".to_string()),
            created: Some("2026-10-04T12:00:00Z".to_string()),
            scope: Some("local".to_string()),
            driver: Some("bridge".to_string()),
            enable_ipv6: Some(false),
            internal: Some(false),
            attachable: Some(true),
            ingress: Some(false),
            containers: Some(containers),
            options: Some(options),
            labels: Some(labels),
            ipam: Some(ipam),
            ..Default::default()
        }
    }

    #[fixture]
    fn empty_network_inspect() -> NetworkInspect {
        NetworkInspect {
            ipam: Some(Ipam::default()),
            ..Default::default()
        }
    }

    // -------------------- tests -------------------- //

    #[rstest]
    #[case("custom-network", "custom-network")]
    #[case("host", "host")]
    fn test_docker_network_display(#[case] name: &str, #[case] expected: &str) {
        let network = DockerNetwork {
            name: name.to_string(),
        };
        assert_eq!(format!("{network}"), expected);
    }

    #[rstest]
    fn test_docker_network_try_from_valid_network() {
        let raw = Network {
            name: Some("frontend-tier".to_string()),
            ..Default::default()
        };

        let result = DockerNetwork::try_from(raw).expect("Should convert cleanly");
        assert_eq!(result.name, "frontend-tier");
    }

    #[rstest]
    fn test_docker_network_try_from_none_name_falls_back() {
        let raw = Network {
            name: None,
            ..Default::default()
        };

        let result = DockerNetwork::try_from(raw).expect("Should convert with fallback");
        assert_eq!(result.name, "<none>");
    }

    #[rstest]
    fn test_docker_network_detail_new_with_full_values(
        default_docker_network: DockerNetwork,
        full_network_inspect: NetworkInspect,
    ) {
        let detail = DockerNetworkDetail::new(&default_docker_network, full_network_inspect);

        assert_eq!(detail.id, "network-id-xyz789");
        assert_eq!(detail.name, "bridge");
        assert_eq!(detail.created, "2026-10-04T12:00:00Z");
        assert_eq!(detail.scope, "local");
        assert_eq!(detail.driver, "bridge");
        assert_eq!(detail.enable_ipv6, false);
        assert_eq!(detail.internal, false);
        assert_eq!(detail.attachable, true);
        assert_eq!(detail.ingress, false);
        assert_eq!(detail.options.get("icc"), Some(&"true".to_string()));
        assert_eq!(
            detail.labels.get("com.docker.network.bridge.name"),
            Some(&"docker0".to_string())
        );

        assert_eq!(detail.ipam.driver, "default");
        assert_eq!(detail.ipam.configs.len(), 1);
        assert_eq!(
            detail.ipam.configs[0].subnet,
            Some("172.17.0.0/16".to_string())
        );
        assert_eq!(
            detail.ipam.configs[0].gateway,
            Some("172.17.0.1".to_string())
        );
        assert_eq!(
            detail.ipam.configs[0].ip_range,
            Some("172.17.0.0/24".to_string())
        );
    }

    #[rstest]
    fn test_docker_network_detail_new_with_none_fields_uses_fallbacks(
        default_docker_network: DockerNetwork,
        empty_network_inspect: NetworkInspect,
    ) {
        let detail = DockerNetworkDetail::new(&default_docker_network, empty_network_inspect);

        assert_eq!(detail.id, "<none>");
        assert_eq!(detail.created, "<none>");
        assert_eq!(detail.scope, "<none>");
        assert_eq!(detail.driver, "<none>");
        assert_eq!(detail.enable_ipv6, false);
        assert_eq!(detail.internal, false);
        assert_eq!(detail.attachable, false);
        assert_eq!(detail.ingress, false);
        assert!(detail.containers.is_empty());
        assert!(detail.options.is_empty());
        assert!(detail.labels.is_empty());
        assert_eq!(detail.ipam.driver, "<none>");
        assert!(detail.ipam.configs.is_empty());
    }
}
