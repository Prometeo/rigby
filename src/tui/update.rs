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

        KeyCode::Up | KeyCode::Char('k') => match app.focus {
            Focus::ItemsList => {
                app.move_items_list_up().await;
            }
            Focus::Content => {
                app.logs_scroll_up();
            }
        },

        KeyCode::Down | KeyCode::Char('j') => match app.focus {
            Focus::ItemsList => {
                app.move_items_list_down().await;
            }
            Focus::Content => {
                app.logs_scroll_logs_down();
            }
        },

        KeyCode::Char('g') => match app.focus {
            Focus::Content => {
                app.logs_scroll_to_top();
            }
            Focus::ItemsList => {}
        },

        KeyCode::Char('G') => match app.focus {
            Focus::Content => {
                app.logs_scroll_to_bottom();
            }
            Focus::ItemsList => {}
        },

        KeyCode::Left | KeyCode::Right => {
            app.toggle_container_tab().await;
        }

        KeyCode::Tab => {
            app.toggle_focus();
        }
        _ => {}
    }

    Ok(())
}
