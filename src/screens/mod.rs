pub mod chat;
pub mod login;

use crate::app::AppState;
use ratatui::{Frame, crossterm::event::KeyEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Chat,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Login
    }
}

pub trait ScreenHandler {
    fn render(&self, frame: &mut Frame, app_state: &AppState);
    fn handle_key_event(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen>;
}

pub fn get_screen_handler(screen: &Screen) -> Box<dyn ScreenHandler> {
    match screen {
        Screen::Login => Box::new(login::LoginScreen),
        Screen::Chat => Box::new(chat::ChatScreen),
    }
}
