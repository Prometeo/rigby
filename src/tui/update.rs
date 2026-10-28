use crate::tui::app::App;
use crate::tui::components::Focus;
use color_eyre::{Report, Result};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn update(app: &mut App, key_event: KeyEvent) -> Result<(), Report> {
    // 1. Handle keystrokes for the text box if the popup is open
    if app.popup.is_editing() {
        match key_event.code {
            KeyCode::Enter => {
                app.execute_container_command().await;
            }
            KeyCode::Char(c) => {
                app.popup.text.push(c);
            }
            KeyCode::Backspace => {
                app.popup.text.pop();
            }
            KeyCode::Esc => {
                app.popup.close_and_clear();
            }
            _ => {}
        }

        return Ok(());
    }

    // Handle normal UI navigation if the popup is closed
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

        KeyCode::Up => match app.ui.focus {
            Focus::ItemsList => {
                app.move_items_list_up().await;
            }
            Focus::Content => {
                app.logs.scroll_up();
            }
        },

        KeyCode::Down => match app.ui.focus {
            Focus::ItemsList => {
                app.move_items_list_down().await;
            }
            Focus::Content => {
                app.logs.scroll_down();
            }
        },

        KeyCode::Char('g') => match app.ui.focus {
            Focus::Content => {
                app.logs.scroll_to_top();
            }
            Focus::ItemsList => {}
        },

        KeyCode::Char('G') => match app.ui.focus {
            Focus::Content => {
                app.logs.scroll_to_bottom();
            }
            Focus::ItemsList => {}
        },

        KeyCode::Left if key_event.modifiers == KeyModifiers::CONTROL => {
            app.logs.scroll_to_start();
        }

        KeyCode::Left => {
            app.logs.scroll_left();
        }

        KeyCode::Right if key_event.modifiers == KeyModifiers::CONTROL => {
            app.logs.scroll_to_end();
        }

        KeyCode::Right => {
            app.logs.scroll_right();
        }

        KeyCode::Char(c @ ('d' | 'l')) => {
            app.toggle_container_tab(c).await;
        }

        KeyCode::Char(c @ ('s' | 'a' | 'r' | 'k' | 'e')) => app.footer_action(c).await,

        KeyCode::Tab => {
            app.ui.toggle_focus();
        }
        _ => {}
    }

    Ok(())
}
