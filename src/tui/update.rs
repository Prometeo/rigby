use crate::tui::app::{App, Focus, MenuItem};
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

        KeyCode::Up | KeyCode::Char('k') => match app.focus {
            Focus::Menu => {
                let previous = app.menu.state.selected();

                app.menu.move_up();

                if previous != app.menu.state.selected() {
                    let i = app.menu_data_loaded().await?;
                }
            }
            Focus::Content => {}
        },

        KeyCode::Down | KeyCode::Char('j') => match app.focus {
            Focus::Menu => {
                let previous = app.menu.state.selected();

                app.menu.move_down();

                if previous != app.menu.state.selected() {
                    app.menu_data_loaded().await?;
                }
            }
            Focus::Content => {}
        },

        KeyCode::Tab => {
            app.toggle_focus();
        }
        _ => {}
    }

    Ok(())
}
