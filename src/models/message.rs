use chrono::{DateTime, Utc};
use matrix_sdk::ruma::OwnedUserId;

#[derive(Debug, Clone)]
pub struct Message {
    pub event_id: String,
    pub sender: OwnedUserId,
    pub sender_display_name: Option<String>,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
    pub is_own_message: bool,
}

#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(String),
    Emote(String),
    Notice(String),
    Image { body: String, url: String },
    File { body: String, url: String },
    Unknown,
}

impl Message {
    pub fn new(
        event_id: String,
        sender: OwnedUserId,
        sender_display_name: Option<String>,
        content: MessageContent,
        timestamp: DateTime<Utc>,
        is_own_message: bool,
    ) -> Self {
        Self {
            event_id,
            sender,
            sender_display_name,
            content,
            timestamp,
            is_own_message,
        }
    }

    pub fn get_sender_name(&self) -> String {
        self.sender_display_name
            .clone()
            .unwrap_or_else(|| self.sender.to_string())
    }

    pub fn get_text_content(&self) -> String {
        match &self.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Emote(text) => format!("* {}", text),
            MessageContent::Notice(text) => text.clone(),
            MessageContent::Image { body, .. } => format!("[Image: {}]", body),
            MessageContent::File { body, .. } => format!("[File: {}]", body),
            MessageContent::Unknown => "[Unknown message type]".to_string(),
        }
    }
}
