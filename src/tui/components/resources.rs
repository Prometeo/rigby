use super::StatefullList;
use crate::container::models::DockerContainer;
use crate::image::models::DockerImage;
use crate::networking::models::DockerNetwork;
use crate::volume::models::DockerVolume;

pub struct DockerResources {
    pub containers: StatefullList<DockerContainer>,
    pub images: StatefullList<DockerImage>,
    pub networks: StatefullList<DockerNetwork>,
    pub volumes: StatefullList<DockerVolume>,
}

impl DockerResources {
    pub fn new() -> Self {
        Self {
            containers: StatefullList::default(),
            images: StatefullList::default(),
            networks: StatefullList::default(),
            volumes: StatefullList::default(),
        }
    }
}
