use crate::utils::parse_timestamp_from_epoch;
use bollard::models::{ImageConfig, ImageSummary};
use color_eyre::Report;
use std::{collections::HashMap, fmt};

#[derive(Clone)]
pub struct DockerImage {
    pub id: String,
    pub tag: Vec<String>,
    pub size: String,
    pub created: String,
    pub containers: i64,
}

impl fmt::Display for DockerImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag: &str = self.tag.first().map(String::as_str).unwrap_or("<none>");
        write!(f, "{tag}")
    }
}

impl TryFrom<ImageSummary> for DockerImage {
    type Error = Report;

    fn try_from(image: ImageSummary) -> Result<Self, Self::Error> {
        Ok(Self {
            id: image.id,
            tag: image.repo_tags,
            size: image.size.to_string(),
            created: parse_timestamp_from_epoch(image.created).unwrap_or_default(),
            containers: image.containers,
        })
    }
}

pub struct DockerImageDetail {
    pub id: String,
    pub tag: String,
    pub size: String,
    pub created: String,
    pub user: String,
    pub ports: Vec<String>,
    pub env: Vec<String>,
    pub cmd: Vec<String>,
    pub working_dir: String,
    pub labels: HashMap<String, String>,
    pub entrypoint: Vec<String>,
    pub volumes: Vec<String>,
    pub containers: i64,
}

impl DockerImageDetail {
    pub fn new(image_info: ImageConfig, image: &DockerImage) -> Self {
        let tag: String = image
            .tag
            .first()
            .map(String::as_str)
            .unwrap_or("<none>")
            .into();

        Self {
            id: image.id.clone(),
            tag,
            size: image.size.clone(),
            created: image.created.clone(),
            user: image_info.user.unwrap_or("<none>".into()),
            ports: image_info.exposed_ports.unwrap_or_default(),
            env: image_info.env.unwrap_or_default(),
            cmd: image_info.cmd.unwrap_or_default(),
            working_dir: image_info.working_dir.unwrap_or("<none>".into()),
            labels: image_info.labels.unwrap_or_default(),
            entrypoint: image_info.entrypoint.unwrap_or_default(),
            volumes: image_info.volumes.unwrap_or_default(),
            containers: image.containers,
        }
    }
}

impl fmt::Display for DockerImageDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field_width: usize = 16;

        let ports = self.ports.join(", ");
        let env = self.env.join(", ");
        let cmd = self.cmd.join(", ");

        let labels = self
            .labels
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("\n");

        let labels = labels.replace('\n', &format!("\n{:width$}", "", width = field_width));

        let entrypoint = self.entrypoint.join(", ");
        let volumes = self.volumes.join(", ");

        writeln!(f, "{:<width$}{}", "ID:", self.id, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Tag:", self.tag, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Size:", self.size, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "Created:",
            self.created,
            width = field_width
        )?;
        writeln!(f, "{:<width$}{}", "User:", self.user, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Ports:", ports, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Env:", env, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Cmd:", cmd, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "WorkingDir:",
            self.working_dir,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Entrypoint:",
            entrypoint,
            width = field_width
        )?;
        writeln!(f, "{:<width$}{}", "Volumes:", volumes, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "Containers:",
            self.containers,
            width = field_width
        )?;
        writeln!(f, "{:<width$}{}", "Labels:", labels, width = field_width)?;

        Ok(())
    }
}
