use crate::image::models::{DockerImage, DockerImageDetail};
use bollard::{Docker, query_parameters::ListImagesOptionsBuilder};
use color_eyre::{Result, eyre::eyre};

pub async fn list_images(client: &Docker) -> Result<Vec<DockerImage>> {
    let options = ListImagesOptionsBuilder::default().all(true).build();
    let images = client.list_images(Some(options)).await?;

    let image_list = images
        .into_iter()
        .map(DockerImage::try_from)
        .collect::<Result<Vec<DockerImage>, _>>()?;

    Ok(image_list)
}

pub async fn inspect_image(client: &Docker, image: &DockerImage) -> Result<DockerImageDetail> {
    let image_info = client.inspect_image(&image.id).await?;
    let image_config = image_info
        .config
        .ok_or_else(|| eyre!("Image has no configuration"))?;

    Ok(DockerImageDetail::new(image_config, image))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::parse_timestamp_from_epoch;
    use bollard::models::ImageSummary;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_converts_image_summary_to_docker_image() {
        let image = ImageSummary {
            id: "sha256:123".to_string(),
            repo_tags: vec!["nginx:latest".to_string()],
            size: 1024,
            created: 1_700_000_000,
            ..Default::default()
        };

        let docker_image = DockerImage::try_from(image).unwrap();

        assert_eq!(docker_image.id, "sha256:123");
        assert_eq!(docker_image.tag, vec!["nginx:latest"]);
        assert_eq!(docker_image.size, "1024");
        assert_eq!(
            docker_image.created,
            parse_timestamp_from_epoch(1_700_000_000).unwrap()
        );
    }
}
