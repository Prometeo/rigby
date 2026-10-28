use ratatui::widgets::ListState;

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

        if self.items.is_empty() {
            self.state.select(None);
            return;
        }

        if let Some(target_key) = selected_key
            && let Some(new_pos) = self
                .items
                .iter()
                .position(|item| key_fn(item) == target_key)
        {
            self.state.select(Some(new_pos));
            return;
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
