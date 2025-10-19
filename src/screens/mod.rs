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
        Screen::Chat => Box::new(ChatScreen), // Placeholder for now
    }
}

// Placeholder chat screen
struct ChatScreen;

impl ScreenHandler for ChatScreen {
    fn render(&self, frame: &mut Frame, _app_state: &AppState) {
        use ratatui::{
            text::Line,
            widgets::{Block, Borders, Paragraph},
        };

        let paragraph = Paragraph::new(Line::from("Chat Screen - Coming Soon!"))
            .block(Block::default().borders(Borders::ALL).title("Chat"));

        frame.render_widget(paragraph, frame.area());
    }

    fn handle_key_event(&self, key: KeyEvent, _app_state: &mut AppState) -> Option<Screen> {
        use ratatui::crossterm::event::KeyCode;

        match key.code {
            KeyCode::Char('q') => None,          // Quit application
            KeyCode::Esc => Some(Screen::Login), // Go back to login
            _ => Some(Screen::Chat),             // Stay on chat screen
        }
    }
}
