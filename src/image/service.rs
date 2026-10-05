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
