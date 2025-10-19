use crate::{
    data::{Message, Room, User},
    ui::{
        components::{StatefulMessageList, StatefulRoomList},
        input_handler::TextInputState,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum ChatFocus {
    RoomList,
    MessageList,
    MessageInput,
    RoomFilter,
}

impl Default for ChatFocus {
    fn default() -> Self {
        ChatFocus::RoomList
    }
}

#[derive(Debug)]
pub struct ChatScreenState {
    // Data
    pub rooms: Vec<Room>,
    pub messages: Vec<Message>,
    pub current_user: Option<User>,
    pub selected_room_id: Option<String>,

    // UI State
    pub room_list: StatefulRoomList,
    pub message_list: StatefulMessageList,
    pub message_input: TextInputState,
    pub filter_input: TextInputState,
    pub focus: ChatFocus,

    // Interaction state
    pub typing_users: Vec<String>,
    pub is_loading_messages: bool,
    pub is_sending_message: bool,
    pub pending_send: bool,
    pub error_message: Option<String>,

    // Settings
    pub show_timestamps: bool,
    pub show_presence: bool,
    pub auto_scroll: bool,
    pub room_list_width: u16,
}

impl Default for ChatScreenState {
    fn default() -> Self {
        Self {
            rooms: Vec::new(),
            messages: Vec::new(),
            current_user: None,
            selected_room_id: None,

            room_list: StatefulRoomList::new(),
            message_list: StatefulMessageList::new(),
            message_input: TextInputState::default(),
            filter_input: TextInputState::default(),
            focus: ChatFocus::default(),

            typing_users: Vec::new(),
            is_loading_messages: false,
            is_sending_message: false,
            pending_send: false,
            error_message: None,

            show_timestamps: true,
            show_presence: true,
            auto_scroll: true,
            room_list_width: 30,
        }
    }
}

impl ChatScreenState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user(mut self, user: User) -> Self {
        self.current_user = Some(user);
        self
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            ChatFocus::RoomList => ChatFocus::MessageList,
            ChatFocus::MessageList => ChatFocus::MessageInput,
            ChatFocus::MessageInput => ChatFocus::RoomList,
            ChatFocus::RoomFilter => ChatFocus::RoomList,
        };
    }

    pub fn set_focus(&mut self, focus: ChatFocus) {
        self.focus = focus;
    }

    pub fn selected_room(&self) -> Option<&Room> {
        self.room_list.selected_room(&self.rooms)
    }

    pub fn selected_room_mut(&mut self) -> Option<&mut Room> {
        if let Some(index) = self.room_list.selected_room_index() {
            self.rooms.get_mut(index)
        } else {
            None
        }
    }

    pub fn current_user_id(&self) -> Option<&str> {
        self.current_user.as_ref().map(|u| u.user_id.as_str())
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);

        // Auto-scroll if enabled
        if self.auto_scroll {
            self.message_list.scroll_to_bottom(&self.messages, 10);
        }
    }

    pub fn update_room(&mut self, room_id: &str, updater: impl FnOnce(&mut Room)) {
        if let Some(room) = self.rooms.iter_mut().find(|r| r.room_id == room_id) {
            updater(room);
        }
    }

    pub fn set_loading_messages(&mut self, loading: bool) {
        self.is_loading_messages = loading;
    }

    pub fn set_sending_message(&mut self, sending: bool) {
        self.is_sending_message = sending;
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error;
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn add_typing_user(&mut self, user_id: String) {
        if !self.typing_users.contains(&user_id) {
            self.typing_users.push(user_id);
        }
    }

    pub fn remove_typing_user(&mut self, user_id: &str) {
        self.typing_users.retain(|id| id != user_id);
    }

    pub fn clear_typing_users(&mut self) {
        self.typing_users.clear();
    }

    pub fn prepare_message_send(&mut self) {
        if !self.message_input.value.trim().is_empty() {
            self.pending_send = true;
            self.is_sending_message = true;
        }
    }

    pub fn complete_message_send(&mut self) {
        self.message_input.clear();
        self.pending_send = false;
        self.is_sending_message = false;
        self.focus = ChatFocus::MessageList;
    }

    pub fn cancel_message_send(&mut self) {
        self.pending_send = false;
        self.is_sending_message = false;
    }

    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
        self.message_list.toggle_auto_scroll();
    }

    pub fn toggle_timestamps(&mut self) {
        self.show_timestamps = !self.show_timestamps;
    }

    pub fn toggle_presence(&mut self) {
        self.show_presence = !self.show_presence;
    }

    pub fn set_room_list_width(&mut self, width: u16) {
        self.room_list_width = width.max(20).min(60);
    }

    pub fn handle_new_room(&mut self, room: Room) {
        // Add room and sort by last message time or name
        self.rooms.push(room);
        self.sort_rooms();
    }

    pub fn handle_room_update(&mut self, room_id: &str, room: Room) {
        if let Some(index) = self.rooms.iter().position(|r| r.room_id == room_id) {
            self.rooms[index] = room;
            self.sort_rooms();
        }
    }

    pub fn handle_room_removal(&mut self, room_id: &str) {
        self.rooms.retain(|r| r.room_id != room_id);

        // If the removed room was selected, select another one
        if self.selected_room_id.as_deref() == Some(room_id) {
            if !self.rooms.is_empty() {
                self.room_list.select_first(&self.rooms);
                self.selected_room_id = self.selected_room().map(|r| r.room_id.clone());
            } else {
                self.selected_room_id = None;
                self.messages.clear();
            }
        }
    }

    fn sort_rooms(&mut self) {
        // Sort rooms by unread count (descending) then by last message time (descending)
        self.rooms
            .sort_by(|a, b| match b.unread_count.cmp(&a.unread_count) {
                std::cmp::Ordering::Equal => match (&b.last_message, &a.last_message) {
                    (Some(b_msg), Some(a_msg)) => b_msg.timestamp.cmp(&a_msg.timestamp),
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => a.display_name().cmp(b.display_name()),
                },
                other => other,
            });
    }

    pub fn get_status_text(&self) -> String {
        if let Some(ref error) = self.error_message {
            format!("Error: {}", error)
        } else if self.is_sending_message {
            "Sending message...".to_string()
        } else if self.is_loading_messages {
            "Loading messages...".to_string()
        } else if let Some(room) = self.selected_room() {
            if self.typing_users.is_empty() {
                format!("In {} • {} members", room.display_name(), room.member_count)
            } else {
                match self.typing_users.len() {
                    1 => format!("{} is typing...", self.typing_users[0]),
                    2 => format!(
                        "{} and {} are typing...",
                        self.typing_users[0], self.typing_users[1]
                    ),
                    n => format!(
                        "{}, {} and {} others are typing...",
                        self.typing_users[0],
                        self.typing_users[1],
                        n - 2
                    ),
                }
            }
        } else {
            "Select a room to start chatting".to_string()
        }
    }

    pub fn get_input_placeholder(&self) -> &'static str {
        match self.focus {
            ChatFocus::MessageInput => {
                if self.selected_room().is_some() {
                    "Type your message... (Enter to send, Esc to cancel)"
                } else {
                    "Select a room first"
                }
            }
            ChatFocus::RoomFilter => "Filter rooms... (Enter to apply, Esc to cancel)",
            _ => "",
        }
    }

    pub fn can_send_message(&self) -> bool {
        !self.message_input.value.trim().is_empty()
            && self.selected_room().is_some()
            && !self.is_sending_message
    }

    pub fn unread_message_count(&self) -> usize {
        self.rooms.iter().map(|r| r.unread_count).sum()
    }

    pub fn filtered_room_count(&self) -> usize {
        self.rooms.len() // This would be calculated based on current filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{RoomType, User, UserPresence};

    fn create_test_room(id: &str, name: &str) -> Room {
        let mut room = Room::new(id.to_string(), RoomType::DirectMessage);
        room.display_name = Some(name.to_string());
        room
    }

    fn create_test_user(id: &str, name: &str) -> User {
        User {
            user_id: id.to_string(),
            display_name: Some(name.to_string()),
            avatar_url: None,
            presence: UserPresence::Online,
        }
    }

    #[test]
    fn test_chat_state_creation() {
        let state = ChatScreenState::new();
        assert_eq!(state.focus, ChatFocus::RoomList);
        assert!(state.rooms.is_empty());
        assert!(state.messages.is_empty());
        assert!(state.show_timestamps);
    }

    #[test]
    fn test_focus_cycling() {
        let mut state = ChatScreenState::new();

        assert_eq!(state.focus, ChatFocus::RoomList);
        state.cycle_focus();
        assert_eq!(state.focus, ChatFocus::MessageList);
        state.cycle_focus();
        assert_eq!(state.focus, ChatFocus::MessageInput);
        state.cycle_focus();
        assert_eq!(state.focus, ChatFocus::RoomList);
    }

    #[test]
    fn test_typing_users() {
        let mut state = ChatScreenState::new();

        state.add_typing_user("@alice:matrix.org".to_string());
        assert_eq!(state.typing_users.len(), 1);

        state.add_typing_user("@bob:matrix.org".to_string());
        assert_eq!(state.typing_users.len(), 2);

        state.remove_typing_user("@alice:matrix.org");
        assert_eq!(state.typing_users.len(), 1);
        assert_eq!(state.typing_users[0], "@bob:matrix.org");

        state.clear_typing_users();
        assert!(state.typing_users.is_empty());
    }

    #[test]
    fn test_room_management() {
        let mut state = ChatScreenState::new();
        let room1 = create_test_room("!room1:matrix.org", "Room 1");
        let room2 = create_test_room("!room2:matrix.org", "Room 2");

        state.handle_new_room(room1);
        state.handle_new_room(room2);
        assert_eq!(state.rooms.len(), 2);

        state.handle_room_removal("!room1:matrix.org");
        assert_eq!(state.rooms.len(), 1);
        assert_eq!(state.rooms[0].room_id, "!room2:matrix.org");
    }

    #[test]
    fn test_message_sending() {
        let mut state = ChatScreenState::new();
        state.message_input.set_value("Hello world".to_string());

        assert!(state.can_send_message()); // Would be false without selected room

        state.prepare_message_send();
        assert!(state.pending_send);
        assert!(state.is_sending_message);

        state.complete_message_send();
        assert!(!state.pending_send);
        assert!(!state.is_sending_message);
        assert!(state.message_input.is_empty());
        assert_eq!(state.focus, ChatFocus::MessageList);
    }

    #[test]
    fn test_status_text() {
        let mut state = ChatScreenState::new();

        // No room selected
        assert_eq!(state.get_status_text(), "Select a room to start chatting");

        // Error state
        state.set_error(Some("Network error".to_string()));
        assert!(state.get_status_text().starts_with("Error:"));

        // Loading state
        state.clear_error();
        state.set_loading_messages(true);
        assert_eq!(state.get_status_text(), "Loading messages...");

        // Sending state
        state.set_loading_messages(false);
        state.set_sending_message(true);
        assert_eq!(state.get_status_text(), "Sending message...");
    }
}
