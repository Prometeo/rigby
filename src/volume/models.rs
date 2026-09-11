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
