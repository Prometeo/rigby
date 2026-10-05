use crate::utils::parse_timestamp_from_epoch;
use bollard::{
    models::{ContainerSummary, PortSummary},
    plugin::ContainerInspectResponse,
};
use color_eyre::{Report, eyre::eyre};
use std::fmt;

#[derive(Clone)]
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
        let id = container.id.ok_or_else(|| eyre!("Missing container ID"))?;
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

        let state = container.state.map(|s| s.to_string()).unwrap_or_default();

        Ok(Self {
            id,
            name,
            image: container.image.unwrap_or_default(),
            created: parse_timestamp_from_epoch(container.created.unwrap_or(0)).unwrap_or_default(),
            state,
            status: container.status.unwrap_or_default(),
            ports,
        })
    }
}

pub struct DockerContainerDetail {
    id: String,
    name: String,
    image: String,
    created: String,
    state: String,
    status: String,
    ports: String,
    args: Vec<String>,
    restart_count: i64,
    resolv_conf_path: String,
    hostname_path: String,
    log_path: String,
    driver: String,
    hosts_path: String,
    platform: String,
    mount_label: String,
    process_label: String,
    app_armor_profile: String,
    mounts: Vec<String>,
}

impl DockerContainerDetail {
    pub fn new(container_info: &ContainerInspectResponse, container: &DockerContainer) -> Self {
        Self {
            id: container.id.clone(),
            name: container.name.clone(),
            created: container.created.clone(),
            state: container.state.clone(),
            status: container.status.clone(),
            ports: container.ports.clone(),
            image: container.image.clone(),
            args: container_info.args.clone().unwrap_or_default(),
            restart_count: container_info.restart_count.unwrap_or_default(),
            resolv_conf_path: container_info.resolv_conf_path.clone().unwrap_or_default(),
            hostname_path: container_info.hostname_path.clone().unwrap_or_default(),
            log_path: container_info.log_path.clone().unwrap_or_default(),
            driver: container_info.driver.clone().unwrap_or_default(),
            hosts_path: container_info.hosts_path.clone().unwrap_or_default(),
            platform: container_info.platform.clone().unwrap_or_default(),
            mount_label: container_info.mount_label.clone().unwrap_or_default(),
            process_label: container_info.process_label.clone().unwrap_or_default(),
            app_armor_profile: container_info.app_armor_profile.clone().unwrap_or_default(),
            mounts: container_info
                .mounts
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|mount| {
                    format!(
                        "{}:{}",
                        mount.source.unwrap_or_else(|| "<none>".into()),
                        mount.destination.unwrap_or_else(|| "<none>".into())
                    )
                })
                .collect::<Vec<_>>(),
        }
    }
}

impl fmt::Display for DockerContainerDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field_width: usize = 20;

        writeln!(f, "{:<width$}{}", "ID", self.id, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Name", self.name, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Image", self.image, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "Created",
            self.created,
            width = field_width
        )?;
        writeln!(f, "{:<width$}{}", "State", self.state, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "Status",
            self.status,
            width = field_width
        )?;
        writeln!(f, "{:<width$}{}", "Ports", self.ports, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "Args",
            self.args.join(", "),
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "RestartCount",
            self.restart_count,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "ResolvConfPath",
            self.resolv_conf_path,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "HostnamePath",
            self.hostname_path,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Platform",
            self.platform,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "MountLabel",
            self.mount_label,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "ProcessLabel",
            self.process_label,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "AppArmorProfile",
            self.app_armor_profile,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "LogPath",
            self.log_path,
            width = field_width
        )?;
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
            "HostsPath",
            self.hosts_path,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Mounts",
            self.mounts.join(", "),
            width = field_width
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::{ContainerSummary, ContainerSummaryStateEnum};

    #[test]
    fn test_docker_container_try_from_valid_summary() {
        let summary = ContainerSummary {
            id: Some("abc123456789".to_string()),
            names: Some(vec!["/some-backend".to_string()]),
            image: Some("some:latest".to_string()),
            state: Some(ContainerSummaryStateEnum::RUNNING),
            status: Some("Up 2 hours".to_string()),
            ..Default::default()
        };

        let container = DockerContainer::try_from(summary)
            .expect("Should convert valid summary to DockerContainer");

        assert_eq!(container.id, "abc123456789");
        assert_eq!(container.name, "some-backend");
        assert_eq!(container.state, "running");
    }

    #[test]
    fn test_docker_container_try_from_missing_id_fails() {
        let summary = ContainerSummary {
            id: None,
            names: Some(vec!["/nameless".to_string()]),
            ..Default::default()
        };

        let result = DockerContainer::try_from(summary);
        assert!(result.is_err(), "Expected error when ID is missing");
    }
}
