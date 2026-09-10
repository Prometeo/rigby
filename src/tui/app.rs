use crate::container::{
    models::{DockerContainer, DockerContainerDetail},
    service::{inspect_container, list_containers},
};
use crate::image::{
    models::{DockerImage, DockerImageDetail},
    service::{inspect_image, list_images},
};
use crate::networking::{models::DockerNetwork, service::list_networks};
use crate::utils::driver_connector;
use crate::volume::{models::DockerVolume, service::list_volumes};
use bollard::Docker;
use color_eyre::Result;
use ratatui::widgets::ListState;
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Focus {
    ItemsList,
}

#[derive(Debug, Clone)]
pub enum MenuItem {
    Images,
    Containers,
    Volumes,
    Networks,
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuItem::Images => write!(f, "Images"),
            MenuItem::Containers => write!(f, "Containers"),
            MenuItem::Volumes => write!(f, "Volumes"),
            MenuItem::Networks => write!(f, "Networks"),
        }
    }
}

pub enum Details {
    Image(DockerImageDetail),
    Container(DockerContainerDetail),
}

impl fmt::Display for Details {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Details::Image(details) => write!(f, "{details}"),
            Details::Container(details) => write!(f, "{details}"),
        }
    }
}

#[derive(Debug)]
pub struct StatefullList<T> {
    pub items: Vec<T>,
    pub state: ListState,
}

impl<T> StatefullList<T> {
    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn selected(&self) -> Option<&T> {
        let index: usize = self.state.selected()?;
        self.items.get(index)
    }

    pub fn move_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i: usize = self.state.selected().unwrap_or(0);
        self.state.select(Some(i.saturating_sub(1)));
    }

    pub fn move_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i: usize = self.state.selected().unwrap_or(0);
        let next: usize = (i + 1).min(self.items.len() - 1);
        self.state.select(Some(next));
    }
}

impl<T> Default for StatefullList<T> {
    fn default() -> Self {
        let mut state: ListState = ListState::default();
        state.select(Some(0));
        Self {
            items: Vec::new(),
            state,
        }
    }
}

pub struct App {
    pub quit: bool,
    pub menu: StatefullList<MenuItem>,
    pub focus: Focus,
    pub containers: StatefullList<DockerContainer>,
    pub images: StatefullList<DockerImage>,
    pub networks: StatefullList<DockerNetwork>,
    pub volumes: StatefullList<DockerVolume>,
    pub client: Docker,
    pub details: Option<Details>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn tick(&self) -> Result<()> {
        Ok(())
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::ItemsList => Focus::ItemsList,
        }
    }

    pub async fn menu_data_loaded(&mut self) {
        self.load_selected().await
    }

    pub async fn move_items_list_up(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => self.images.move_up(),
            Some(MenuItem::Containers) => self.containers.move_up(),
            Some(MenuItem::Networks) => self.networks.move_up(),
            Some(MenuItem::Volumes) => self.volumes.move_up(),
            _ => {}
        }

        self.load_selected_details().await;
    }

    pub async fn move_items_list_down(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => self.images.move_down(),
            Some(MenuItem::Containers) => self.containers.move_down(),
            Some(MenuItem::Networks) => self.networks.move_down(),
            Some(MenuItem::Volumes) => self.volumes.move_down(),
            _ => {}
        }

        self.load_selected_details().await;
    }

    pub async fn select_menu_item(&mut self, option: char) {
        match option {
            '1' => self.menu.state.select(Some(0)),
            '2' => self.menu.state.select(Some(1)),
            '3' => self.menu.state.select(Some(2)),
            '4' => self.menu.state.select(Some(3)),
            _ => {}
        }
        self.load_selected().await;
    }

    pub async fn load_selected(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => {
                if self.images.items.is_empty() {
                    self.images.items = list_images(&self.client).await.unwrap_or_default();
                }
                self.images.select_first();
            }

            Some(MenuItem::Containers) => {
                if self.containers.items.is_empty() {
                    self.containers.items = list_containers(&self.client).await.unwrap_or_default();
                }
                self.containers.select_first();
            }

            Some(MenuItem::Volumes) => {
                if self.volumes.items.is_empty() {
                    self.volumes.items = list_volumes(&self.client).await.unwrap_or_default();
                }
                self.volumes.select_first();
            }

            Some(MenuItem::Networks) => {
                if self.networks.items.is_empty() {
                    self.networks.items = list_networks(&self.client).await.unwrap_or_default();
                }
                self.containers.select_first();
            }
            None => {}
        }

        self.load_selected_details().await;
    }

    pub async fn load_selected_details(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => {
                let Some(image) = self.images.selected().cloned() else {
                    self.details = None;
                    return;
                };

                if let Ok(details) = inspect_image(&self.client, &image).await {
                    self.details = Some(Details::Image(details))
                };
            }
            Some(MenuItem::Containers) => {
                let Some(container) = self.containers.selected().cloned() else {
                    self.details = None;
                    return;
                };

                if let Ok(details) = inspect_container(&self.client, &container).await {
                    self.details = Some(Details::Container(details))
                }
            }
            _ => {}
        };
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }
}

impl Default for App {
    fn default() -> Self {
        let mut menu: StatefullList<MenuItem> = StatefullList::default();
        let client = driver_connector()
            .expect("Failed to connect to the Docker daemon. Is the docker daemon runing?");

        menu.items = vec![
            MenuItem::Images,
            MenuItem::Containers,
            MenuItem::Volumes,
            MenuItem::Networks,
        ];
        Self {
            quit: false,
            menu,
            focus: Focus::ItemsList,
            containers: StatefullList::default(),
            images: StatefullList::default(),
            networks: StatefullList::default(),
            volumes: StatefullList::default(),
            client,
            details: None,
        }
    }
}
