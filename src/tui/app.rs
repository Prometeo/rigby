use crate::container::{models::DockerContainer, service::list_containers};
use crate::utils::driver_connector;
use bollard::Docker;
use color_eyre::Result;
use ratatui::widgets::ListState;
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Focus {
    Menu,
    Content,
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
    pub client: Docker,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn tick(&self) -> Result<()> {
        Ok(())
    }

    pub async fn menu_data_loaded(&mut self) -> Result<()> {
        self.load_selected().await
    }

    pub async fn load_selected(&mut self) -> Result<()> {
        match self.menu.selected() {
            Some(MenuItem::Images) => return Ok(()),

            Some(MenuItem::Containers) => {
                if self.containers.items.is_empty() {
                    self.containers.items = list_containers(&self.client).await.unwrap_or_default();
                }
            }

            Some(MenuItem::Volumes) => return Ok(()),

            Some(MenuItem::Networks) => return Ok(()),
            None => {}
        }

        Ok(())
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
            focus: Focus::Menu,
            containers: StatefullList::default(),
            client,
        }
    }
}
