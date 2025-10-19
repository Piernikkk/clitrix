use chrono::{DateTime, Local};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub presence: UserPresence,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserPresence {
    Online,
    Offline,
    Away,
    Busy,
    Unknown,
}

impl User {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            display_name: None,
            avatar_url: None,
            presence: UserPresence::Unknown,
        }
    }

    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.user_id)
    }

    pub fn presence_icon(&self) -> &'static str {
        match self.presence {
            UserPresence::Online => "🟢",
            UserPresence::Away => "🟡",
            UserPresence::Busy => "🔴",
            UserPresence::Offline => "⚫",
            UserPresence::Unknown => "⚪",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Room {
    pub room_id: String,
    pub display_name: Option<String>,
    pub topic: Option<String>,
    pub avatar_url: Option<String>,
    pub member_count: usize,
    pub unread_count: usize,
    pub last_message: Option<Message>,
    pub room_type: RoomType,
    pub is_encrypted: bool,
    pub is_direct: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomType {
    DirectMessage,
    PublicRoom,
    PrivateRoom,
    Space,
}

impl Room {
    pub fn new(room_id: String, room_type: RoomType) -> Self {
        Self {
            room_id,
            display_name: None,
            topic: None,
            avatar_url: None,
            member_count: 0,
            unread_count: 0,
            last_message: None,
            is_encrypted: false,
            is_direct: room_type == RoomType::DirectMessage,
            room_type,
        }
    }

    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.room_id)
    }

    pub fn type_icon(&self) -> &'static str {
        match self.room_type {
            RoomType::DirectMessage => "💬",
            RoomType::PublicRoom => "🏛️",
            RoomType::PrivateRoom => "🔒",
            RoomType::Space => "🌌",
        }
    }

    pub fn encryption_icon(&self) -> &'static str {
        if self.is_encrypted { "🔐" } else { "" }
    }

    pub fn unread_indicator(&self) -> String {
        if self.unread_count > 0 {
            format!(" ({})", self.unread_count)
        } else {
            String::new()
        }
    }

    pub fn last_message_preview(&self) -> String {
        if let Some(ref msg) = self.last_message {
            let content = match &msg.content {
                MessageContent::Text { body } => {
                    if body.len() > 50 {
                        format!("{}...", &body[..47])
                    } else {
                        body.clone()
                    }
                }
                MessageContent::Image { .. } => "📷 Image".to_string(),
                MessageContent::File { filename, .. } => format!("📎 {}", filename),
                MessageContent::Audio { .. } => "🎵 Audio".to_string(),
                MessageContent::Video { .. } => "🎥 Video".to_string(),
                MessageContent::Emote { body } => format!("* {}", body),
                MessageContent::Notice { body } => format!("ℹ️ {}", body),
                MessageContent::System { message } => format!("🔧 {}", message),
            };

            let sender_name = msg.sender.display_name();
            let time = msg.timestamp.format("%H:%M");

            format!("{}: {} ({})", sender_name, content, time)
        } else {
            "No messages".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub event_id: String,
    pub sender: User,
    pub content: MessageContent,
    pub timestamp: DateTime<Local>,
    pub is_edited: bool,
    pub reply_to: Option<String>, // event_id of the message being replied to
    pub reactions: HashMap<String, Vec<String>>, // emoji -> list of user_ids
}

#[derive(Debug, Clone)]
pub enum MessageContent {
    Text {
        body: String,
    },
    Image {
        body: String,
        url: String,
        thumbnail_url: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    },
    File {
        body: String,
        filename: String,
        url: String,
        size: Option<u64>,
        mimetype: Option<String>,
    },
    Audio {
        body: String,
        url: String,
        duration: Option<u64>,
    },
    Video {
        body: String,
        url: String,
        thumbnail_url: Option<String>,
        duration: Option<u64>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Emote {
        body: String,
    },
    Notice {
        body: String,
    },
    System {
        message: String,
    },
}

impl Message {
    pub fn new_text(event_id: String, sender: User, body: String) -> Self {
        Self {
            event_id,
            sender,
            content: MessageContent::Text { body },
            timestamp: Local::now(),
            is_edited: false,
            reply_to: None,
            reactions: HashMap::new(),
        }
    }

    pub fn new_system(event_id: String, message: String) -> Self {
        Self {
            event_id,
            sender: User::new("@system:matrix.org".to_string()),
            content: MessageContent::System { message },
            timestamp: Local::now(),
            is_edited: false,
            reply_to: None,
            reactions: HashMap::new(),
        }
    }

    pub fn content_text(&self) -> String {
        match &self.content {
            MessageContent::Text { body } => body.clone(),
            MessageContent::Image { body, .. } => format!("📷 {}", body),
            MessageContent::File { filename, .. } => format!("📎 {}", filename),
            MessageContent::Audio { body, .. } => format!("🎵 {}", body),
            MessageContent::Video { body, .. } => format!("🎥 {}", body),
            MessageContent::Emote { body } => format!("* {}", body),
            MessageContent::Notice { body } => format!("ℹ️ {}", body),
            MessageContent::System { message } => format!("🔧 {}", message),
        }
    }

    pub fn is_from_user(&self, user_id: &str) -> bool {
        self.sender.user_id == user_id
    }

    pub fn add_reaction(&mut self, emoji: String, user_id: String) {
        self.reactions
            .entry(emoji)
            .or_insert_with(Vec::new)
            .push(user_id);
    }

    pub fn remove_reaction(&mut self, emoji: &str, user_id: &str) {
        if let Some(users) = self.reactions.get_mut(emoji) {
            users.retain(|id| id != user_id);
            if users.is_empty() {
                self.reactions.remove(emoji);
            }
        }
    }

    pub fn format_reactions(&self) -> String {
        if self.reactions.is_empty() {
            String::new()
        } else {
            let mut reactions = Vec::new();
            for (emoji, users) in &self.reactions {
                reactions.push(format!("{} {}", emoji, users.len()));
            }
            format!(" [{}]", reactions.join(" "))
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypingIndicator {
    pub users: Vec<User>,
    pub room_id: String,
}

impl TypingIndicator {
    pub fn new(room_id: String) -> Self {
        Self {
            users: Vec::new(),
            room_id,
        }
    }

    pub fn add_user(&mut self, user: User) {
        if !self.users.iter().any(|u| u.user_id == user.user_id) {
            self.users.push(user);
        }
    }

    pub fn remove_user(&mut self, user_id: &str) {
        self.users.retain(|u| u.user_id != user_id);
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    pub fn format_typing_text(&self) -> String {
        match self.users.len() {
            0 => String::new(),
            1 => format!("{} is typing...", self.users[0].display_name()),
            2 => format!(
                "{} and {} are typing...",
                self.users[0].display_name(),
                self.users[1].display_name()
            ),
            n => format!(
                "{}, {} and {} others are typing...",
                self.users[0].display_name(),
                self.users[1].display_name(),
                n - 2
            ),
        }
    }
}

// Convenience types for collections
pub type RoomList = Vec<Room>;
pub type MessageList = Vec<Message>;
pub type UserList = Vec<User>;

// Error types for Matrix operations
#[derive(Debug, Clone)]
pub enum MatrixError {
    NetworkError(String),
    AuthenticationError(String),
    RoomNotFound(String),
    MessageSendFailed(String),
    Unknown(String),
}

impl std::fmt::Display for MatrixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatrixError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            MatrixError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            MatrixError::RoomNotFound(room_id) => write!(f, "Room not found: {}", room_id),
            MatrixError::MessageSendFailed(msg) => write!(f, "Failed to send message: {}", msg),
            MatrixError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for MatrixError {}
