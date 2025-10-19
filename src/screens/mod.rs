pub mod homeserver_select;
pub mod login;

use async_trait::async_trait;
use ratatui::{Frame, crossterm::event::KeyEvent};

use crate::app::AppState;

#[derive(Clone, Debug)]
pub enum Screen {
    HomeServerSelect,
    Login,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::HomeServerSelect
    }
}

#[async_trait]
pub trait ScreenHandler {
    fn render(&self, frame: &mut Frame, app_state: &AppState);
    async fn handle_key_event(&mut self, key: KeyEvent, app_state: &mut AppState)
    -> Option<Screen>;
}

pub fn get_screen_handler(screen: &Screen) -> Box<dyn ScreenHandler> {
    match screen {
        Screen::HomeServerSelect => Box::new(homeserver_select::HomeserverSelectScreen::new()),
        Screen::Login => Box::new(login::LoginScreen),
    }
}
