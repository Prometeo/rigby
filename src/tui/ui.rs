use crate::{
    container::models::DockerContainer,
    tui::app::{App, Focus, MenuItem},
};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, HorizontalAlignment, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(frame.area());

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(outer_layout[0]);

    // Left main menu
    render_list(
        &app.menu.items,
        outer_layout[0],
        &mut app.menu.state,
        frame,
        "Menu",
        app.focus == Focus::Menu,
        |item| ListItem::new(item.to_string()),
    );

    // Render details pane
    render_details(frame, outer_layout[1]);

    match app.menu.selected() {
        Some(MenuItem::Containers) => {
            render_list(
                &app.containers.items,
                inner_layout[1],
                &mut app.containers.state,
                frame,
                "Containers",
                app.focus == Focus::Content,
                render_container_item,
            );
        }
        Some(MenuItem::Images) => {
            render_list(
                &app.images.items,
                inner_layout[1],
                &mut app.images.state,
                frame,
                "Images",
                app.focus == Focus::Content,
                |item| ListItem::new(item.to_string()),
            );
        }
        Some(MenuItem::Networks) => {
            render_list(
                &app.networks.items,
                inner_layout[1],
                &mut app.networks.state,
                frame,
                "Networks",
                app.focus == Focus::Content,
                |item| ListItem::new(item.to_string()),
            );
        }
        Some(MenuItem::Volumes) => {
            render_list(
                &app.volumes.items,
                inner_layout[1],
                &mut app.volumes.state,
                frame,
                "Volumes",
                app.focus == Focus::Content,
                |item| ListItem::new(item.to_string()),
            );
        }
        None => {}
    }
}

fn render_list<T, F>(
    items: &[T],
    area: Rect,
    state: &mut ListState,
    frame: &mut Frame,
    title: &str,
    focused: bool,
    item_renderer: F,
) where
    F: Fn(&T) -> ListItem<'static>,
{
    let items = items.iter().map(item_renderer);

    let block = if focused {
        Block::bordered()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::Green))
    } else {
        Block::bordered().title(title).borders(Borders::ALL)
    };

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

fn render_details(frame: &mut Frame, area: Rect) {
    let text: &str = "Content Detail";
    let paragraph = Paragraph::new(text)
        .block(Block::bordered().title("Details"))
        .style(Color::Blue)
        .alignment(HorizontalAlignment::Center);
    frame.render_widget(paragraph, area);
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
