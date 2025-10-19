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
use crate::screens::get_screen_handler;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app_state = AppState::default();
    let terminal = ratatui::init();
    let result = run(terminal, &mut app_state);

    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        // Get the current screen handler
        let screen_handler = get_screen_handler(&app_state.current_screen);

        // Render the current screen
        terminal.draw(|frame| {
            screen_handler.render(frame, app_state);
        })?;

        // Handle input events
        if let Event::Key(key) = event::read()? {
            // Let the current screen handle the key event
            match screen_handler.handle_key_event(key, app_state) {
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
