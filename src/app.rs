use crate::data::MatrixService;
use crate::screens::Screen;
use crate::screens::chat::ChatScreenState;
use crate::screens::login::LoginForm;
use crate::screens::{
    ScreenHandler, chat::ChatScreen, homeserver_select::HomeserverSelectScreen, login::LoginScreen,
};

#[derive(Debug)]
pub struct AppState {
    pub current_screen: Screen,
    pub login_form: LoginForm,
    pub homeserver: String,
    pub profile: Profile,
    pub logged_in: bool,
    pub matrix_service: MatrixService,
    pub chat_state: ChatScreenState,
    pub homeserver_screen: HomeserverSelectScreen,
    pub login_screen: LoginScreen,
    pub chat_screen: ChatScreen,
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
            homeserver: String::from("matrix.org"),
            homeserver_screen: HomeserverSelectScreen::default(),
            login_screen: LoginScreen,
            chat_screen: ChatScreen,
        }
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

    pub fn render_current_screen(&self, frame: &mut ratatui::Frame) {
        match self.current_screen {
            Screen::Login => self.login_screen.render(frame, self),
            Screen::Chat => self.chat_screen.render(frame, self),
            Screen::HomeserverSelect => self.homeserver_screen.render(frame, self),
        }
    }

    pub async fn handle_current_screen_key_event(
        &mut self,
        key: ratatui::crossterm::event::KeyEvent,
    ) -> Option<Screen> {
        match self.current_screen {
            Screen::Login => {
                // Placeholder - just handle basic navigation for now
                use ratatui::crossterm::event::KeyCode;
                match key.code {
                    KeyCode::Esc => None,
                    _ => Some(Screen::Login),
                }
            }
            Screen::Chat => {
                // Placeholder - just handle basic navigation for now
                use ratatui::crossterm::event::KeyCode;
                match key.code {
                    KeyCode::Esc => Some(Screen::Login),
                    _ => Some(Screen::Chat),
                }
            }
            Screen::HomeserverSelect => {
                // Handle homeserver screen using separate method to avoid borrowing issues
                let result = self
                    .homeserver_screen
                    .handle_key_event_with_deps(key, &self.matrix_service)
                    .await;

                // Update app state based on result
                if matches!(result, Some(Screen::Login)) {
                    self.login_form.homeserver = self.homeserver_screen.value.clone();
                    self.homeserver = self.homeserver_screen.value.value.clone();
                }

                result
            }
        }
    }

    pub fn login_success(&mut self, user: crate::data::User) {
        self.logged_in = true;
        self.profile.username = self.login_form.username.value.clone();
        self.profile.homeserver = self.login_form.homeserver.value.clone();
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
