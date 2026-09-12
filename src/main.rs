mod container;
mod image;
mod networking;
mod tui;
mod utils;
mod volume;
use crate::tui::{
    Tui,
    app::{App, spawn_docker_poller},
};
use color_eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{BufWriter, stderr};
use tui::event::{Event, EventHandler};
use tui::update::update;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app: App = App::new();
    app.poll_rx = Some(spawn_docker_poller(app.client.clone()));
    app.menu_data_loaded().await;

    let backend = CrosstermBackend::new(BufWriter::new(stderr()));
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);

    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    tui.draw(&mut app)?;

    while !app.quit {
        match tui.events.next()? {
            Event::Tick => app.tick().await?,
            Event::Key(key_event) => {
                update(&mut app, key_event).await?;

                // DRAIN: Process any queued keystrokes that arrived while rendering/updating
                // so we don't redraw the entire terminal for every single repeating keypress
                while let Ok(Event::Key(queued_key)) = tui.events.try_next() {
                    update(&mut app, queued_key).await?;
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };

        tui.draw(&mut app)?;
    }

    tui.exit()?;
    Ok(())
}
