use crate::image::{
    models::{DockerImage, DockerImageDetail},
    service::{inspect_image, list_images},
};
use crate::networking::{
    models::{DockerNetwork, DockerNetworkDetail},
    service::{inspect_network, list_networks},
};
use crate::utils::driver_connector;
use crate::volume::{
    models::{DockerVolume, DockerVolumeDetail},
    service::list_volumes,
};
use crate::{
    container::{
        models::{DockerContainer, DockerContainerDetail},
        service::{
            get_container_logs, inspect_container, list_containers, start_container, stop_container,
        },
    },
    volume::service::inspect_volume,
};
use bollard::Docker;
use color_eyre::Result;
use ratatui::widgets::ListState;
use std::fmt;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::task::AbortHandle;

#[derive(PartialEq, Eq)]
pub enum Focus {
    ItemsList,
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

pub enum Details {
    Image(DockerImageDetail),
    Container(DockerContainerDetail),
    Volume(DockerVolumeDetail),
    Network(DockerNetworkDetail),
}

impl fmt::Display for Details {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Details::Image(details) => write!(f, "{details}"),
            Details::Container(details) => write!(f, "{details}"),
            Details::Network(details) => write!(f, "{details}"),
            Details::Volume(details) => write!(f, "{details}"),
        }
    }
}

#[derive(Debug)]
pub struct StatefullList<T> {
    pub items: Vec<T>,
    pub state: ListState,
}

impl<T> StatefullList<T> {
    pub fn update_items<F, K>(&mut self, mut new_items: Vec<T>, key_fn: F)
    where
        F: Fn(&T) -> K,
        K: Ord,
    {
        let selected_key = self.selected().map(&key_fn);

        new_items.sort_by_key(&key_fn);
        self.items = new_items;

        // Restore selection
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }

        if let Some(target_key) = selected_key {
            // Find where the previously selected item is now
            if let Some(new_pos) = self
                .items
                .iter()
                .position(|item| key_fn(item) == target_key)
            {
                self.state.select(Some(new_pos));
                return;
            }
        }

        let current_index = self.state.selected().unwrap_or(0);
        self.state
            .select(Some(current_index.min(self.items.len() - 1)));
    }

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

pub struct DockerPollData {
    pub containers: Vec<DockerContainer>,
    pub images: Vec<DockerImage>,
    pub volumes: Vec<DockerVolume>,
    pub networks: Vec<DockerNetwork>,
}

#[derive(Default, PartialEq, Eq, Clone)]
pub enum ContainerTab {
    #[default]
    Details,
    Logs,
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
    pub container_tab: ContainerTab,
    pub container_logs: Vec<String>,
    pub log_rx: Option<UnboundedReceiver<String>>,
    pub log_abort_handle: Option<AbortHandle>,
    pub log_vertical_scroll: u16,
    pub log_horizontal_scroll: u16,
    pub log_auto_scroll: bool,
    pub log_visible_height: u16,
    pub log_visible_width: u16,
    pub poll_rx: Option<UnboundedReceiver<DockerPollData>>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn tick(&mut self) -> Result<()> {
        const MAX_LOG_LINES: usize = 5000;

        if let Some(rx) = self.log_rx.as_mut() {
            while let Ok(line) = rx.try_recv() {
                self.container_logs.push(line);
            }

            if self.container_logs.len() > MAX_LOG_LINES {
                let excess = self.container_logs.len() - MAX_LOG_LINES;
                self.container_logs.drain(0..excess);
                self.log_vertical_scroll = self.log_vertical_scroll.saturating_sub(excess as u16);
            }
        }

        if let Some(rx) = self.poll_rx.as_mut() {
            while let Ok(data) = rx.try_recv() {
                self.containers
                    .update_items(data.containers, |c| c.id.clone());
                self.images.update_items(data.images, |i| i.id.clone());
                self.volumes.update_items(data.volumes, |v| v.name.clone());
                self.networks
                    .update_items(data.networks, |n| n.name.clone());
            }
        }

        Ok(())
    }

