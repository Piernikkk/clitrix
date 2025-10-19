use ratatui::{DefaultTerminal, Frame};

use crate::settings::Settings;

pub struct App {
    pub page: Page,
    pub title: String,
    pub settings: Settings,
}

pub enum Page {
    Home,
    Chat,
    Login,
}

impl App {
    pub fn new(title: String) -> Self {
        return Self {
            title,
            page: Page::Login,
            settings: Settings::default(),
        };
    }
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
        }
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}
