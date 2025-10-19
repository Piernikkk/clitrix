use std::rc::Rc;

use async_trait::async_trait;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
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

pub struct LoginScreen;

#[derive(Debug)]
pub struct LoginScreenState {
    username: TextInputState,
    password: TextInputState,
    invalid: Option<String>,
}

impl Default for LoginScreenState {
    fn default() -> Self {
        Self {
            username: TextInputState::new(String::new(), true),
            password: TextInputState::new(String::new(), false),
            invalid: None,
        }
    }
}

impl LoginScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn render_inputs(&self, frame: &mut Frame, area: &Rc<[Rect]>, state: &AppState) {
        let username_area = area[0];
        let password_area = area[1];

        let username_input = TextInput::new(
            &state.login_screen.username.value,
            state.login_screen.username.cursor_position,
            "Usernmae",
            "Enter your username",
            false,
            state.login_screen.username.is_focused,
        );

        let password_input = TextInput::new(
            &state.login_screen.password.value,
            state.login_screen.password.cursor_position,
            "Password",
            "Enter your password",
            true,
            state.login_screen.password.is_focused,
        );

        frame.render_widget(username_input, username_area);
        frame.render_widget(password_input, password_area);
    }
}

#[async_trait]
impl ScreenHandler for LoginScreen {
    fn render(&self, frame: &mut Frame, state: &crate::app::AppState) {
        frame.render_widget(Clear, frame.area());

        let form_container = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([Constraint::Length(15)])
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
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(1), // Error message
                Constraint::Length(6), // Instructions
            ])
            .split(content);

        self.render_inputs(frame, &chunks, state);

        if state.login_screen.invalid.is_some() {
            let error_msg = Paragraph::new(state.login_screen.invalid.clone().unwrap())
                .style(Style::default().fg(COLORS.error_msg));
            frame.render_widget(error_msg, chunks[2]);
        }

        let instructions = vec![
            Line::from("Type to edit homeserver URL"),
            Line::from("Enter - Validate and continue"),
            Line::from("Tab - Switch input field"),
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

        frame.render_widget(instructions_paragraph, chunks[3]);
    }

    async fn handle_key_event(
        &mut self,
        key: ratatui::crossterm::event::KeyEvent,
        state: &mut crate::app::AppState,
    ) -> Option<crate::screens::Screen> {
        match key.code {
            KeyCode::Esc => None,
            KeyCode::Tab => {
                if state.login_screen.username.is_focused {
                    state.login_screen.username.is_focused = false;
                    state.login_screen.password.is_focused = true;
                } else if state.login_screen.password.is_focused {
                    state.login_screen.password.is_focused = false;
                    state.login_screen.username.is_focused = true;
                }

                Some(Screen::Login)
            }
            KeyCode::Enter => {
                if state.login_screen.username.value.is_empty() {
                    state.login_screen.invalid = Some("❌ Username cannot be empty.".to_string());
                    Some(Screen::Login)
                } else if state.login_screen.password.value.is_empty() {
                    state.login_screen.invalid = Some("❌ Password cannot be empty.".to_string());
                    Some(Screen::Login)
                } else {
                    Some(Screen::Login)
                }
            }
            _ => {
                if state.login_screen.username.is_focused {
                    if state.login_screen.username.handle_key_event(key) == true {
                        state.login_screen.invalid = None;
                    }
                } else if state.login_screen.password.is_focused {
                    if state.login_screen.password.handle_key_event(key) == true {
                        state.login_screen.invalid = None;
                    }
                }

                Some(Screen::Login)
            }
        }
    }
}