    pub fn logs_scroll_up(&mut self) {
        let total_lines = self.container_logs.len() as u16;
        let max_scroll = total_lines.saturating_sub(self.log_visible_height);

        if self.log_auto_scroll {
            self.log_auto_scroll = false;
            self.log_vertical_scroll = max_scroll.saturating_sub(1);
        } else {
            self.log_vertical_scroll = self.log_vertical_scroll.saturating_sub(1);
        }
    }

    pub fn logs_scroll_logs_down(&mut self) {
        let total_lines = self.container_logs.len() as u16;
        let max_scroll = total_lines.saturating_sub(self.log_visible_height);

        self.log_vertical_scroll = self.log_vertical_scroll.saturating_add(1);

        if self.log_vertical_scroll >= max_scroll {
            self.log_auto_scroll = true;
        }
    }

    pub fn vertical_logs_scroll_to_top(&mut self) {
        self.log_auto_scroll = false;
        self.log_vertical_scroll = 0;
    }

    pub fn vertical_logs_scroll_bottom(&mut self) {
        self.log_auto_scroll = true;
        let total_lines = self.container_logs.len() as u16;
        self.log_vertical_scroll = total_lines.saturating_sub(self.log_visible_height);
    }

    fn max_horizontal_scroll(&self) -> u16 {
        let max_line_width = self
            .container_logs
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as u16;

        max_line_width.saturating_sub(self.log_visible_width)
    }

    pub fn horizontal_logs_scroll_left(&mut self) {
        self.log_horizontal_scroll = self.log_horizontal_scroll.saturating_sub(4);
    }

    pub fn horizontal_logs_scroll_right(&mut self) {
        let max_scroll = self.max_horizontal_scroll();
        self.log_horizontal_scroll = (self.log_horizontal_scroll + 4).min(max_scroll);
    }

    pub fn horizontal_logs_scroll_to_start(&mut self) {
        self.log_horizontal_scroll = 0;
    }

    pub fn horizontal_logs_scroll_to_end(&mut self) {
        self.log_horizontal_scroll = self.max_horizontal_scroll();
    }

    pub fn start_logs_stream(&mut self) {
        self.stop_logs_stream();
        self.container_logs.clear();
        self.log_horizontal_scroll = 0;

        if let Some(container) = self.containers.selected() {
            let (rx, handle) = get_container_logs(self.client.clone(), container.id.clone(), 100);
            self.log_rx = Some(rx);
            self.log_abort_handle = Some(handle);
        }
    }

