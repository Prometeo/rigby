use crate::{container::models::DockerContainer, tui::app::App};

use crate::tui::components::{ContainerTab, Focus, MenuItem};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Tabs,
    },
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(rows[1]);

    // ----------------- Top Menu (Tabs) -----------------
    render_menu_tabs(app, rows[0], frame);

    // ----------------- Middle Items List -----------------
    let items_area = panels[0];
    let list_focused = app.ui.focus == Focus::ItemsList;

    match app.ui.menu.selected() {
        Some(MenuItem::Containers) => {
            CustomList::with_renderer(
                &app.resources.containers.items,
                "Containers",
                render_container_item,
            )
            .focused(list_focused)
            .footer(get_list_footer('c'))
            .render(frame, items_area, &mut app.resources.containers.state);
        }
        Some(MenuItem::Images) => {
            CustomList::new(&app.resources.images.items, "Images")
                .focused(list_focused)
                .footer(get_list_footer('i'))
                .render(frame, items_area, &mut app.resources.images.state);
            app.ui.container_tab = ContainerTab::Details;
        }
        Some(MenuItem::Networks) => {
            CustomList::new(&app.resources.networks.items, "Networks")
                .focused(list_focused)
                .footer(get_list_footer('n'))
                .render(frame, items_area, &mut app.resources.networks.state);
            app.ui.container_tab = ContainerTab::Details;
        }
        Some(MenuItem::Volumes) => {
            CustomList::new(&app.resources.volumes.items, "Volumes")
                .focused(list_focused)
                .footer(get_list_footer('v'))
                .render(frame, items_area, &mut app.resources.volumes.state);
            app.ui.container_tab = ContainerTab::Details;
        }
        None => {}
    }

    // ----------------- Right Details Pane -----------------
    render_content(app, panels[1], frame, app.ui.focus == Focus::Content);

    // ----------------- Popup -----------------
    render_popup(app, frame);
}

fn render_popup(app: &App, frame: &mut Frame) {
    if !app.popup.is_editing() {
        return;
    }

    let area = centered_rect(30, 3, frame.area());
    frame.render_widget(Clear, area);

    let title = if let Some(container) = app.resources.containers.selected() {
        format!(" Exec in {} (Enter to run, Esc to cancel) ", container.name)
    } else {
        " Execute Command ".to_string()
    };

    let block = Block::bordered()
        .title(title)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(app.popup.text.as_str()).block(block);
    frame.render_widget(paragraph, area);

    frame.set_cursor_position(ratatui::layout::Position {
        x: area.x + app.popup.text.chars().count() as u16 + 1,
        y: area.y + 1,
    });
}

fn render_menu_tabs(app: &App, area: Rect, frame: &mut Frame) {
    let titles: Vec<Line> = app
        .ui
        .menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            Line::from(vec![
                Span::styled(
                    format!(" [{}] ", i + 1),
                    Style::default().fg(Color::LightGreen),
                ),
                Span::raw(item.to_string()),
                Span::raw(" "),
            ])
        })
        .collect();

    let selected_index = app.ui.menu.state.selected().unwrap_or(0);

    let block = Block::bordered()
        .title("Menu")
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Green));
    let tabs = Tabs::new(titles)
        .block(block)
        .select(selected_index)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    frame.render_widget(tabs, area);
}

pub struct CustomList<'a, T, F> {
    items: &'a [T],
    title: &'a str,
    item_renderer: F,
    focused: bool,
    footer: Option<Line<'a>>,
}

impl<'a, T: ToString> CustomList<'a, T, fn(&T) -> ListItem<'static>> {
    /// Creates a list using `ToString` as the default item renderer
    pub fn new(items: &'a [T], title: &'a str) -> Self {
        Self {
            items,
            title,
            item_renderer: |item| ListItem::new(item.to_string()),
            focused: false,
            footer: None,
        }
    }
}

impl<'a, T, F> CustomList<'a, T, F>
where
    F: Fn(&T) -> ListItem<'static>,
{
    /// Creates a list with a custom item-rendering function
    pub fn with_renderer(items: &'a [T], title: &'a str, item_renderer: F) -> Self {
        Self {
            items,
            title,
            item_renderer,
            focused: false,
            footer: None,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn footer(mut self, footer: Line<'a>) -> Self {
        self.footer = Some(footer);
        self
    }

    pub fn render(self, frame: &mut Frame, area: Rect, state: &mut ListState) {
        let items = self.items.iter().map(self.item_renderer);
        let mut block = styled_block(self.title, self.focused);
        if self.focused
            && let Some(footer) = self.footer
        {
            block = block.title_bottom(footer);
        }

        let list = List::new(items)
            .block(block)
            .highlight_symbol("> ")
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(45, 55, 72))
                    .add_modifier(Modifier::BOLD),
            )
            .fg(Color::Blue);

        frame.render_stateful_widget(list, area, state);
    }
}

/// Shared helper for consistent focused/unfocused borders across panels
fn styled_block(title: &str, focused: bool) -> Block<'_> {
    if focused {
        Block::bordered()
            .title(title)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::Green))
    } else {
        Block::bordered().title(title)
    }
}

