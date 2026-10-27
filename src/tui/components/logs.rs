use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::AbortHandle;

pub struct LogViewer {
    pub lines: Vec<String>,
    pub rx: Option<UnboundedReceiver<String>>,
    pub exec_rx: Option<UnboundedReceiver<String>>,
    pub abort_handle: Option<AbortHandle>,
    pub vertical_scroll: u16,
    pub horizontal_scroll: u16,
    pub auto_scroll: bool,
    pub visible_height: u16,
    pub visible_width: u16,
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::default()],
            rx: None,
            exec_rx: None,
            abort_handle: None,
            vertical_scroll: 0,
            horizontal_scroll: 0,
            auto_scroll: true,
            visible_height: 20,
            visible_width: 20,
        }
    }

    /// Read channels and truncate old logs
    pub fn process_ticks(&mut self) {
        const MAX_LOG_LINES: usize = 5000;

        // Standard container logs
        if let Some(rx) = self.rx.as_mut() {
            while let Ok(line) = rx.try_recv() {
                self.lines.push(line);
            }
        }

        // Execution logs
        // when executing docker exec docker does not parse it line by line It just dumps raw chunks of standard output (stdout)
        // If a command outputs 50 lines very quickly, Docker might send all 50 lines in a single chunk
        // .lines() split that giant chunk at every \n
        if let Some(rx) = self.exec_rx.as_mut() {
            while let Ok(line) = rx.try_recv() {
                for sub_line in line.lines() {
                    self.lines.push(sub_line.to_string());
                }
            }
        }

        // Tuncate
        if self.lines.len() > MAX_LOG_LINES {
            let excess = self.lines.len() - MAX_LOG_LINES;
            self.lines.drain(0..excess);
            self.vertical_scroll = self.vertical_scroll.saturating_sub(excess as u16);
        }
    }

    pub fn scroll_up(&mut self) {
        let total_lines: u16 = self.lines.len() as u16;
        let max_scroll: u16 = total_lines.saturating_sub(self.visible_height);

        if self.auto_scroll {
            self.auto_scroll = false;
            self.vertical_scroll = max_scroll.saturating_sub(1);
        } else {
            self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
        }
    }

    pub fn scroll_down(&mut self) {
        let total_lines: u16 = self.lines.len() as u16;
        let max_scroll: u16 = total_lines.saturating_sub(self.visible_height);

        self.vertical_scroll = self.vertical_scroll.saturating_add(1);

        if self.vertical_scroll >= max_scroll {
            self.auto_scroll = true;
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.auto_scroll = false;
        self.vertical_scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.auto_scroll = true;
        let total_lines: u16 = self.lines.len() as u16;
        self.vertical_scroll = total_lines.saturating_sub(self.visible_height);
    }

    fn max_horizontal_scroll(&self) -> u16 {
        let max_line_width: u16 = self
            .lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as u16;
        max_line_width.saturating_sub(self.visible_width)
    }

    pub fn scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(4);
    }

    pub fn scroll_right(&mut self) {
        let max_scroll = self.max_horizontal_scroll();
        self.horizontal_scroll = (self.horizontal_scroll + 4).min(max_scroll);
    }

    pub fn scroll_to_start(&mut self) {
        self.horizontal_scroll = 0;
    }

    pub fn scroll_to_end(&mut self) {
        self.horizontal_scroll = self.max_horizontal_scroll();
    }

    pub fn stop_stream(&mut self) {
        if let Some(handle) = self.abort_handle.take() {
            handle.abort();
        }
        self.rx = None;
    }
}
