pub mod chat_state;
pub mod room_chat_layout;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

use crate::{
    app::AppState,
    screens::{Screen, ScreenHandler},
};
use async_trait::async_trait;

pub use chat_state::{ChatFocus, ChatScreenState};
pub use room_chat_layout::RoomChatLayout;

pub struct ChatScreen;

impl ChatScreen {
    fn handle_room_list_keys(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app_state
                    .chat_state
                    .room_list
                    .select_previous(&app_state.chat_state.rooms);
                self.handle_room_selection_change(app_state);
                Some(Screen::Chat)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app_state
                    .chat_state
                    .room_list
                    .select_next(&app_state.chat_state.rooms);
                self.handle_room_selection_change(app_state);
                Some(Screen::Chat)
            }
            KeyCode::Enter => {
                app_state.chat_state.focus = ChatFocus::MessageInput;
                Some(Screen::Chat)
            }
            KeyCode::Char('u') => {
                app_state.chat_state.room_list.toggle_unread_filter();
                Some(Screen::Chat)
            }
            KeyCode::Char('/') => {
                app_state.chat_state.focus = ChatFocus::RoomFilter;
                Some(Screen::Chat)
            }
            _ => Some(Screen::Chat),
        }
    }

    fn handle_message_list_keys(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app_state.chat_state.message_list.scroll_up();
                Some(Screen::Chat)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app_state
                    .chat_state
                    .message_list
                    .scroll_down(&app_state.chat_state.messages, 10);
                Some(Screen::Chat)
            }
            KeyCode::PageUp => {
                app_state.chat_state.message_list.page_up(5);
                Some(Screen::Chat)
            }
            KeyCode::PageDown => {
                app_state
                    .chat_state
                    .message_list
                    .page_down(&app_state.chat_state.messages, 10, 5);
                Some(Screen::Chat)
            }
            KeyCode::Home => {
                app_state.chat_state.message_list.scroll_to_top();
                Some(Screen::Chat)
            }
            KeyCode::End => {
                app_state
                    .chat_state
                    .message_list
                    .scroll_to_bottom(&app_state.chat_state.messages, 10);
                Some(Screen::Chat)
            }
            KeyCode::Enter => {
                app_state.chat_state.focus = ChatFocus::MessageInput;
                Some(Screen::Chat)
            }
            _ => Some(Screen::Chat),
        }
    }

    fn handle_message_input_keys(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen> {
        match key.code {
            KeyCode::Enter if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Mark message for sending
                app_state.chat_state.pending_send = true;
                Some(Screen::Chat)
            }
            KeyCode::Esc => {
                app_state.chat_state.focus = ChatFocus::MessageList;
                Some(Screen::Chat)
            }
            _ => {
                // Handle text input
                app_state.chat_state.message_input.handle_key_event(key);
                Some(Screen::Chat)
            }
        }
    }

    fn handle_room_filter_keys(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen> {
        match key.code {
            KeyCode::Enter => {
                app_state
                    .chat_state
                    .room_list
                    .set_filter(app_state.chat_state.filter_input.value.clone());
                app_state.chat_state.focus = ChatFocus::RoomList;
                Some(Screen::Chat)
            }
            KeyCode::Esc => {
                app_state.chat_state.filter_input.clear();
                app_state.chat_state.room_list.clear_filter();
                app_state.chat_state.focus = ChatFocus::RoomList;
                Some(Screen::Chat)
            }
            _ => {
                app_state.chat_state.filter_input.handle_key_event(key);
                Some(Screen::Chat)
            }
        }
    }

    fn handle_room_selection_change(&self, app_state: &mut AppState) {
        let room_id = if let Some(room) = app_state
            .chat_state
            .room_list
            .selected_room(&app_state.chat_state.rooms)
        {
            room.room_id.clone()
        } else {
            return;
        };

        // Clear current messages and mark for loading
        app_state.chat_state.messages.clear();
        app_state.chat_state.message_input.clear();
        app_state.chat_state.selected_room_id = Some(room_id.clone());

        // In a real implementation, this would trigger async message loading
        // For now, we'll use dummy data
        self.load_dummy_messages_for_room(&room_id, app_state);
    }

    fn load_dummy_messages_for_room(&self, room_id: &str, app_state: &mut AppState) {
        use crate::data::{Message, User, UserPresence};

        let mut messages = Vec::new();

        // Create dummy messages based on room
        match room_id {
            "!direct1:matrix.org" => {
                let alice = User {
                    user_id: "@alice:matrix.org".to_string(),
                    display_name: Some("Alice Smith".to_string()),
                    avatar_url: None,
                    presence: UserPresence::Online,
                };

                let you = if let Some(ref user) = app_state.matrix_service.current_user {
                    user.clone()
                } else {
                    User {
                        user_id: "@you:matrix.org".to_string(),
                        display_name: Some("You".to_string()),
                        avatar_url: None,
                        presence: UserPresence::Online,
                    }
                };

                messages.push(Message::new_text(
                    "$msg1".to_string(),
                    alice.clone(),
                    "Hey! How's the new Matrix client coming along?".to_string(),
                ));

                messages.push(Message::new_text(
                    "$msg2".to_string(),
                    you.clone(),
                    "Pretty good! Just working on the UI now.".to_string(),
                ));

                messages.push(Message::new_text(
                    "$msg3".to_string(),
                    alice.clone(),
                    "That's awesome! Are you using Ratatui?".to_string(),
                ));

                messages.push(Message::new_text(
                    "$msg4".to_string(),
                    you.clone(),
                    "Yes! It's really nice for TUI development.".to_string(),
                ));

                messages.push(Message::new_text(
                    "$msg5".to_string(),
                    alice.clone(),
                    "I'd love to try it out when you're done! 🚀".to_string(),
                ));
            }
            "!rust:matrix.org" => {
                let rustdev = User {
                    user_id: "@rustdev:matrix.org".to_string(),
                    display_name: Some("RustDev".to_string()),
                    avatar_url: None,
                    presence: UserPresence::Online,
                };

                let rustfan = User {
                    user_id: "@rustfan:matrix.org".to_string(),
                    display_name: Some("RustFan".to_string()),
                    avatar_url: None,
                    presence: UserPresence::Online,
                };

                messages.push(Message::new_text(
                    "$msg6".to_string(),
                    rustdev.clone(),
                    "Anyone know how to handle async in Ratatui?".to_string(),
                ));

                messages.push(Message::new_text(
                    "$msg7".to_string(),
                    rustfan.clone(),
                    "You might want to check out tokio integration".to_string(),
                ));

                messages.push(Message::new_text(
                    "$msg8".to_string(),
                    rustdev.clone(),
                    "Thanks! I'll look into that.".to_string(),
                ));
            }
            _ => {
                // Generic messages for other rooms
                let user1 = User {
                    user_id: "@user1:matrix.org".to_string(),
                    display_name: Some("User One".to_string()),
                    avatar_url: None,
                    presence: UserPresence::Online,
                };

                messages.push(Message::new_text(
                    "$generic1".to_string(),
                    user1.clone(),
                    "Welcome to the room!".to_string(),
                ));

                messages.push(Message::new_system(
                    "$system1".to_string(),
                    format!("This is a dummy room: {}", room_id),
                ));
            }
        }

        app_state.chat_state.messages = messages;
        app_state
            .chat_state
            .message_list
            .scroll_to_bottom(&app_state.chat_state.messages, 10);
    }

    fn load_dummy_rooms(&self, app_state: &mut AppState) {
        use crate::data::{Message, Room, RoomType, User, UserPresence};

        let mut rooms = Vec::new();

        // Direct message rooms
        let mut dm1 = Room::new("!direct1:matrix.org".to_string(), RoomType::DirectMessage);
        dm1.display_name = Some("Alice Smith".to_string());
        dm1.member_count = 2;
        dm1.unread_count = 3;
        dm1.is_encrypted = true;
        dm1.last_message = Some(Message::new_text(
            "$event1".to_string(),
            User {
                user_id: "@alice:matrix.org".to_string(),
                display_name: Some("Alice Smith".to_string()),
                avatar_url: None,
                presence: UserPresence::Online,
            },
            "Hey! How's the Matrix client coming along?".to_string(),
        ));
        rooms.push(dm1);

        let mut dm2 = Room::new("!direct2:matrix.org".to_string(), RoomType::DirectMessage);
        dm2.display_name = Some("Bob Johnson".to_string());
        dm2.member_count = 2;
        dm2.unread_count = 0;
        dm2.is_encrypted = true;
        dm2.last_message = Some(Message::new_text(
            "$event2".to_string(),
            User {
                user_id: "@bob:matrix.org".to_string(),
                display_name: Some("Bob Johnson".to_string()),
                avatar_url: None,
                presence: UserPresence::Away,
            },
            "Sure, let's meet at 3 PM".to_string(),
        ));
        rooms.push(dm2);

        // Public rooms
        let mut public1 = Room::new("!rust:matrix.org".to_string(), RoomType::PublicRoom);
        public1.display_name = Some("Rust Programming".to_string());
        public1.topic = Some("Discussion about Rust programming language".to_string());
        public1.member_count = 1247;
        public1.unread_count = 12;
        public1.is_encrypted = false;
        public1.last_message = Some(Message::new_text(
            "$event3".to_string(),
            User {
                user_id: "@rustdev:matrix.org".to_string(),
                display_name: Some("RustDev".to_string()),
                avatar_url: None,
                presence: UserPresence::Online,
            },
            "Anyone know how to handle async in Ratatui?".to_string(),
        ));
        rooms.push(public1);

        let mut public2 = Room::new("!matrix-dev:matrix.org".to_string(), RoomType::PublicRoom);
        public2.display_name = Some("Matrix Development".to_string());
        public2.topic = Some("Matrix protocol and client development".to_string());
        public2.member_count = 892;
        public2.unread_count = 0;
        public2.is_encrypted = true;
        public2.last_message = Some(Message::new_text(
            "$event4".to_string(),
            User {
                user_id: "@matrixdev:matrix.org".to_string(),
                display_name: Some("Matrix Developer".to_string()),
                avatar_url: None,
                presence: UserPresence::Online,
            },
            "New Matrix SDK release is out!".to_string(),
        ));
        rooms.push(public2);

        // Private room
        let mut private1 = Room::new("!private1:matrix.org".to_string(), RoomType::PrivateRoom);
        private1.display_name = Some("Secret Project".to_string());
        private1.topic = Some("Top secret project discussion".to_string());
        private1.member_count = 5;
        private1.unread_count = 1;
        private1.is_encrypted = true;
        private1.last_message = Some(Message::new_text(
            "$event5".to_string(),
            User {
                user_id: "@lead:matrix.org".to_string(),
                display_name: Some("Project Lead".to_string()),
                avatar_url: None,
                presence: UserPresence::Busy,
            },
            "Meeting notes are uploaded".to_string(),
        ));
        rooms.push(private1);

        app_state.chat_state.rooms = rooms;

        // Select first room and load its messages
        if !app_state.chat_state.rooms.is_empty() {
            app_state
                .chat_state
                .room_list
                .select_first(&app_state.chat_state.rooms);

            let room_id = if let Some(room) = app_state
                .chat_state
                .room_list
                .selected_room(&app_state.chat_state.rooms)
            {
                room.room_id.clone()
            } else {
                return;
            };

            self.load_dummy_messages_for_room(&room_id, app_state);
        }
    }

    fn handle_pending_send(&self, app_state: &mut AppState) {
        if app_state.chat_state.pending_send && app_state.chat_state.can_send_message() {
            // Simulate sending message
            let content = app_state.chat_state.message_input.value.clone();

            if let Some(current_user) = app_state.matrix_service.current_user.clone() {
                let message = crate::data::Message::new_text(
                    format!("$sent_{}", chrono::Local::now().timestamp()),
                    current_user,
                    content,
                );

                app_state.chat_state.add_message(message);
                app_state.chat_state.complete_message_send();
            }
        }
    }
}

