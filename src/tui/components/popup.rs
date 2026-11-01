#[derive(Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    PopUp,
}

pub struct CommandPopup {
    pub mode: InputMode,
    pub text: String,
}

impl CommandPopup {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Normal,
            text: String::new(),
        }
    }

    pub fn is_editing(&self) -> bool {
        self.mode == InputMode::PopUp
    }

    pub fn open(&mut self) {
        self.mode = InputMode::PopUp;
    }

    pub fn close_and_clear(&mut self) {
        self.mode = InputMode::Normal;
        self.text.clear();
    }
}

pub enum ConfirmAction {
    None,
    StopContainer(String),
    RestartContainer(String),
    RemoveContainer(String),
}

#[derive(PartialEq, Eq, Clone)]
pub enum ConfirmButton {
    Accept,
    Cancel,
}

pub struct ConfirmPopup {
    pub mode: InputMode,
    pub is_open: bool,
    pub title: String,
    pub message: String,
    pub selected_button: ConfirmButton,
    pub pending_action: ConfirmAction,
}

impl ConfirmPopup {
    pub fn new() -> Self {
        Self {
            is_open: false,
            title: String::new(),
            message: String::new(),
            selected_button: ConfirmButton::Cancel,
            mode: InputMode::Normal,
            pending_action: ConfirmAction::None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.mode == InputMode::PopUp
    }

    pub fn open(&mut self, title: &str, message: &str, action: ConfirmAction) {
        self.is_open = true;
        self.title = title.into();
        self.message = message.into();
        self.selected_button = ConfirmButton::Cancel;
        self.mode = InputMode::PopUp;
        self.pending_action = action;
    }

    pub fn close_and_clear(&mut self) {
        self.selected_button = ConfirmButton::Cancel;
        self.close();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.mode = InputMode::Normal;
        self.pending_action = ConfirmAction::None;
    }

    pub fn toggle_selection(&mut self) {
        self.selected_button = match self.selected_button {
            ConfirmButton::Accept => ConfirmButton::Cancel,
            ConfirmButton::Cancel => ConfirmButton::Accept,
        };
    }
}
