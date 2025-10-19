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
    ui::text_input::TextInput,
};
use async_trait::async_trait;

#[derive(Debug, Default)]
pub struct HomeserverSelectScreen;

impl HomeserverSelectScreen {
    pub fn new() -> Self {
        Self
    }

    fn render_homeserver_input(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        let title = "Homeserver";
        let placeholder = "Enter homeserver URL (e.g., matrix.org)";

        let text_input = TextInput::editable(
            &app_state.homeserver_screen.value.value,
            app_state.homeserver_screen.value.cursor_position,
            title,
            placeholder,
        );

        frame.render_widget(text_input, area);
    }
}

#[async_trait]
impl ScreenHandler for HomeserverSelectScreen {
    fn render(&self, frame: &mut Frame, app_state: &AppState) {
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
        self.render_homeserver_input(frame, chunks[0], app_state);

        // Show validation error if invalid
        if app_state.homeserver_screen.invalid {
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
        match key.code {
            KeyCode::Esc => None,
            KeyCode::Enter => {
                if !app_state.homeserver_screen.value.value.is_empty() {
                    match app_state
                        .matrix_service
                        .check_homeserver(&app_state.homeserver_screen.value.value)
                        .await
                    {
                        Ok(true) => {
                            // Update app state when homeserver is valid
                            app_state.login_form.homeserver =
                                app_state.homeserver_screen.value.clone();
                            app_state.homeserver = app_state.homeserver_screen.value.value.clone();
                            app_state.homeserver_screen.invalid = false;
                            Some(Screen::Login)
                        }
                        Ok(false) | Err(_) => {
                            app_state.homeserver_screen.invalid = true;
                            Some(Screen::HomeserverSelect)
                        }
                    }
                } else {
                    app_state.homeserver_screen.invalid = true;
                    Some(Screen::HomeserverSelect)
                }
            }
            _ => {
                // Use TextInputState's built-in key handling
                let handled = app_state.homeserver_screen.value.handle_key_event(key);

                if handled {
                    app_state.homeserver_screen.invalid = false; // Clear error when user starts typing
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
