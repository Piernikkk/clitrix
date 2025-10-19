use crate::{
    matrix_service::MatrixService,
    screens::{
        Screen, chat::ChatScreenState, homeserver_select::HomeserverSelectState,
        login::LoginScreenState,
    },
};

#[derive(Debug)]
pub struct AppState {
    pub matrix_service: MatrixService,
    pub current_screen: Screen,
    pub homeserver_select_screen: HomeserverSelectState,
    pub login_screen: LoginScreenState,
    pub chat_screen: ChatScreenState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            matrix_service: MatrixService::new(),
            current_screen: Screen::default(),
            homeserver_select_screen: HomeserverSelectState::default(),
            login_screen: LoginScreenState::default(),
            chat_screen: ChatScreenState::default(),
        }
    }
    pub fn set_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }
}
