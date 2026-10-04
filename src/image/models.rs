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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};
    use std::collections::HashMap;

    // -------------------- fixtures -------------------- //
    #[fixture]
    fn default_image() -> DockerImage {
        DockerImage {
            id: "sha256:1234567890abcdef".to_string(),
            tag: vec!["redis:alpine".to_string(), "redis:latest".to_string()],
            size: "10485760".to_string(),
            created: "2026-09-28 12:00:00".to_string(),
            containers: 2,
        }
    }

    // image without tags
    #[fixture]
    fn untagged_image(mut default_image: DockerImage) -> DockerImage {
        default_image.tag.clear();
        default_image
    }

    // Fully populated ImageConfig fixture
    #[fixture]
    fn full_image_config() -> ImageConfig {
        let mut labels = HashMap::new();
        labels.insert("maintainer".to_string(), "devops".to_string());

        ImageConfig {
            user: Some("appuser".to_string()),
            exposed_ports: Some(vec!["8080/tcp".to_string()]),
            env: Some(vec!["ENV=prod".to_string()]),
            cmd: Some(vec!["./run.sh".to_string()]),
            working_dir: Some("/app".to_string()),
            labels: Some(labels),
            entrypoint: Some(vec!["/bin/sh".to_string()]),
            volumes: Some(vec!["/data".to_string()]),
            ..Default::default()
        }
    }

    // -------------------- tests -------------------- //
    #[rstest]
    #[case(vec!["redis:alpine".to_string(), "redis:latest".to_string()], "redis:alpine")]
    #[case(vec!["my-app:1.0".to_string()], "my-app:1.0")]
    #[case(vec![], "<none>")]
    fn test_docker_image_display_tags(#[case] tags: Vec<String>, #[case] expected: &str) {
        let image = DockerImage {
            id: "sha256:test".into(),
            tag: tags,
            size: "1000".into(),
            created: "today".into(),
            containers: 1,
        };
        assert_eq!(format!("{image}"), expected);
    }

    #[rstest]
    fn test_docker_image_try_from_image_summary() {
        let summary = ImageSummary {
            id: "sha256:fedcba".to_string(),
            repo_tags: vec!["postgres:16".to_string()],
            size: 52428800,
            created: 1600000000,
            containers: 4,
            ..Default::default()
        };

        let image =
            DockerImage::try_from(summary).expect("Should convert cleanly from ImageSummary");
        assert_eq!(image.id, "sha256:fedcba");
        assert_eq!(image.tag, vec!["postgres:16"]);
        assert_eq!(image.size, "52428800");
        assert_eq!(image.containers, 4);
        assert!(!image.created.is_empty());
    }

    #[rstest]
    fn test_docker_image_detail_new_with_full_values(
        full_image_config: ImageConfig,
        default_image: DockerImage,
    ) {
        let detail = DockerImageDetail::new(full_image_config, &default_image);

        assert_eq!(detail.id, default_image.id);
        assert_eq!(detail.tag, "redis:alpine");
        assert_eq!(detail.user, "appuser");
        assert_eq!(detail.working_dir, "/app");
        assert_eq!(detail.env, vec!["ENV=prod"]);
        assert_eq!(detail.cmd, vec!["./run.sh"]);
        assert_eq!(detail.entrypoint, vec!["/bin/sh"]);
        assert_eq!(detail.labels.get("maintainer"), Some(&"devops".to_string()));
        assert_eq!(detail.containers, default_image.containers);
    }

    #[rstest]
    fn test_docker_image_detail_new_falls_back_on_none_fields(untagged_image: DockerImage) {
        let empty_config = ImageConfig::default();
        let detail = DockerImageDetail::new(empty_config, &untagged_image);

        assert_eq!(detail.tag, "<none>");
        assert_eq!(detail.user, "<none>");
        assert_eq!(detail.working_dir, "<none>");
        assert!(detail.ports.is_empty());
        assert!(detail.env.is_empty());
        assert!(detail.cmd.is_empty());
        assert!(detail.entrypoint.is_empty());
        assert!(detail.volumes.is_empty());
        assert!(detail.labels.is_empty());
    }
}
