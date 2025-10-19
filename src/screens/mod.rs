pub mod chat;
pub mod homeserver_select;
pub mod login;

use crate::app::AppState;
use async_trait::async_trait;
use ratatui::{Frame, crossterm::event::KeyEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Chat,
    HomeserverSelect,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::HomeserverSelect
    }
}

#[async_trait]
pub trait ScreenHandler {
    fn render(&self, frame: &mut Frame, app_state: &AppState);
    async fn handle_key_event(&mut self, key: KeyEvent, app_state: &mut AppState)
    -> Option<Screen>;
}

// Keep this function for backward compatibility if needed
pub fn get_screen_handler(screen: &Screen) -> Box<dyn ScreenHandler> {
    match screen {
        Screen::Login => Box::new(login::LoginScreen),
        Screen::Chat => Box::new(chat::ChatScreen),
        Screen::HomeserverSelect => Box::new(homeserver_select::HomeserverSelectScreen::default()),
    }
}
