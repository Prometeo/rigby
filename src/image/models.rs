use bollard::models::ImageSummary;
use color_eyre::Report;
use std::fmt;

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
            created: "13-10-2026".into(),
            containers: image.containers,
        })
    }
}
