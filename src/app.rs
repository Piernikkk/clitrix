use crate::data::MatrixService;
use crate::screens::Screen;
use crate::screens::chat::ChatScreenState;

#[derive(Debug)]
pub struct AppState {
    pub current_screen: Screen,
    pub login_form: LoginForm,
    pub profile: Profile,
    pub logged_in: bool,
    pub matrix_service: MatrixService,
    pub chat_state: ChatScreenState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::default(),
            login_form: LoginForm::default(),
            profile: Profile::default(),
            logged_in: false,
            matrix_service: MatrixService::new(),
            chat_state: ChatScreenState::default(),
        }
    }
}

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

#[derive(Debug)]
pub struct Profile {
    pub username: String,
    pub email: String,
    pub homeserver: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            username: String::new(),
            email: String::new(),
            homeserver: String::from("matrix.org"),
        }
    }
}

impl AppState {
    pub fn set_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }

    pub fn login_success(&mut self, user: crate::data::User) {
        self.logged_in = true;
        self.profile.username = self.login_form.username.clone();
        self.profile.homeserver = self.login_form.homeserver.clone();
        self.chat_state = ChatScreenState::default().with_user(user);
        self.current_screen = Screen::Chat;
    }

    pub fn logout(&mut self) {
        self.logged_in = false;
        self.profile = Profile::default();
        self.login_form.clear();
        self.matrix_service = MatrixService::new();
        self.chat_state = ChatScreenState::default();
        self.current_screen = Screen::Login;
    }
}
