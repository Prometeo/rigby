use crate::{
    container::models::DockerContainer,
    tui::app::{App, ContainerTab, Focus, MenuItem},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs},
};
use std::rc::Rc;

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

    match app.menu.selected() {
        Some(MenuItem::Containers) => {
            render_list(
                &app.containers.items,
                items_area,
                &mut app.containers.state,
                frame,
                "Containers",
                app.focus == Focus::ItemsList,
                render_container_item,
            );
        }
        Some(MenuItem::Images) => {
            render_list(
                &app.images.items,
                items_area,
                &mut app.images.state,
                frame,
                "Images",
                app.focus == Focus::ItemsList,
                |item| ListItem::new(item.to_string()),
            );
        }
        Some(MenuItem::Networks) => {
            render_list(
                &app.networks.items,
                items_area,
                &mut app.networks.state,
                frame,
                "Networks",
                app.focus == Focus::ItemsList,
                |item| ListItem::new(item.to_string()),
            );
        }
        Some(MenuItem::Volumes) => {
            render_list(
                &app.volumes.items,
                items_area,
                &mut app.volumes.state,
                frame,
                "Volumes",
                app.focus == Focus::ItemsList,
                |item| ListItem::new(item.to_string()),
            );
        }
        None => {}
    }

    // ----------------- Right Details Pane -----------------
    render_details(app, panels[1], frame);
}

fn render_menu_tabs(app: &App, area: Rect, frame: &mut Frame) {
    let titles: Vec<Line> = app
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

    let selected_index = app.menu.state.selected().unwrap_or(0);

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

fn render_details(app: &App, area: Rect, frame: &mut Frame) {
    // Split the right area vertically: tabs at the top, content below
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    render_detail_tabs(app, frame, &rows);

    let (content, title): (Text, &str) = match app.container_tab {
        ContainerTab::Details => {
            let details = app
                .details
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "Nothing Selected".into());

            (Text::from(details), "Details")
        }
        ContainerTab::Logs => {
            if app.container_logs.is_empty() {
                (Text::from("No logs available"), "Logs")
            } else {
                let lines: Vec<Line> = app
                    .container_logs
                    .iter()
                    .map(|log| Line::raw(log.clone()))
                    .collect();

                (Text::from(lines), "Logs")
            }
        }
    };

    let paragraph = Paragraph::new(content).block(Block::bordered().title(title));
    frame.render_widget(paragraph, rows[1]);
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

fn render_detail_tabs(app: &App, frame: &mut Frame, rows: &Rc<[Rect]>) {
    let mut tab_titles = vec![Line::from(" Details ")];
    if matches!(app.menu.selected(), Some(MenuItem::Containers)) {
        tab_titles.push(Line::from(" Logs "));
    };

    let selected_tab = match app.container_tab {
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

    frame.render_widget(tabs, rows[0]);
}
