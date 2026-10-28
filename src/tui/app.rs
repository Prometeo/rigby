use crate::utils::driver_connector;
use crate::volume::{
    models::{DockerVolume, DockerVolumeDetail},
    service::list_volumes,
};
use crate::{
    container::service::remove_container,
    image::{
        models::{DockerImage, DockerImageDetail},
        service::{inspect_image, list_images},
    },
};
use crate::{
    container::service::restart_container,
    networking::{
        models::{DockerNetwork, DockerNetworkDetail},
        service::{inspect_network, list_networks},
    },
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
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::models::EventMessageTypeEnum;
use color_eyre::Result;
use futures_util::StreamExt;
use std::fmt;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use crate::tui::components::StatefullList;
use crate::tui::components::logs::LogViewer;
use crate::tui::components::resources::DockerResources;

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

pub enum DockerEventUpdate {
    Containers(Vec<DockerContainer>),
    Images(Vec<DockerImage>),
    Volumes(Vec<DockerVolume>),
    Networks(Vec<DockerNetwork>),
}

#[derive(Default, PartialEq, Eq, Clone)]
pub enum ContainerTab {
    #[default]
    Details,
    Logs,
}

#[derive(Default, PartialEq, Eq, Clone)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

pub struct App {
    pub quit: bool,
    pub client: Docker,
    pub focus: Focus,

    // UI State
    pub menu: StatefullList<MenuItem>,
    pub container_tab: ContainerTab,
    pub details: Option<Details>,
    pub input_mode: InputMode,
    pub command_input: String,

    // Resources
    pub resources: DockerResources,

    // Isolated Components
    pub logs: LogViewer,

    pub event_rx: Option<UnboundedReceiver<DockerEventUpdate>>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn tick(&mut self) -> Result<()> {
        self.logs.process_ticks();

        let mut pending_updates = Vec::new();
        if let Some(rx) = self.event_rx.as_mut() {
            while let Ok(update) = rx.try_recv() {
                pending_updates.push(update);
            }
        }

        let mut reload_details = false;
        for update in pending_updates {
            match update {
                DockerEventUpdate::Containers(conts) => {
                    self.resources
                        .containers
                        .update_items(conts, |c| c.id.clone());
                    if let Some(MenuItem::Containers) = self.menu.selected()
                        && self.container_tab == ContainerTab::Details
                    {
                        reload_details = true;
                    }
                }
                DockerEventUpdate::Images(imgs) => {
                    self.resources.images.update_items(imgs, |i| i.id.clone());
                    if let Some(MenuItem::Images) = self.menu.selected() {
                        reload_details = true;
                    }
                }
                DockerEventUpdate::Volumes(vols) => {
                    self.resources
                        .volumes
                        .update_items(vols, |v| v.name.clone());
                    if let Some(MenuItem::Volumes) = self.menu.selected() {
                        reload_details = true;
                    }
                }
                DockerEventUpdate::Networks(nets) => {
                    self.resources
                        .networks
                        .update_items(nets, |n| n.name.clone());
                    if let Some(MenuItem::Networks) = self.menu.selected() {
                        reload_details = true;
                    }
                }
            }
        }

        if reload_details {
            self.load_selected_details().await;
        }
        Ok(())
    }

    pub fn start_logs_stream(&mut self) {
        self.logs.stop_stream();
        self.logs.lines.clear();
        self.logs.horizontal_scroll = 0;

        if let Some(container) = self.resources.containers.selected() {
            let (rx, handle) = get_container_logs(self.client.clone(), container.id.clone(), 100);
            self.logs.rx = Some(rx);
            self.logs.abort_handle = Some(handle);
        }
    }

    pub fn stop_logs_stream(&mut self) {
        self.logs.stop_stream();
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
            && let Some(container) = self.resources.containers.selected()
        {
            match action {
                's' => stop_container(&self.client, &container.id).await,
                'a' => start_container(&self.client, &container.id).await,
                'r' => restart_container(&self.client, &container.id).await,
                'k' => remove_container(&self.client, &container.id).await,
                'e' => self.input_mode = InputMode::Editing,
                _ => {}
            }
        }
    }

    pub async fn move_items_list_up(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => {
                self.resources.images.move_up();
                self.load_selected_details().await;
            }
            Some(MenuItem::Containers) => {
                self.resources.containers.move_up();
                match self.container_tab {
                    ContainerTab::Logs => self.start_logs_stream(),
                    ContainerTab::Details => self.load_selected_details().await,
                }
            }
            Some(MenuItem::Networks) => {
                self.resources.networks.move_up();
                self.load_selected_details().await;
            }
            Some(MenuItem::Volumes) => {
                self.resources.volumes.move_up();
                self.load_selected_details().await;
            }
            _ => {}
        }
    }

    pub async fn move_items_list_down(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => {
                self.resources.images.move_down();
                self.load_selected_details().await;
            }
            Some(MenuItem::Containers) => {
                self.resources.containers.move_down();
                match self.container_tab {
                    ContainerTab::Logs => self.start_logs_stream(),
                    ContainerTab::Details => self.load_selected_details().await,
                }
            }
            Some(MenuItem::Networks) => {
                self.resources.networks.move_down();
                self.load_selected_details().await;
            }
            Some(MenuItem::Volumes) => {
                self.resources.volumes.move_down();
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
                if self.resources.images.items.is_empty() {
                    let images = list_images(&self.client).await.unwrap_or_default();
                    self.resources.images.update_items(images, |i| i.id.clone());
                }
                self.resources.images.select_first();
                self.load_selected_details().await;
            }
            Some(MenuItem::Containers) => {
                self.stop_logs_stream();
                if self.resources.containers.items.is_empty() {
                    let containers = list_containers(&self.client).await.unwrap_or_default();
                    self.resources
                        .containers
                        .update_items(containers, |c| c.id.clone());
                }
                self.container_tab = ContainerTab::Details;
                self.resources.containers.select_first();
                self.load_selected_details().await;
            }
            Some(MenuItem::Volumes) => {
                self.stop_logs_stream();
                if self.resources.volumes.items.is_empty() {
                    let volumes = list_volumes(&self.client).await.unwrap_or_default();
                    self.resources
                        .volumes
                        .update_items(volumes, |v| v.name.clone());
                }
                self.resources.volumes.select_first();
                self.load_selected_details().await;
            }
            Some(MenuItem::Networks) => {
                self.stop_logs_stream();
                if self.resources.networks.items.is_empty() {
                    let networks = list_networks(&self.client).await.unwrap_or_default();
                    self.resources
                        .networks
                        .update_items(networks, |n| n.name.clone());
                }
                self.resources.networks.select_first();
                self.load_selected_details().await;
            }
            None => {}
        }
    }

    pub async fn load_selected_details(&mut self) {
        match self.menu.selected() {
            Some(MenuItem::Images) => {
                let Some(image) = self.resources.images.selected().cloned() else {
                    self.details = None;
                    return;
                };
                if let Ok(details) = inspect_image(&self.client, &image).await {
                    self.details = Some(Details::Image(details))
                };
            }
            Some(MenuItem::Containers) => {
                let Some(container) = self.resources.containers.selected().cloned() else {
                    self.details = None;
                    return;
                };
                if let Ok(details) = inspect_container(&self.client, &container).await {
                    self.details = Some(Details::Container(details))
                }
            }
            Some(MenuItem::Volumes) => {
                let Some(volume) = self.resources.volumes.selected().cloned() else {
                    self.details = None;
                    return;
                };
                if let Ok(details) = inspect_volume(&self.client, &volume).await {
                    self.details = Some(Details::Volume(details))
                }
            }
            Some(MenuItem::Networks) => {
                let Some(network) = self.resources.networks.selected().cloned() else {
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

    pub fn spawn_docker_event_listener(client: Docker) -> UnboundedReceiver<DockerEventUpdate> {
        let (tx, rx) = unbounded_channel();
        tokio::spawn(async move {
            let mut event_stream = client.events(None);
            while let Some(event_result) = event_stream.next().await {
                let Ok(event) = event_result else { continue };
                match event.typ {
                    Some(EventMessageTypeEnum::CONTAINER) => {
                        if let Ok(containers) = list_containers(&client).await {
                            let _ = tx.send(DockerEventUpdate::Containers(containers));
                        }
                    }
                    Some(EventMessageTypeEnum::VOLUME) => {
                        if let Ok(volumes) = list_volumes(&client).await {
                            let _ = tx.send(DockerEventUpdate::Volumes(volumes));
                        }
                    }
                    Some(EventMessageTypeEnum::NETWORK) => {
                        if let Ok(networks) = list_networks(&client).await {
                            let _ = tx.send(DockerEventUpdate::Networks(networks));
                        }
                    }
                    Some(EventMessageTypeEnum::IMAGE) => {
                        if let Ok(images) = list_images(&client).await {
                            let _ = tx.send(DockerEventUpdate::Images(images));
                        }
                    }
                    _ => {}
                }
            }
        });
        rx
    }

    pub async fn execute_container_command(&mut self) {
        if self.command_input.is_empty() {
            self.input_mode = InputMode::Normal;
            return;
        }

        if let Some(MenuItem::Containers) = self.menu.selected()
            && let Some(container) = self.resources.containers.selected()
        {
            let cmd: Vec<String> = self
                .command_input
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            let command_string = self.command_input.clone();

            let exec_options = CreateExecOptions {
                cmd: Some(cmd),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };
            let client = self.client.clone();
            let container_id = container.id.clone();

            let (tx, rx) = unbounded_channel();
            self.logs.exec_rx = Some(rx);
            self.logs
                .lines
                .push(format!(">>> Executing: {}", command_string));

            tokio::spawn(async move {
                if let Ok(exec) = client.create_exec(&container_id, exec_options).await
                    && let Ok(StartExecResults::Attached { mut output, .. }) =
                        client.start_exec(&exec.id, None).await
                {
                    while let Some(Ok(msg)) = output.next().await {
                        let _ = tx.send(msg.to_string());
                    }
                    let _ = tx.send(format!("<<< Finished: {}", command_string));
                }
            });
        }
        self.command_input.clear();
        self.input_mode = InputMode::Normal;
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
        let event_rx = Some(Self::spawn_docker_event_listener(client.clone()));

        Self {
            quit: false,
            menu,
            focus: Focus::ItemsList,
            resources: DockerResources::new(),
            client,
            details: None,
            container_tab: ContainerTab::Details,
            logs: LogViewer::new(),
            event_rx,
            input_mode: InputMode::Normal,
            command_input: String::new(),
        }
    }
}
