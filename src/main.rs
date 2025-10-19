mod app;
mod data;
mod screens;
mod ui;

use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event},
};

use crate::app::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app_state = AppState::default();
    let terminal = ratatui::init();
    let result = run(terminal, &mut app_state).await;

    ratatui::restore();
    result
}

async fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        // Render the current screen
        terminal.draw(|frame| {
            app_state.render_current_screen(frame);
        })?;

        // Handle input events
        if let Event::Key(key) = event::read()? {
            // Let the current screen handle the key event
            match app_state.handle_current_screen_key_event(key).await {
                Some(new_screen) => {
                    app_state.set_screen(new_screen);
                }
                None => {
                    // Screen handler returned None, indicating we should quit
                    break;
                }
            }
        }
    }

    Ok(())
}
