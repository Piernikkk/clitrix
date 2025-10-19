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
use async_trait::async_trait;

#[derive(Debug)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub homeserver: String,
    pub active_field: usize, // Index-based field selection
    pub editing: bool,
    pub username_cursor: usize,
    pub password_cursor: usize,
    pub homeserver_cursor: usize,
}

impl Default for LoginForm {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            homeserver: String::from("matrix.org"),
            active_field: 0, // Start with username field
            editing: false,
            username_cursor: 0,
            password_cursor: 0,
            homeserver_cursor: 10, // Position after "matrix.org"
        }
    }
}

impl LoginForm {
    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % 3;
    }

    pub fn previous_field(&mut self) {
        self.active_field = if self.active_field == 0 {
            2
        } else {
            self.active_field - 1
        };
    }

    pub fn get_field_value(&self, field_index: usize) -> &str {
        match field_index {
            0 => &self.username,
            1 => &self.password,
            2 => &self.homeserver,
            _ => "",
        }
    }

    pub fn get_field_cursor(&self, field_index: usize) -> usize {
        match field_index {
            0 => self.username_cursor,
            1 => self.password_cursor,
            2 => self.homeserver_cursor,
            _ => 0,
        }
    }

    pub fn get_field_mut(&mut self, field_index: usize) -> Option<(&mut String, &mut usize)> {
        match field_index {
            0 => Some((&mut self.username, &mut self.username_cursor)),
            1 => Some((&mut self.password, &mut self.password_cursor)),
            2 => Some((&mut self.homeserver, &mut self.homeserver_cursor)),
            _ => None,
        }
    }

    pub fn enter_char(&mut self, c: char) {
        if let Some((value, cursor)) = self.get_field_mut(self.active_field) {
            value.insert(*cursor, c);
            *cursor += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if let Some((value, cursor)) = self.get_field_mut(self.active_field) {
            if *cursor > 0 {
                value.remove(*cursor - 1);
                *cursor -= 1;
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if let Some((_, cursor)) = self.get_field_mut(self.active_field) {
            if *cursor > 0 {
                *cursor -= 1;
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        if let Some((value, cursor)) = self.get_field_mut(self.active_field) {
            if *cursor < value.len() {
                *cursor += 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
        self.homeserver = String::from("matrix.org");
        self.username_cursor = 0;
        self.password_cursor = 0;
        self.homeserver_cursor = 10;
        self.active_field = 0;
        self.editing = false;
    }

    pub fn is_valid(&self) -> bool {
        !self.username.is_empty() && !self.password.is_empty() && !self.homeserver.is_empty()
    }
}

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

#[async_trait]
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

    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
        app_state: &mut AppState,
    ) -> Option<Screen> {
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
