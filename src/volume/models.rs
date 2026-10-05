use bollard::{
    models::Volume,
    plugin::{VolumeScopeEnum, VolumeUsageData},
};
use color_eyre::Report;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub struct DockerVolume {
    pub name: String,
}

impl fmt::Display for DockerVolume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<Volume> for DockerVolume {
    type Error = Report;

    fn try_from(volume: Volume) -> Result<Self, Self::Error> {
        Ok(Self { name: volume.name })
    }
}

pub struct DockerVolumeDetail {
    name: String,
    driver: String,
    mountpoint: String,
    created: String,
    status: Vec<String>,
    labels: HashMap<String, String>,
    scope: VolumeScopeEnum,
    options: HashMap<String, String>,
    volume_usage_data: Option<VolumeUsageData>,
}

impl DockerVolumeDetail {
    pub fn new(volume: &DockerVolume, volume_detail: Volume) -> Self {
        Self {
            name: volume.name.clone(),
            driver: volume_detail.driver,
            mountpoint: volume_detail.mountpoint,
            created: volume_detail.created_at.unwrap_or("<none>".into()),
            status: volume_detail.status.unwrap_or_default(),
            labels: volume_detail.labels,
            scope: volume_detail.scope.unwrap_or(VolumeScopeEnum::EMPTY),
            options: volume_detail.options,
            volume_usage_data: volume_detail.usage_data,
        }
    }
}

impl fmt::Display for DockerVolumeDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field_width: usize = 16;
        let labels = self
            .labels
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ");
        let options = self
            .options
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ");
        let scope = match self.scope {
            VolumeScopeEnum::EMPTY => "",
            VolumeScopeEnum::LOCAL => "Local",
            VolumeScopeEnum::GLOBAL => "Global",
        };
        let (size, ref_count) = match &self.volume_usage_data {
            Some(usage) => (usage.size.to_string(), usage.ref_count.to_string()),
            None => ("<unknown>".to_string(), "<unknown>".to_string()),
        };
        writeln!(f, "{:<width$}{}", "Name:", self.name, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "Driver:",
            self.driver,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "MountPoint:",
            self.mountpoint,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Created:",
            self.created,
            width = field_width
        )?;
        writeln!(
            f,
            "{:<width$}{}",
            "Status:",
            self.status.join(", "),
            width = field_width
        )?;
        writeln!(f, "{:<width$}{}", "Labels:", labels, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Scope:", scope, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Options:", options, width = field_width)?;
        writeln!(f, "{:<width$}{}", "Size:", size, width = field_width)?;
        writeln!(
            f,
            "{:<width$}{}",
            "RefCount:",
            ref_count,
            width = field_width
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::{
        models::Volume,
        plugin::{VolumeScopeEnum, VolumeUsageData},
    };
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use std::collections::HashMap;

    // -------------------- fixtures -------------------- //

    #[fixture]
    fn default_docker_volume() -> DockerVolume {
        DockerVolume {
            name: "pgdata".to_string(),
        }
    }

    #[fixture]
    fn full_volume_detail() -> Volume {
        let mut labels = HashMap::new();
        labels.insert("com.example.vendor".to_string(), "acme".to_string());

        let mut options = HashMap::new();
        options.insert("type".to_string(), "tmpfs".to_string());

        Volume {
            name: "pgdata".to_string(),
            driver: "local".to_string(),
            mountpoint: "/var/lib/docker/volumes/pgdata/_data".to_string(),
            created_at: Some("2026-10-04T12:00:00Z".to_string()),
            status: Some(vec!["ready".to_string(), "active".to_string()]),
            labels,
            scope: Some(VolumeScopeEnum::LOCAL),
            options,
            usage_data: Some(VolumeUsageData {
                size: 1048576,
                ref_count: 2,
            }),
            ..Default::default()
        }
    }

    #[fixture]
    fn empty_volume_detail() -> Volume {
        Volume {
            name: "minimal-volume".to_string(),
            driver: "local".to_string(),
            mountpoint: "/var/lib/docker/volumes/minimal-volume/_data".to_string(),
            created_at: None,
            status: None,
            labels: HashMap::new(),
            scope: None,
            options: HashMap::new(),
            usage_data: None,
            ..Default::default()
        }
    }

    // -------------------- tests -------------------- //

    #[rstest]
    #[case("pgdata", "pgdata")]
    #[case("nginx_cache", "nginx_cache")]
    fn test_docker_volume_display(#[case] name: &str, #[case] expected: &str) {
        let volume = DockerVolume {
            name: name.to_string(),
        };
        assert_eq!(format!("{volume}"), expected);
    }

    #[rstest]
    fn test_docker_volume_try_from() {
        let raw = Volume {
            name: "app-storage".to_string(),
            driver: "local".to_string(),
            mountpoint: "/data".to_string(),
            created_at: None,
            status: None,
            labels: HashMap::new(),
            scope: None,
            options: HashMap::new(),
            usage_data: None,
            ..Default::default()
        };

        let result = DockerVolume::try_from(raw).expect("Should convert cleanly");
        assert_eq!(result.name, "app-storage");
    }

    #[rstest]
    fn test_docker_volume_detail_new_with_full_values(
        default_docker_volume: DockerVolume,
        full_volume_detail: Volume,
    ) {
        let detail = DockerVolumeDetail::new(&default_docker_volume, full_volume_detail);

        assert_eq!(detail.name, "pgdata");
        assert_eq!(detail.driver, "local");
        assert_eq!(detail.mountpoint, "/var/lib/docker/volumes/pgdata/_data");
        assert_eq!(detail.created, "2026-10-04T12:00:00Z");
        assert_eq!(detail.status, vec!["ready", "active"]);
        assert_eq!(
            detail.labels.get("com.example.vendor"),
            Some(&"acme".to_string())
        );
        assert_eq!(detail.scope, VolumeScopeEnum::LOCAL);
        assert_eq!(detail.options.get("type"), Some(&"tmpfs".to_string()));

        let usage = detail.volume_usage_data.expect("Usage data should exist");
        assert_eq!(usage.size, 1048576);
        assert_eq!(usage.ref_count, 2);
    }

    #[rstest]
    fn test_docker_volume_detail_new_with_none_fields_uses_fallbacks(
        default_docker_volume: DockerVolume,
        empty_volume_detail: Volume,
    ) {
        let detail = DockerVolumeDetail::new(&default_docker_volume, empty_volume_detail);

        assert_eq!(detail.created, "<none>");
        assert!(detail.status.is_empty());
        assert_eq!(detail.scope, VolumeScopeEnum::EMPTY);
        assert!(detail.labels.is_empty());
        assert!(detail.options.is_empty());
        assert!(detail.volume_usage_data.is_none());
    }

    #[rstest]
    #[case(VolumeScopeEnum::EMPTY, "")]
    #[case(VolumeScopeEnum::LOCAL, "Local")]
    #[case(VolumeScopeEnum::GLOBAL, "Global")]
    fn test_docker_volume_detail_display_scope_variants(
        default_docker_volume: DockerVolume,
        empty_volume_detail: Volume,
        #[case] scope_variant: VolumeScopeEnum,
        #[case] expected_scope_str: &str,
    ) {
        let mut raw = empty_volume_detail;
        raw.scope = Some(scope_variant);

        let detail = DockerVolumeDetail::new(&default_docker_volume, raw);
        let output = format!("{detail}");

        let expected_line = format!("{:<16}{}", "Scope:", expected_scope_str);
        assert!(output.contains(&expected_line));
    }
}
