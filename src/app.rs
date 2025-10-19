use crate::screens::{Screen, login::LoginField};

#[derive(Debug)]
pub struct AppState {
    pub current_screen: Screen,
    pub login_form: LoginForm,
    pub profile: Profile,
    pub logged_in: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::default(),
            login_form: LoginForm::default(),
            profile: Profile::default(),
            logged_in: false,
        }
    }
}

#[derive(Debug)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub homeserver: String,
    pub active_field: LoginField,
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
            active_field: LoginField::default(),
            editing: false,
            username_cursor: 0,
            password_cursor: 0,
            homeserver_cursor: 10, // Position after "matrix.org"
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

    pub fn enter_char(&mut self, c: char) {
        match self.active_field {
            LoginField::Username => {
                self.username.insert(self.username_cursor, c);
                self.username_cursor += 1;
            }
            LoginField::Password => {
                self.password.insert(self.password_cursor, c);
                self.password_cursor += 1;
            }
            LoginField::Homeserver => {
                self.homeserver.insert(self.homeserver_cursor, c);
                self.homeserver_cursor += 1;
            }
        }
    }

    pub fn delete_char(&mut self) {
        match self.active_field {
            LoginField::Username => {
                if self.username_cursor > 0 {
                    self.username.remove(self.username_cursor - 1);
                    self.username_cursor -= 1;
                }
            }
            LoginField::Password => {
                if self.password_cursor > 0 {
                    self.password.remove(self.password_cursor - 1);
                    self.password_cursor -= 1;
                }
            }
            LoginField::Homeserver => {
                if self.homeserver_cursor > 0 {
                    self.homeserver.remove(self.homeserver_cursor - 1);
                    self.homeserver_cursor -= 1;
                }
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        match self.active_field {
            LoginField::Username => {
                if self.username_cursor > 0 {
                    self.username_cursor -= 1;
                }
            }
            LoginField::Password => {
                if self.password_cursor > 0 {
                    self.password_cursor -= 1;
                }
            }
            LoginField::Homeserver => {
                if self.homeserver_cursor > 0 {
                    self.homeserver_cursor -= 1;
                }
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        match self.active_field {
            LoginField::Username => {
                if self.username_cursor < self.username.len() {
                    self.username_cursor += 1;
                }
            }
            LoginField::Password => {
                if self.password_cursor < self.password.len() {
                    self.password_cursor += 1;
                }
            }
            LoginField::Homeserver => {
                if self.homeserver_cursor < self.homeserver.len() {
                    self.homeserver_cursor += 1;
                }
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
        self.active_field = LoginField::Username;
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

    pub fn login_success(&mut self) {
        self.logged_in = true;
        self.profile.username = self.login_form.username.clone();
        self.profile.homeserver = self.login_form.homeserver.clone();
        self.current_screen = Screen::Chat;
    }

    pub fn logout(&mut self) {
        self.logged_in = false;
        self.profile = Profile::default();
        self.login_form.clear();
        self.current_screen = Screen::Login;
    }
}
