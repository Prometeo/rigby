use crate::{
    container::models::DockerContainer,
    tui::app::{App, ContainerTab, Focus, MenuItem},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Tabs,
    },
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
                get_list_footer('c'),
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
                get_list_footer('i'),
            );
            app.container_tab = ContainerTab::Details;
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
                get_list_footer('n'),
            );
            app.container_tab = ContainerTab::Details;
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
                get_list_footer('v'),
            );
            app.container_tab = ContainerTab::Details;
        }
        None => {}
    }

    // ----------------- Right Details Pane -----------------
    render_content(app, panels[1], frame, app.focus == Focus::Content);
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
    footer: Line,
) where
    F: Fn(&T) -> ListItem<'static>,
{
    let items = items.iter().map(item_renderer);
    let block = if focused {
        Block::bordered()
            .title(title)
            .title_bottom(footer)
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

fn render_content(app: &mut App, area: Rect, frame: &mut Frame, focused: bool) {
    // Split the right area vertically: tabs at the top, content below
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    render_detail_tabs(app, frame, &rows);

    let content_area = rows[1];

    let make_block = |title: &str| {
        let title_owned = title.to_string();
        if focused {
            Block::bordered()
                .title(title_owned)
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green))
        } else {
            Block::bordered().title(title_owned).borders(Borders::ALL)
        }
    };

    match app.container_tab {
        ContainerTab::Details => {
            let details = app
                .details
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "Nothing Selected".into());

            let paragraph = Paragraph::new(Text::from(details)).block(make_block(" Details "));

            frame.render_widget(paragraph, content_area);
        }

        ContainerTab::Logs => render_logs(app, make_block, frame, content_area),
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

fn render_logs<F>(app: &mut App, make_block: F, frame: &mut Frame, area: Rect)
where
    F: Fn(&str) -> Block<'static>,
{
    if app.container_logs.is_empty() {
        let paragraph = Paragraph::new(Text::from("No logs available")).block(make_block(" Logs "));
        frame.render_widget(paragraph, area);
        return;
    }

    let visible_height = area.height.saturating_sub(2);
    let visible_width = area.width.saturating_sub(2);
    app.log_visible_height = visible_height;
    app.log_visible_width = visible_width;

    // Calculate Vertical Bounds
    let total_lines = app.container_logs.len();
    let max_vertical_scroll = total_lines.saturating_sub(visible_height as usize);

    let vertical_scroll = if app.log_auto_scroll {
        max_vertical_scroll as u16
    } else {
        app.log_vertical_scroll.min(max_vertical_scroll as u16)
    };

    // Calculate Horizontal Bounds
    let max_line_width = app
        .container_logs
        .iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(0);
    let max_horizontal_scroll = max_line_width.saturating_sub(visible_width as usize) as u16;
    let horizontal_scroll = app.log_horizontal_scroll.min(max_horizontal_scroll);

    // Prepare All Styled Lines
    let lines: Vec<Line> = app
        .container_logs
        .iter()
        .map(|log| {
            if log.starts_with("[ERR]") {
                Line::from(Span::styled(log.as_str(), Style::default().fg(Color::Red)))
            } else {
                Line::from(log.as_str())
            }
        })
        .collect();

    let title = if app.log_auto_scroll {
        " Logs [FOLLOWING - Press Up to pause] "
    } else {
        " Logs [PAUSED - Press 'G' to resume follow] "
    };

    // Render Paragraph with 2D Scroll
    let paragraph = Paragraph::new(Text::from(lines))
        .block(make_block(title))
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
        }
        _ => {}
    }
    footer
}
