use crate::tui::app::App;
use crate::tui::components::{ConfirmButton, Focus};
use color_eyre::{Report, Result};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn update(app: &mut App, key_event: KeyEvent) -> Result<(), Report> {
    if app.command_popup.is_editing() {
        exec_command(app, key_event).await?;
        return Ok(());
    }

    if app.confirm_popup.is_open() {
        confirmation_popup(app, key_event).await?;
        return Ok(());
    }

    // Handle general UI navigation
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

        KeyCode::Tab => {
            app.load_selected_details().await;
            app.ui.toggle_focus();
        }

        _ => dynamic_key_mapping(app, key_event).await?,
    }

    Ok(())
}

async fn dynamic_key_mapping(app: &mut App, key_event: KeyEvent) -> Result<(), Report> {
    match key_event.code {
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
        _ => {}
    }

    Ok(())
}

async fn exec_command(app: &mut App, key_event: KeyEvent) -> Result<(), Report> {
    match key_event.code {
        KeyCode::Enter => {
            app.execute_container_command().await;
        }
        KeyCode::Char(c) => {
            app.command_popup.text.push(c);
        }
        KeyCode::Backspace => {
            app.command_popup.text.pop();
        }
        KeyCode::Esc => {
            app.command_popup.close_and_clear();
        }
        _ => {}
    }

    Ok(())
}

async fn confirmation_popup(app: &mut App, key_event: KeyEvent) -> Result<(), Report> {
    match key_event.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_popup.toggle_selection();
        }
        KeyCode::Esc => {
            app.confirm_popup.close_and_clear();
        }
        KeyCode::Enter => {
            if app.confirm_popup.selected_button == ConfirmButton::Accept {
                app.execute_container_pending_action().await;
            } else {
                app.confirm_popup.close_and_clear();
            }
        }
        _ => {}
    }
    Ok(())
}
