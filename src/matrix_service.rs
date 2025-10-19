use color_eyre::{Result, eyre::eyre};
use matrix_sdk::{
    Client, RoomState,
    ruma::{OwnedRoomId, events::room::message::RoomMessageEventContent},
};

use crate::models::{message::Message, room::Room, user::User};

#[derive(Debug, Clone)]
pub struct MatrixService {
    pub homeserver_url: Option<String>,
    pub user: Option<User>,
    pub client: Option<Client>,
}

impl MatrixService {
    pub fn new() -> Self {
        Self {
            homeserver_url: None,
            user: None,
            client: None,
        }
    }

    pub fn set_homeserver(&mut self, homeserver: &str) {
        let homeserver_url =
            if homeserver.starts_with("http://") || homeserver.starts_with("https://") {
                homeserver.to_string()
            } else {
                format!("https://{}", homeserver)
            };
        self.homeserver_url = Some(homeserver_url);
    }

    pub async fn check_homeserver(homeserver: &str) -> Result<()> {
        if homeserver.is_empty() {
            return Err(eyre!("Homeserver cannot be empty"));
        }

        let homeserver_url =
            if homeserver.starts_with("http://") || homeserver.starts_with("https://") {
                homeserver.to_string()
            } else {
                format!("https://{}", homeserver)
            };

        match Client::builder()
            .homeserver_url(homeserver_url)
            .build()
            .await
        {
            Ok(client) => {
                // Try to get the server's capabilities or well-known info
                match client.get_capabilities().await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let error_msg = e.to_string().to_lowercase();

                        // These errors indicate a valid Matrix homeserver
                        if error_msg.contains("no access token")
                            || error_msg.contains("auth")
                            || error_msg.contains("unauthorized")
                            || error_msg.contains("403")
                            || error_msg.contains("401")
                            || error_msg.contains("404")
                            || error_msg.contains("not found")
                            || error_msg.contains("capabilities")
                        {
                            Ok(()) // Server exists and is a valid Matrix homeserver
                        } else if error_msg.contains("connection")
                            || error_msg.contains("timeout")
                            || error_msg.contains("network")
                            || error_msg.contains("dns")
                        {
                            Err(eyre!("Homeserver not reachable: network error"))
                        } else {
                            Err(eyre!("Homeserver validation failed: {}", e))
                        }
                    }
                }
            }
            Err(e) => Err(eyre!("Failed to create client for homeserver: {}", e)),
        }
    }
    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let homeserver_url = match &self.homeserver_url {
            Some(url) => url,
            None => return Err(eyre!("Homeserver URL is not set")),
        };

        let client = Client::builder()
            .homeserver_url(homeserver_url)
            .build()
            .await
            .map_err(|e| eyre!("Failed to create Matrix client: {}", e))?;

        let response = client
            .matrix_auth()
            .login_username(username, password)
            .initial_device_display_name("Clitrix")
            .await?;

        self.user = Some(User {
            user_id: response.user_id.to_string(),
            device_id: response.device_id,
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_in: response.expires_in,
            display_name: None,
            avatar_url: None,
        });

        self.client = Some(client);

        Ok(())
    }

    pub async fn get_rooms(&self) -> Result<Vec<Room>> {
        let client = match &self.client {
            Some(client) => client,
            None => return Err(eyre!("Client is not initialized")),
        };

        let mut rooms = Vec::new();

        for room in client.rooms() {
            if room.state() == RoomState::Joined {
                let room_id = room.room_id().to_owned();
                let display_name = room.display_name().await.ok().map(|n| n.to_string());
                let is_direct = room.is_direct().await.ok().unwrap_or(false);

                // Get last message (simplified)
                let last_message = None;

                let unread_count = room
                    .unread_notification_counts()
                    .notification_count
                    .try_into()
                    .unwrap_or(0);

                rooms.push(Room {
                    room_id,
                    display_name,
                    avatar_url: None,
                    last_message,
                    unread_count,
                    is_encrypted: false,
                    is_direct,
                });
            }
        }

        Ok(rooms)
    }

    pub async fn sync_once(&self) -> Result<()> {
        let client = match &self.client {
            Some(client) => client,
            None => return Err(eyre!("Client is not initialized")),
        };

        client.sync_once(Default::default()).await?;
        Ok(())
    }

    pub async fn get_messages(&self, room_id: &OwnedRoomId, _limit: u32) -> Result<Vec<Message>> {
        let client = match &self.client {
            Some(client) => client,
            None => return Err(eyre!("Client is not initialized")),
        };

        let _room = client
            .get_room(room_id)
            .ok_or_else(|| eyre!("Room not found"))?;

        // TODO: Implement proper message fetching using Matrix SDK timeline API
        // For now, return empty messages list
        // The timeline API requires more complex setup with proper sync and state management

        let messages = Vec::new();

        Ok(messages)
    }

    pub async fn send_message(&self, room_id: &OwnedRoomId, message: &str) -> Result<()> {
        let client = match &self.client {
            Some(client) => client,
            None => return Err(eyre!("Client is not initialized")),
        };

        let room = client
            .get_room(room_id)
            .ok_or_else(|| eyre!("Room not found"))?;

        let content = RoomMessageEventContent::text_plain(message);
        room.send(content).await?;

        Ok(())
    }
}
