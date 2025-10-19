use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    app::AppState,
    screens::{Screen, ScreenHandler},
    ui::text_input::TextInput,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LoginField {
    Username,
    Password,
    Homeserver,
}

impl Default for LoginField {
    fn default() -> Self {
        LoginField::Username
    }
}

impl LoginField {
    fn to_index(&self) -> usize {
        match self {
            LoginField::Username => 0,
            LoginField::Password => 1,
            LoginField::Homeserver => 2,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => LoginField::Username,
            1 => LoginField::Password,
            2 => LoginField::Homeserver,
            _ => LoginField::Username,
        }
    }
}

pub struct LoginScreen;

impl LoginScreen {
    fn get_field_title(&self, field: &LoginField) -> &'static str {
        match field {
            LoginField::Username => "Username",
            LoginField::Password => "Password",
            LoginField::Homeserver => "Homeserver",
        }
    }

    fn get_field_placeholder(&self, field: &LoginField) -> &'static str {
        match field {
            LoginField::Username => "@username",
            LoginField::Password => "Enter your password",
            LoginField::Homeserver => "matrix.org",
        }
    }

    fn get_field_value<'a>(&self, field: &LoginField, app_state: &'a AppState) -> &'a str {
        app_state.login_form.get_field_value(field.to_index())
    }

    fn render_field_input(
        &self,
        frame: &mut Frame,
        area: Rect,
        field: &LoginField,
        app_state: &AppState,
        is_active: bool,
    ) {
        let value = self.get_field_value(field, app_state);
        let title = self.get_field_title(field);
        let placeholder = self.get_field_placeholder(field);

        let cursor_pos = app_state.login_form.get_field_cursor(field.to_index());

        let text_input = if matches!(field, LoginField::Password) {
            TextInput::password_field(value, cursor_pos, title, placeholder)
                .focused(is_active)
                .editing(app_state.login_form.editing && is_active)
        } else if is_active && app_state.login_form.editing {
            TextInput::editable(value, cursor_pos, title, placeholder)
        } else {
            TextInput::new(value, cursor_pos, title, placeholder).focused(is_active)
        };

        frame.render_widget(text_input, area);
    }

    fn center_rect(&self, percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl ScreenHandler for LoginScreen {
    fn render(&self, frame: &mut Frame, app_state: &AppState) {
        // Clear the entire area
        frame.render_widget(Clear, frame.area());

        // Create centered login form
        let form_area = self.center_rect(50, 60, frame.area());

        // Main container
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

        // Render form fields
        let active_field = LoginField::from_index(app_state.login_form.active_field);

        self.render_field_input(
            frame,
            chunks[0],
            &LoginField::Username,
            app_state,
            active_field == LoginField::Username,
        );

        self.render_field_input(
            frame,
            chunks[1],
            &LoginField::Password,
            app_state,
            active_field == LoginField::Password,
        );

        self.render_field_input(
            frame,
            chunks[2],
            &LoginField::Homeserver,
            app_state,
            active_field == LoginField::Homeserver,
        );

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

    fn handle_key_event(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen> {
        if app_state.login_form.editing {
            // Handle editing mode
            match key.code {
                KeyCode::Esc => {
                    app_state.login_form.editing = false;
                    Some(Screen::Login)
                }
                KeyCode::Enter => {
                    app_state.login_form.editing = false;
                    // Trigger dummy login attempt
                    if app_state.login_form.is_valid() {
                        // In a real implementation, this would be async
                        // For now, simulate successful login
                        let user = crate::data::User {
                            user_id: format!(
                                "@{}:{}",
                                app_state.login_form.username, app_state.login_form.homeserver
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
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app_state.login_form.previous_field();
                    } else {
                        app_state.login_form.next_field();
                    }
                    Some(Screen::Login)
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
