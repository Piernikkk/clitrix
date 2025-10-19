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

#[derive(Default)]
pub struct HomeserverSelectScreen {
    homeserver_cursor: usize,
    editing: bool,
    is_active: bool,
    value: String,
    invalid: bool,
}

impl HomeserverSelectScreen {
    fn render_homeserver_input(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        let value = app_state.homeserver.as_str();
        let title = "Homeserver";
        let placeholder = "Enter homeserver URL";

        let cursor_pos = self.homeserver_cursor;

        let text_input = if self.is_active && app_state.login_form.editing {
            TextInput::editable(value, cursor_pos, title, placeholder)
        } else {
            TextInput::new(value, cursor_pos, title, placeholder).focused(self.is_active)
        };

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
            .constraints([Constraint::Length(19)])
            .split(frame.area())[0];

        let main_block = Block::default()
            .borders(Borders::ALL)
            .title("Matrix Login")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(main_block, form_area);

        // Inner area for form fields
        let inner_area = form_area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });

        // Layout for form fields and instructions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Homeserver
                Constraint::Length(2), // Spacer
                Constraint::Length(4), // Instructions
                Constraint::Min(0),    // Remaining space
            ])
            .split(inner_area);

        // let active_field = LoginField::from_index(app_state.login_form.active_field);

        // self.render_field_input(
        //     frame,
        //     chunks[0],
        //     &LoginField::Username,
        //     app_state,
        //     active_field == LoginField::Username,
        // );

        // self.render_field_input(
        //     frame,
        //     chunks[1],
        //     &LoginField::Password,
        //     app_state,
        //     active_field == LoginField::Password,
        // );

        self.render_homeserver_input(frame, chunks[2], app_state);

        // Instructions
        let instructions = if app_state.login_form.editing {
            vec![
                Line::from("ESC - Stop editing"),
                Line::from("Tab/Shift+Tab - Switch fields"),
                Line::from("Enter - Submit login"),
                Line::from("Ctrl+C - Quit"),
            ]
        } else {
            vec![
                Line::from("Enter/Space - Start editing field"),
                Line::from("Tab/Shift+Tab - Switch fields"),
                Line::from("Ctrl+L - Submit login"),
                Line::from("Ctrl+C - Quit"),
            ]
        };

        let instructions_paragraph = Paragraph::new(instructions)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(instructions_paragraph, chunks[4]);
    }

    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
        app_state: &mut AppState,
    ) -> Option<Screen> {
        if app_state.login_form.editing {
            // Handle editing mode
            match key.code {
                KeyCode::Esc => {
                    self.editing = false;
                    Some(Screen::Login)
                }
                KeyCode::Enter => {
                    self.editing = false;
                    if app_state
                        .matrix_service
                        .check_homeserver(&self.value)
                        .await
                        .is_ok()
                    {
                        app_state.login_form.homeserver = self.value.clone();
                        Some(Screen::Login)
                    } else {
                        // Stay on login screen if form is invalid
                        self.invalid = true;
                        Some(Screen::HomeserverSelect)
                    }
                }
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'c' => None, // Quit
                            _ => Some(Screen::Login),
                        }
                    } else {
                        app_state.login_form.enter_char(c);
                        Some(Screen::Login)
                    }
                }
                KeyCode::Backspace => {
                    app_state.login_form.delete_char();
                    Some(Screen::Login)
                }
                KeyCode::Left => {
                    app_state.login_form.move_cursor_left();
                    Some(Screen::Login)
                }
                KeyCode::Right => {
                    app_state.login_form.move_cursor_right();
                    Some(Screen::Login)
                }
                _ => Some(Screen::Login),
            }
        } else {
            // Handle navigation mode
            match key.code {
                KeyCode::Char('q') => None,
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app_state.login_form.previous_field();
                    } else {
                        app_state.login_form.next_field();
                    }
                    Some(Screen::Login)
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    app_state.login_form.editing = true;
                    Some(Screen::Login)
                }
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'c' => None, // Quit
                            'l' => {
                                // Trigger dummy login attempt
                                if app_state.login_form.is_valid() {
                                    // In a real implementation, this would be async
                                    // For now, simulate successful login
                                    let user = crate::data::User {
                                        user_id: format!(
                                            "@{}:{}",
                                            app_state.login_form.username,
                                            app_state.login_form.homeserver
                                        ),
                                        display_name: Some(app_state.login_form.username.clone()),
                                        avatar_url: None,
                                        presence: crate::data::UserPresence::Online,
                                    };
                                    app_state.matrix_service.current_user = Some(user.clone());
                                    app_state.matrix_service.is_authenticated = true;
                                    app_state.login_success(user);
                                    Some(Screen::Chat)
                                } else {
                                    // Stay on login screen if form is invalid
                                    Some(Screen::Login)
                                }
                            }
                            _ => Some(Screen::Login),
                        }
                    } else {
                        Some(Screen::Login)
                    }
                }
                _ => Some(Screen::Login),
            }
        }
    }
}