fn render_content(app: &mut App, area: Rect, frame: &mut Frame, focused: bool) {
    // Split the right area vertically: tabs at the top, content below
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    render_detail_tabs(app, frame, rows[0]);

    let content_area = rows[1];

    match app.ui.container_tab {
        ContainerTab::Details => {
            let details = app
                .ui
                .details
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "Nothing Selected".into());

            let paragraph =
                Paragraph::new(Text::from(details)).block(styled_block(" Details ", focused));

            frame.render_widget(paragraph, content_area);
        }

        ContainerTab::Logs => render_logs(app, focused, frame, content_area),
    }
}

fn render_container_item(container: &DockerContainer) -> ListItem<'static> {
    let color = match container.state.as_str() {
        "running" => Color::Green,
        "paused" | "restarting" => Color::Yellow,
        "created" => Color::Blue,
        "removing" => Color::Magenta,
        "exited" => Color::Gray,
        "dead" => Color::Red,
        _ => Color::Reset,
    };

    let style = Style::default().fg(color);

    let line = Line::from(vec![Span::styled(container.name.clone(), style)]);
    ListItem::new(line)
}

fn render_detail_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let mut tab_titles = vec![Line::from(" Details ")];
    if matches!(app.ui.menu.selected(), Some(MenuItem::Containers)) {
        tab_titles.push(Line::from(" Logs "));
    };

    let selected_tab = match app.ui.container_tab {
        ContainerTab::Details => 0,
        ContainerTab::Logs => 1,
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::bordered().title("View"))
        .select(selected_tab)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    frame.render_widget(tabs, area);
}

fn render_logs(app: &mut App, focused: bool, frame: &mut Frame, area: Rect) {
    if app.logs.lines.is_empty() {
        let paragraph =
            Paragraph::new(Text::from("No logs available")).block(styled_block(" Logs ", focused));
        frame.render_widget(paragraph, area);
        return;
    }

    let visible_height = area.height.saturating_sub(2);
    let visible_width = area.width.saturating_sub(2);
    app.logs.visible_height = visible_height;
    app.logs.visible_width = visible_width;

    // Calculate Vertical Bounds
    let total_lines = app.logs.lines.len();
    let max_vertical_scroll = total_lines.saturating_sub(visible_height as usize);

    let vertical_scroll = if app.logs.auto_scroll {
        max_vertical_scroll as u16
    } else {
        app.logs.vertical_scroll.min(max_vertical_scroll as u16)
    };

    // Calculate Horizontal Bounds
    let max_line_width = app.logs.lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let max_horizontal_scroll = max_line_width.saturating_sub(visible_width as usize) as u16;
    let horizontal_scroll = app.logs.horizontal_scroll.min(max_horizontal_scroll);

    // Prepare All Styled Lines
    let lines: Vec<Line> = app
        .logs
        .lines
        .iter()
        .map(|log| {
            if log.starts_with("[ERR]") {
                Line::from(Span::styled(log.as_str(), Style::default().fg(Color::Red)))
            } else {
                Line::from(log.as_str())
            }
        })
        .collect();

    let title = if app.logs.auto_scroll {
        " Logs [FOLLOWING - Press Up to pause] "
    } else {
        " Logs [PAUSED - Press 'G' to resume follow] "
    };

    // Render Paragraph with 2D Scroll
    let paragraph = Paragraph::new(Text::from(lines))
        .block(styled_block(title, focused))
        .scroll((vertical_scroll, horizontal_scroll));

    frame.render_widget(paragraph, area);

    // Vertical Scrollbar (Right)
    let mut v_scrollbar_state =
        ScrollbarState::new(max_vertical_scroll).position(vertical_scroll as usize);
    frame.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area,
        &mut v_scrollbar_state,
    );

    // Horizontal Scrollbar (Bottom)
    if max_horizontal_scroll > 0 {
        let mut h_scrollbar_state = ScrollbarState::new(max_horizontal_scroll as usize)
            .position(horizontal_scroll as usize);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(Some("←"))
                .end_symbol(Some("→")),
            area,
            &mut h_scrollbar_state,
        );
    }
}

fn get_list_footer(selected_item: char) -> Line<'static> {
    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::Gray);

    let mut footer = Line::from(vec![Span::raw(" ")]);

    #[allow(clippy::single_match)]
    match selected_item {
        'c' => {
            footer.push_span(Span::styled("s", key_style));
            footer.push_span(Span::styled(": stop  ", desc_style));
            footer.push_span(Span::styled("a", key_style));
            footer.push_span(Span::styled(": start  ", desc_style));
            footer.push_span(Span::styled("r", key_style));
            footer.push_span(Span::styled(": restart  ", desc_style));
            footer.push_span(Span::styled("k", key_style));
            footer.push_span(Span::styled(": delete  ", desc_style));
            footer.push_span(Span::styled("e", key_style));
            footer.push_span(Span::styled(": exec  ", desc_style));
        }
        _ => {}
    }
    footer
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
