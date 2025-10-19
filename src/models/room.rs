use matrix_sdk::ruma::OwnedRoomId;

#[derive(Debug, Clone)]
pub struct Room {
    pub room_id: OwnedRoomId,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub last_message: Option<String>,
    pub unread_count: u64,
    pub is_encrypted: bool,
    pub is_direct: bool,
}

impl Room {
    pub fn new(room_id: OwnedRoomId) -> Self {
        Self {
            room_id,
            display_name: None,
            avatar_url: None,
            last_message: None,
            unread_count: 0,
            is_encrypted: false,
            is_direct: false,
        }
    }

    pub fn get_display_name(&self) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| self.room_id.to_string())
    }
}