#[async_trait]
impl ScreenHandler for ChatScreen {
    fn render(&self, frame: &mut Frame, app_state: &AppState) {
        // Load dummy data if rooms are empty (first time)
        if app_state.chat_state.rooms.is_empty() {
            // This is a bit of a hack since we can't mutate app_state here
            // In a real implementation, this would be handled elsewhere
        }

        let layout = RoomChatLayout::new()
            .room_list_width(30)
            .show_room_filter(matches!(app_state.chat_state.focus, ChatFocus::RoomFilter));

        layout.render(frame, &app_state.chat_state);
    }

    async fn handle_key_event(
        &mut self,
        key: KeyEvent,
        app_state: &mut AppState,
    ) -> Option<Screen> {
        // Load dummy rooms on first access
        if app_state.chat_state.rooms.is_empty() {
            self.load_dummy_rooms(app_state);
        }

        // Handle pending message send
        self.handle_pending_send(app_state);

        // Global key bindings
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return None; // Quit application
            }
            KeyCode::Esc
                if !matches!(
                    app_state.chat_state.focus,
                    ChatFocus::MessageInput | ChatFocus::RoomFilter
                ) =>
            {
                return Some(Screen::Login); // Go back to login
            }
            KeyCode::Tab => {
                app_state.chat_state.cycle_focus();
                return Some(Screen::Chat);
            }
            _ => {}
        }

        // Handle keys based on current focus
        match app_state.chat_state.focus {
            ChatFocus::RoomList => self.handle_room_list_keys(key, app_state),
            ChatFocus::MessageList => self.handle_message_list_keys(key, app_state),
            ChatFocus::MessageInput => self.handle_message_input_keys(key, app_state),
            ChatFocus::RoomFilter => self.handle_room_filter_keys(key, app_state),
        }
    }
}
