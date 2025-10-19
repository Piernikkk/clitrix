use async_trait::async_trait;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    app::AppState,
    matrix_service::MatrixService,
    screens::{Screen, ScreenHandler},
    ui::text_input::{TextInput, input_handler::TextInputState},
};

struct Colors {
    border: Color,
    error_msg: Color,
    controls_title: Color,
}

const COLORS: Colors = Colors {
    border: Color::Cyan,
    error_msg: Color::Red,
    controls_title: Color::Green,
};

pub struct HomeserverSelectScreen;

#[derive(Debug)]
pub struct HomeserverSelectState {
    text_input: TextInputState,
    invalid: Option<String>,
}

impl Default for HomeserverSelectState {
    fn default() -> Self {
        Self {
            text_input: TextInputState::new("matrix.org".to_string(), true),
            invalid: None,
        }
    }
}

impl HomeserverSelectScreen {
    pub fn new() -> Self {
        Self
    }
    fn render_homeserver_input(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let text_input = TextInput::new(
            &state.homeserver_select_screen.text_input.value,
            state.homeserver_select_screen.text_input.cursor_position,
            "Homeserver",
            "Enter your homeserver URL (e.g., matrix.org)",
            false,
            state.homeserver_select_screen.text_input.is_focused,
        );

        frame.render_widget(text_input, area);
    }
}

#[async_trait]
impl ScreenHandler for HomeserverSelectScreen {
    fn render(&self, frame: &mut Frame, state: &AppState) {
        frame.render_widget(Clear, frame.area());

        let form_container = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([Constraint::Length(12)])
            .split(frame.area())[0]
            .inner(Margin {
                horizontal: 10,
                vertical: 0,
            });

        let main_block = Block::default()
            .borders(Borders::ALL)
            .title("Homeserver Selection")
            .title_style(
                Style::default()
                    .fg(COLORS.border)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(main_block, form_container);

        let content = form_container.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Homeserver input
                Constraint::Length(1), // Error message
                Constraint::Length(5), // Instructions
            ])
            .split(content);

        self.render_homeserver_input(frame, chunks[0], state);

        if state.homeserver_select_screen.invalid.is_some() {
            let error_msg = Paragraph::new(state.homeserver_select_screen.invalid.clone().unwrap())
                .style(Style::default().fg(COLORS.error_msg));
            frame.render_widget(error_msg, chunks[1]);
        }

        let instructions = vec![
            Line::from("Type to edit homeserver URL"),
            Line::from("Enter - Validate and continue"),
            Line::from("ESC - Exit application"),
        ];

        let instructions_paragraph = Paragraph::new(instructions)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .title_style(Style::default().fg(COLORS.controls_title)),
            )
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(instructions_paragraph, chunks[2]);
    }

    async fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> Option<Screen> {
        match key.code {
            KeyCode::Esc => None,
            KeyCode::Enter => {
                if state.homeserver_select_screen.text_input.value.is_empty() {
                    state.homeserver_select_screen.invalid =
                        Some("❌ Homeserver URL cannot be empty.".to_string());
                    Some(Screen::HomeServerSelect)
                } else {
                    match MatrixService::check_homeserver(
                        &state.homeserver_select_screen.text_input.value,
                    )
                    .await
                    {
                        Ok(_) => {
                            state
                                .matrix_service
                                .set_homeserver(&state.homeserver_select_screen.text_input.value);
                            Some(Screen::Login)
                        }
                        Err(err) => {
                            state.homeserver_select_screen.invalid =
                                Some(format!("❌ Invalid homeserver: {}", err));
                            Some(Screen::HomeServerSelect)
                        }
                    }
                }
            }
            _ => {
                if state.homeserver_select_screen.text_input.is_focused {
                    if state
                        .homeserver_select_screen
                        .text_input
                        .handle_key_event(key)
                        == true
                    {
                        state.homeserver_select_screen.invalid = None;
                    }
                }

                Some(Screen::HomeServerSelect)
            }
        }
    }
}
