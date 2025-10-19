mod app;
pub mod matrix_service;
pub mod models;
mod screens;
mod ui;

use async_trait::async_trait;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event},
};

use crate::{app::AppState, screens::get_screen_handler};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app_state = AppState::new();
    let terminal = ratatui::init();
    let result = run(terminal, &mut app_state).await;

    ratatui::restore();
    result
}

async fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        let mut screen_handler = get_screen_handler(&app_state.current_screen);

        terminal.draw(|frame| {
            screen_handler.render(frame, app_state);
        })?;

        if let Event::Key(key) = event::read()? {
            match screen_handler.handle_key_event(key, app_state).await {
                Some(new_screen) => {
                    app_state.set_screen(new_screen);
                }
                None => {
                    break;
                }
            }
        }
    }

    Ok(())
}
