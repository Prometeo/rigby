#[derive(Default, PartialEq, Eq, Clone)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
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
        self.mode == InputMode::Editing
    }

    pub fn open(&mut self) {
        self.mode = InputMode::Editing;
    }

    pub fn close_and_clear(&mut self) {
        self.mode = InputMode::Normal;
        self.text.clear();
    }
}
