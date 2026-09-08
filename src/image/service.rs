use crate::image::models::{self, DockerImage};
use bollard::{Docker, query_parameters::ListImagesOptionsBuilder};
use color_eyre::Result;

pub async fn list_images(client: &Docker) -> Result<Vec<DockerImage>> {
    let options = ListImagesOptionsBuilder::default().all(true).build();
    let images = client.list_images(Some(options)).await?;

    let image_list = images
        .into_iter()
        .map(DockerImage::try_from)
        .collect::<Result<Vec<DockerImage>, _>>()?;

    Ok(image_list)
}
