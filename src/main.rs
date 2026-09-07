mod container;
mod tui;
mod utils;
use crate::tui::{app::App, tui::Tui};
use color_eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use tui::event::{Event, EventHandler};
use tui::update::update;

#[tokio::main]
async fn main() -> Result<()> {
    // Create an application.
    let mut app: App = App::new();
    app.menu_data_loaded().await?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    while !app.quit {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            Event::Tick => app.tick().await?,
            Event::Key(key_event) => {
                update(&mut app, key_event).await?;
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };
    }

    tui.exit()?;
    Ok(())
}
