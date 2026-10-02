use crate::tui::app::{App, Focus};
use color_eyre::{Report, Result};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn update(app: &mut App, key_event: KeyEvent) -> Result<(), Report> {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }

        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }

        KeyCode::Char(c @ ('1' | '2' | '3' | '4')) => {
            app.select_menu_item(c).await;
        }

        KeyCode::Up => match app.focus {
            Focus::ItemsList => {
                app.move_items_list_up().await;
            }
            Focus::Content => {
                app.logs_scroll_up();
            }
        },

        KeyCode::Down => match app.focus {
            Focus::ItemsList => {
                app.move_items_list_down().await;
            }
            Focus::Content => {
                app.logs_scroll_logs_down();
            }
        },

        KeyCode::Char('g') => match app.focus {
            Focus::Content => {
                app.vertical_logs_scroll_to_top();
            }
            Focus::ItemsList => {}
        },

        KeyCode::Char('G') => match app.focus {
            Focus::Content => {
                app.vertical_logs_scroll_bottom();
            }
            Focus::ItemsList => {}
        },

        KeyCode::Left if key_event.modifiers == KeyModifiers::CONTROL => {
            app.horizontal_logs_scroll_to_start();
        }

        KeyCode::Left => {
            app.horizontal_logs_scroll_left();
        }

        KeyCode::Right if key_event.modifiers == KeyModifiers::CONTROL => {
            app.horizontal_logs_scroll_to_end();
        }

        KeyCode::Right => {
            app.horizontal_logs_scroll_right();
        }

        KeyCode::Char(c @ ('d' | 'l')) => {
            app.toggle_container_tab(c).await;
        }

        KeyCode::Char('s') => app.footer_action().await,

        KeyCode::Tab => {
            app.toggle_focus();
        }
        _ => {}
    }

    Ok(())
}