    pub fn stop_logs_stream(&mut self) {
        if let Some(handle) = self.log_abort_handle.take() {
            handle.abort();
        }
        self.log_rx = None;
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::ItemsList => Focus::Content,
            Focus::Content => Focus::ItemsList,
        }
    }

    pub async fn menu_data_loaded(&mut self) {
        self.load_selected().await
    }

    pub async fn footer_action(&mut self, action: char) {
        if let Some(MenuItem::Containers) = self.menu.selected()
            && let Some(container) = self.containers.selected()
        {
            match action {
                's' => stop_container(&self.client, &container.id).await,
                'a' => start_container(&self.client, &container.id).await,
                _ => {}
            }
        }
    }

    pub async fn move_items_list_up(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => {
                self.images.move_up();
                self.load_selected_details().await;
            }
            Some(MenuItem::Containers) => {
                self.containers.move_up();
                match self.container_tab {
                    ContainerTab::Logs => self.start_logs_stream(),
                    ContainerTab::Details => self.load_selected_details().await,
                }
            }
            Some(MenuItem::Networks) => {
                self.networks.move_up();
                self.load_selected_details().await;
            }
            Some(MenuItem::Volumes) => {
                self.volumes.move_up();
                self.load_selected_details().await;
            }
            _ => {}
        }
    }

    pub async fn move_items_list_down(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => {
                self.images.move_down();
                self.load_selected_details().await;
            }
            Some(MenuItem::Containers) => {
                self.containers.move_down();
                match self.container_tab {
                    ContainerTab::Logs => self.start_logs_stream(),
                    ContainerTab::Details => self.load_selected_details().await,
                }
            }
            Some(MenuItem::Networks) => {
                self.networks.move_down();
                self.load_selected_details().await;
            }
            Some(MenuItem::Volumes) => {
                self.volumes.move_down();
                self.load_selected_details().await;
            }
            _ => {}
        }
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
                self.stop_logs_stream();
                if self.images.items.is_empty() {
                    let images = list_images(&self.client).await.unwrap_or_default();
                    self.images.update_items(images, |i| i.id.clone());
                }
                self.images.select_first();
                self.load_selected_details().await;
            }

            Some(MenuItem::Containers) => {
                self.stop_logs_stream();
                if self.containers.items.is_empty() {
                    let containers = list_containers(&self.client).await.unwrap_or_default();
                    self.containers.update_items(containers, |c| c.id.clone());
                }
                self.container_tab = ContainerTab::Details;
                self.containers.select_first();
                self.load_selected_details().await;
            }

            Some(MenuItem::Volumes) => {
                self.stop_logs_stream();
                if self.volumes.items.is_empty() {
                    let volumes = list_volumes(&self.client).await.unwrap_or_default();
                    self.volumes.update_items(volumes, |v| v.name.clone());
                }
                self.volumes.select_first();
                self.load_selected_details().await;
            }

            Some(MenuItem::Networks) => {
                self.stop_logs_stream();
                if self.networks.items.is_empty() {
                    let networks = list_networks(&self.client).await.unwrap_or_default();
                    self.networks.update_items(networks, |n| n.name.clone());
                }
                self.networks.select_first();
                self.load_selected_details().await;
            }

            None => {}
        }
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

            Some(MenuItem::Volumes) => {
                let Some(volume) = self.volumes.selected().cloned() else {
                    self.details = None;
                    return;
                };

                if let Ok(details) = inspect_volume(&self.client, &volume).await {
                    self.details = Some(Details::Volume(details))
                }
            }

            Some(MenuItem::Networks) => {
                let Some(network) = self.networks.selected().cloned() else {
                    self.details = None;
                    return;
                };

                if let Ok(details) = inspect_network(&self.client, &network).await {
                    self.details = Some(Details::Network(details))
                }
            }

            None => {}
        };
    }

    pub async fn toggle_container_tab(&mut self, tab: char) {
        if let Some(MenuItem::Containers) = self.menu.selected()
            && (self.focus == Focus::Content)
        {
            self.container_tab = match tab {
                'd' if self.container_tab != ContainerTab::Details => {
                    self.stop_logs_stream();
                    ContainerTab::Details
                }

                'l' if self.container_tab != ContainerTab::Logs => {
                    self.start_logs_stream();
                    ContainerTab::Logs
                }

                _ => return,
            };
        }
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    pub fn spawn_docker_poller(client: Docker) -> UnboundedReceiver<DockerPollData> {
        let (tx, rx) = unbounded_channel();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;

                // Fetch in parallel or sequentially in background without touching UI thread
                let (containers, images, volumes, networks) = tokio::join!(
                    list_containers(&client),
                    list_images(&client),
                    list_volumes(&client),
                    list_networks(&client),
                );

                let data = DockerPollData {
                    containers: containers.unwrap_or_default(),
                    images: images.unwrap_or_default(),
                    volumes: volumes.unwrap_or_default(),
                    networks: networks.unwrap_or_default(),
                };

                if tx.send(data).is_err() {
                    break;
                }
            }
        });

        rx
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
            container_tab: ContainerTab::Details,
            container_logs: vec![String::default()],
            log_rx: None,
            log_abort_handle: None,
            log_vertical_scroll: 0,
            log_horizontal_scroll: 0,
            log_auto_scroll: true,
            log_visible_height: 20,
            log_visible_width: 5,
            poll_rx: None,
        }
    }
}
