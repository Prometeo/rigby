use crate::tui::app::{App, Focus, MenuItem};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
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
        .highlight_style(Style::default().reversed())
        .bold()
        .fg(Color::Blue);

    frame.render_stateful_widget(list, area, state);
}
