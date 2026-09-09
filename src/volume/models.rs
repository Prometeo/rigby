use bollard::models::Volume;
use color_eyre::Report;
use std::fmt;

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
