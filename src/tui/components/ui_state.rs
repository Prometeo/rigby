use crate::container::models::DockerContainerDetail;
use crate::image::models::DockerImageDetail;
use crate::networking::models::DockerNetworkDetail;
use crate::tui::components::StatefullList;
use crate::volume::models::DockerVolumeDetail;
use std::fmt;

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

#[derive(Default, PartialEq, Eq, Clone)]
pub enum ContainerTab {
    #[default]
    Details,
    Logs,
}

pub struct UiState {
    pub focus: Focus,
    pub menu: StatefullList<MenuItem>,
    pub container_tab: ContainerTab,
    pub details: Option<Details>,
}

impl UiState {
    pub fn new() -> Self {
        let menu = StatefullList {
            items: vec![
                MenuItem::Images,
                MenuItem::Containers,
                MenuItem::Volumes,
                MenuItem::Networks,
            ],
            ..Default::default()
        };

        Self {
            focus: Focus::ItemsList,
            menu,
            container_tab: ContainerTab::Details,
            details: None,
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::ItemsList => Focus::Content,
            Focus::Content => Focus::ItemsList,
        }
    }
}
