use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    app::AppState,
    screens::{Screen, ScreenHandler},
    ui::{input_handler::TextInputState, text_input::TextInput},
};
use async_trait::async_trait;

#[derive(Debug)]
pub struct HomeserverSelectScreen {
    pub value: TextInputState,
    pub invalid: bool,
}

impl Default for HomeserverSelectScreen {
    fn default() -> Self {
        Self {
            value: TextInputState::new("matrix.org".to_string()), // Start with a default value
            invalid: false,
        }
    }
}

impl HomeserverSelectScreen {
    fn render_homeserver_input(&self, frame: &mut Frame, area: Rect) {
        let title = "Homeserver";
        let placeholder = "Enter homeserver URL (e.g., matrix.org)";

        let text_input = TextInput::editable(
            &self.value.value,
            self.value.cursor_position,
            title,
            placeholder,
        );

        frame.render_widget(text_input, area);
    }
}

#[async_trait]
impl ScreenHandler for HomeserverSelectScreen {
    fn render(&self, frame: &mut Frame, _app_state: &AppState) {
        frame.render_widget(Clear, frame.area());

        let form_area = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([Constraint::Length(15)])
            .split(frame.area())[0];

        let main_block = Block::default()
            .borders(Borders::ALL)
            .title("Homeserver Selection")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(main_block, form_area);

        let inner_area = form_area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Homeserver input
                Constraint::Length(1), // Error message
                Constraint::Length(4), // Instructions
                Constraint::Min(0),    // Remaining space
            ])
            .split(inner_area);

        // Render homeserver input
        self.render_homeserver_input(frame, chunks[0]);

        // Show validation error if invalid
        if self.invalid {
            let error_msg =
                Paragraph::new("❌ Invalid homeserver. Please check the URL and try again.")
                    .style(Style::default().fg(Color::Red));
            frame.render_widget(error_msg, chunks[1]);
        }

        // Instructions
        let instructions = vec![
            Line::from("Type to edit homeserver URL"),
            Line::from("Enter - Validate and continue"),
            Line::from("ESC - Exit application"),
            Line::from("Type characters to edit the homeserver URL"),
        ];

        let instructions_paragraph = Paragraph::new(instructions)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(instructions_paragraph, chunks[2]);
    }

    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
        app_state: &mut AppState,
    ) -> Option<Screen> {
        self.handle_key_event_with_deps(key, &app_state.matrix_service)
            .await
    }
}

impl HomeserverSelectScreen {
    pub async fn handle_key_event_with_deps(
        &mut self,
        key: KeyEvent,
        matrix_service: &crate::data::MatrixService,
    ) -> Option<Screen> {
        match key.code {
            KeyCode::Esc => None,
            KeyCode::Enter => {
                if !self.value.value.is_empty() {
                    match matrix_service.check_homeserver(&self.value.value).await {
                        Ok(true) => {
                            self.invalid = false;
                            Some(Screen::Login)
                        }
                        Ok(false) | Err(_) => {
                            self.invalid = true;
                            Some(Screen::HomeserverSelect)
                        }
                    }
                } else {
                    self.invalid = true;
                    Some(Screen::HomeserverSelect)
                }
            }
            _ => {
                // Use TextInputState's built-in key handling
                let handled = self.value.handle_key_event(key);

                if handled {
                    self.invalid = false; // Clear error when user starts typing
                    Some(Screen::HomeserverSelect)
                } else {
                    // Handle keys that TextInputState doesn't handle
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => None,
                        _ => Some(Screen::HomeserverSelect),
                    }
                }
            }
        }
    }
}
