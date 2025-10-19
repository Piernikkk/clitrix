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
    ui::{input_handler::TextInputState, text_input::TextInput},
};

#[derive(Debug)]
pub struct LoginForm {
    pub username: TextInputState,
    pub password: TextInputState,
    pub homeserver: TextInputState,
    pub active_field: LoginField,
    pub editing: bool,
}

impl Default for LoginForm {
    fn default() -> Self {
        Self {
            username: TextInputState::default(),
            password: TextInputState::default(),
            homeserver: TextInputState::new("matrix.org".to_string()),
            active_field: LoginField::Username,
            editing: false,
        }
    }
}

impl LoginForm {
    pub fn next_field(&mut self) {
        self.active_field = match self.active_field {
            LoginField::Username => LoginField::Password,
            LoginField::Password => LoginField::Homeserver,
            LoginField::Homeserver => LoginField::Username,
        };
    }

    pub fn previous_field(&mut self) {
        self.active_field = match self.active_field {
            LoginField::Username => LoginField::Homeserver,
            LoginField::Password => LoginField::Username,
            LoginField::Homeserver => LoginField::Password,
        };
    }

    pub fn get_active_field_mut(&mut self) -> &mut TextInputState {
        match self.active_field {
            LoginField::Username => &mut self.username,
            LoginField::Password => &mut self.password,
            LoginField::Homeserver => &mut self.homeserver,
        }
    }

    pub fn get_field_mut(&mut self, field: &LoginField) -> &mut TextInputState {
        match field {
            LoginField::Username => &mut self.username,
            LoginField::Password => &mut self.password,
            LoginField::Homeserver => &mut self.homeserver,
        }
    }

    pub fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
        self.homeserver.set_value("matrix.org".to_string());
        self.active_field = LoginField::Username;
        self.editing = false;
    }

    pub fn is_valid(&self) -> bool {
        !self.username.value.is_empty()
            && !self.password.value.is_empty()
            && !self.homeserver.value.is_empty()
    }

    pub fn is_homeserver_valid(&self) -> bool {
        !self.homeserver.value.is_empty()
    }

    pub fn get_homeserver_error(&self) -> Option<&'static str> {
        if self.homeserver.value.is_empty() {
            Some("Homeserver cannot be empty")
        } else {
            None
        }
    }

    pub async fn validate_homeserver_async(
        &self,
        matrix_service: &crate::data::MatrixService,
    ) -> Result<bool, String> {
        match matrix_service
            .check_homeserver(&self.homeserver.value)
            .await
        {
            Ok(valid) => Ok(valid),
            Err(e) => Err(e.to_string()),
        }
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

#[derive(Debug)]
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

    fn get_field_state<'a>(
        &self,
        field: &LoginField,
        app_state: &'a AppState,
    ) -> &'a TextInputState {
        match field {
            LoginField::Username => &app_state.login_form.username,
            LoginField::Password => &app_state.login_form.password,
            LoginField::Homeserver => &app_state.login_form.homeserver,
        }
    }

    fn render_field_input(
        &self,
        frame: &mut Frame,
        area: Rect,
        field: &LoginField,
        app_state: &AppState,
        is_active: bool,
    ) {
        let field_state = self.get_field_state(field, app_state);
        let title = self.get_field_title(field);
        let placeholder = self.get_field_placeholder(field);

        let text_input = if matches!(field, LoginField::Password) {
            TextInput::password_field(
                &field_state.value,
                field_state.cursor_position,
                title,
                placeholder,
            )
            .focused(is_active)
            .editing(app_state.login_form.editing && is_active)
        } else if is_active && app_state.login_form.editing {
            TextInput::editable(
                &field_state.value,
                field_state.cursor_position,
                title,
                placeholder,
            )
        } else {
            TextInput::new(
                &field_state.value,
                field_state.cursor_position,
                title,
                placeholder,
            )
            .focused(is_active)
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

#[async_trait::async_trait]
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
        self.render_field_input(
            frame,
            chunks[0],
            &LoginField::Username,
            app_state,
            app_state.login_form.active_field == LoginField::Username,
        );

        self.render_field_input(
            frame,
            chunks[1],
            &LoginField::Password,
            app_state,
            app_state.login_form.active_field == LoginField::Password,
        );

        self.render_field_input(
            frame,
            chunks[2],
            &LoginField::Homeserver,
            app_state,
            app_state.login_form.active_field == LoginField::Homeserver,
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
                                app_state.login_form.username.value,
                                app_state.login_form.homeserver.value
                            ),
                            display_name: Some(app_state.login_form.username.value.clone()),
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
                _ => {
                    // Let the active text input handle the key event
                    let active_field = app_state.login_form.get_active_field_mut();
                    if active_field.handle_key_event(key) {
                        Some(Screen::Login)
                    } else {
                        // If text input didn't handle it, check for global shortcuts
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                None
                            }
                            _ => Some(Screen::Login),
                        }
                    }
                }
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
                                            app_state.login_form.username.value,
                                            app_state.login_form.homeserver.value
                                        ),
                                        display_name: Some(
                                            app_state.login_form.username.value.clone(),
                                        ),
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

impl LoginScreen {
    pub async fn handle_key_event_with_deps(
        &self,
        key: KeyEvent,
        login_form: &mut LoginForm,
    ) -> Option<Screen> {
        if login_form.editing {
            // Handle editing mode
            match key.code {
                KeyCode::Esc => {
                    login_form.editing = false;
                    Some(Screen::Login)
                }
                KeyCode::Enter => {
                    login_form.editing = false;
                    // Trigger dummy login attempt
                    if login_form.is_valid() {
                        // In a real implementation, this would be async
                        // For now, simulate successful login
                        let user = crate::data::User {
                            user_id: format!(
                                "@{}:{}",
                                login_form.username.value, login_form.homeserver.value
                            ),
                            display_name: Some(login_form.username.value.clone()),
                            avatar_url: None,
                            presence: crate::data::UserPresence::Online,
                        };
                        // Note: We can't update matrix_service and login_success here
                        // That will need to be handled in the calling code
                        Some(Screen::Chat)
                    } else {
                        // Stay on login screen if form is invalid
                        Some(Screen::Login)
                    }
                }
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        login_form.previous_field();
                    } else {
                        login_form.next_field();
                    }
                    Some(Screen::Login)
                }
                _ => {
                    // Let the active text input handle the key event
                    let active_field = login_form.get_active_field_mut();
                    if active_field.handle_key_event(key) {
                        Some(Screen::Login)
                    } else {
                        // If text input didn't handle it, check for global shortcuts
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                None
                            }
                            _ => Some(Screen::Login),
                        }
                    }
                }
            }
        } else {
            // Handle navigation mode
            match key.code {
                KeyCode::Char('q') => None,
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        login_form.previous_field();
                    } else {
                        login_form.next_field();
                    }
                    Some(Screen::Login)
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    login_form.editing = true;
                    Some(Screen::Login)
                }
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'c' => None, // Quit
                            'l' => {
                                // Trigger dummy login attempt
                                if login_form.is_valid() {
                                    // In a real implementation, this would be async
                                    // For now, simulate successful login
                                    let user = crate::data::User {
                                        user_id: format!(
                                            "@{}:{}",
                                            login_form.username.value, login_form.homeserver.value
                                        ),
                                        display_name: Some(login_form.username.value.clone()),
                                        avatar_url: None,
                                        presence: crate::data::UserPresence::Online,
                                    };
                                    // Note: We can't update matrix_service and login_success here
                                    // That will need to be handled in the calling code
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
