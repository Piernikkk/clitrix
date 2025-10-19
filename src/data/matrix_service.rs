use crate::data::models::*;

use color_eyre::eyre::{Result, eyre};
use matrix_sdk::{Client, ServerName};
use tokio::time::{Duration, sleep};
use url::Url;

/// Dummy Matrix service that provides mock data and placeholder functions
/// Replace these implementations with actual Matrix SDK calls
#[derive(Debug, Clone)]
pub struct MatrixService {
    pub current_user: Option<User>,
    pub is_authenticated: bool,
    pub sync_token: Option<String>,
}

impl Default for MatrixService {
    fn default() -> Self {
        Self {
            current_user: None,
            is_authenticated: false,
            sync_token: None,
        }
    }
}

impl MatrixService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn check_homeserver(&self, homeserver: &str) -> Result<bool> {
        if homeserver.is_empty() {
            return Err(eyre!("Homeserver cannot be empty"));
        }

        // Normalize the homeserver URL - construct proper URL format
        let homeserver_url =
            if homeserver.starts_with("http://") || homeserver.starts_with("https://") {
                homeserver.to_string()
            } else {
                format!("https://{}", homeserver)
            };

        // Parse and validate the URL first
        let parsed_url = match Url::parse(&homeserver_url) {
            Ok(url) => url,
            Err(_) => return Err(eyre!("Invalid homeserver URL format")),
        };

        // Extract host for ServerName validation
        let host_with_port = if let Some(port) = parsed_url.port() {
            format!("{}:{}", parsed_url.host_str().unwrap_or(""), port)
        } else {
            parsed_url.host_str().unwrap_or("").to_string()
        };

        // Parse the server name using matrix-sdk
        let server_name = match ServerName::parse(&host_with_port) {
            Ok(name) => name,
            Err(_) => return Err(eyre!("Invalid Matrix homeserver format")),
        };

        // Create a temporary client to test the homeserver
        match Client::builder()
            .homeserver_url(homeserver_url)
            .build()
            .await
        {
            Ok(client) => {
                // Try to get the server's capabilities or well-known info
                match client.get_capabilities().await {
                    Ok(_) => Ok(true),
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
                            Ok(true) // Server exists and is a valid Matrix homeserver
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

    /// Dummy login function - replace with actual Matrix SDK authentication
    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
        homeserver: &str,
    ) -> Result<User, MatrixError> {
        // Simulate network delay
        sleep(Duration::from_millis(500)).await;

        // Mock validation
        if username.is_empty() || password.is_empty() {
            return Err(MatrixError::AuthenticationError(
                "Username and password cannot be empty".to_string(),
            ));
        }

        if !homeserver.contains('.') {
            return Err(MatrixError::AuthenticationError(
                "Invalid homeserver URL".to_string(),
            ));
        }

        // Simulate authentication success
        let user_id = if username.starts_with('@') {
            username.to_string()
        } else {
            format!("@{}:{}", username, homeserver)
        };

        let user = User {
            user_id: user_id.clone(),
            display_name: Some(username.to_string()),
            avatar_url: None,
            presence: UserPresence::Online,
        };

        self.current_user = Some(user.clone());
        self.is_authenticated = true;
        self.sync_token = Some("dummy_sync_token".to_string());

        Ok(user)
    }

    /// Dummy logout function
    pub async fn logout(&mut self) -> Result<(), MatrixError> {
        sleep(Duration::from_millis(200)).await;

        self.current_user = None;
        self.is_authenticated = false;
        self.sync_token = None;

        Ok(())
    }

    /// Dummy function to get room list - replace with Matrix SDK room fetching
    pub async fn get_rooms(&self) -> Result<RoomList, MatrixError> {
        if !self.is_authenticated {
            return Err(MatrixError::AuthenticationError(
                "Not authenticated".to_string(),
            ));
        }

        sleep(Duration::from_millis(300)).await;

        // Create dummy rooms
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

        Ok(rooms)
    }

    /// Dummy function to get messages for a room - replace with Matrix SDK message fetching
    pub async fn get_room_messages(&self, room_id: &str) -> Result<MessageList, MatrixError> {
        if !self.is_authenticated {
            return Err(MatrixError::AuthenticationError(
                "Not authenticated".to_string(),
            ));
        }

        sleep(Duration::from_millis(200)).await;

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

                let you = self.current_user.as_ref().unwrap().clone();

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

        Ok(messages)
    }

    /// Dummy function to send a message - replace with Matrix SDK message sending
    pub async fn send_message(&self, room_id: &str, content: &str) -> Result<String, MatrixError> {
        if !self.is_authenticated {
            return Err(MatrixError::AuthenticationError(
                "Not authenticated".to_string(),
            ));
        }

        if content.trim().is_empty() {
            return Err(MatrixError::MessageSendFailed(
                "Message content cannot be empty".to_string(),
            ));
        }

        // Simulate network delay
        sleep(Duration::from_millis(100)).await;

        // Generate dummy event ID
        let event_id = format!("$dummy_{}_{}", room_id, chrono::Local::now().timestamp());

        Ok(event_id)
    }

    /// Dummy function to start typing indicator - replace with Matrix SDK typing
    pub async fn start_typing(&self, room_id: &str) -> Result<(), MatrixError> {
        if !self.is_authenticated {
            return Err(MatrixError::AuthenticationError(
                "Not authenticated".to_string(),
            ));
        }

        // Simulate API call
        sleep(Duration::from_millis(50)).await;

        println!("Started typing in room: {}", room_id); // Debug output
        Ok(())
    }

    /// Dummy function to stop typing indicator - replace with Matrix SDK typing
    pub async fn stop_typing(&self, room_id: &str) -> Result<(), MatrixError> {
        if !self.is_authenticated {
            return Err(MatrixError::AuthenticationError(
                "Not authenticated".to_string(),
            ));
        }

        // Simulate API call
        sleep(Duration::from_millis(50)).await;

        println!("Stopped typing in room: {}", room_id); // Debug output
        Ok(())
    }

    /// Dummy function to get room members - replace with Matrix SDK member fetching
    pub async fn get_room_members(&self, room_id: &str) -> Result<UserList, MatrixError> {
        if !self.is_authenticated {
            return Err(MatrixError::AuthenticationError(
                "Not authenticated".to_string(),
            ));
        }

        sleep(Duration::from_millis(150)).await;

        let mut members = Vec::new();

        // Add current user
        if let Some(ref user) = self.current_user {
            members.push(user.clone());
        }

        // Add dummy members based on room
        match room_id {
            "!direct1:matrix.org" => {
                members.push(User {
                    user_id: "@alice:matrix.org".to_string(),
                    display_name: Some("Alice Smith".to_string()),
                    avatar_url: None,
                    presence: UserPresence::Online,
                });
            }
            "!rust:matrix.org" => {
                members.push(User {
                    user_id: "@rustdev:matrix.org".to_string(),
                    display_name: Some("RustDev".to_string()),
                    avatar_url: None,
                    presence: UserPresence::Online,
                });
                members.push(User {
                    user_id: "@rustfan:matrix.org".to_string(),
                    display_name: Some("RustFan".to_string()),
                    avatar_url: None,
                    presence: UserPresence::Away,
                });
            }
            _ => {
                // Generic members for other rooms
                members.push(User {
                    user_id: "@member1:matrix.org".to_string(),
                    display_name: Some("Member One".to_string()),
                    avatar_url: None,
                    presence: UserPresence::Online,
                });
            }
        }

        Ok(members)
    }

    /// Dummy function to sync with server - replace with Matrix SDK sync
    pub async fn sync(&mut self) -> Result<(), MatrixError> {
        if !self.is_authenticated {
            return Err(MatrixError::AuthenticationError(
                "Not authenticated".to_string(),
            ));
        }

        // Simulate sync delay
        sleep(Duration::from_millis(100)).await;

        // Update sync token
        self.sync_token = Some(format!("sync_token_{}", chrono::Local::now().timestamp()));

        Ok(())
    }

    /// Check if the service is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }

    /// Get the current user
    pub fn current_user(&self) -> Option<&User> {
        self.current_user.as_ref()
    }

    /// Get current user ID for convenience
    pub fn current_user_id(&self) -> Option<&str> {
        self.current_user.as_ref().map(|u| u.user_id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login_success() {
        let mut service = MatrixService::new();
        let result = service.login("testuser", "testpass", "matrix.org").await;

        assert!(result.is_ok());
        assert!(service.is_authenticated());
        assert!(service.current_user().is_some());
    }

    #[tokio::test]
    async fn test_login_failure() {
        let mut service = MatrixService::new();
        let result = service.login("", "", "").await;

        assert!(result.is_err());
        assert!(!service.is_authenticated());
    }

    #[tokio::test]
    async fn test_get_rooms_unauthenticated() {
        let service = MatrixService::new();
        let result = service.get_rooms().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_message() {
        let mut service = MatrixService::new();
        service
            .login("testuser", "testpass", "matrix.org")
            .await
            .unwrap();

        let result = service
            .send_message("!room:matrix.org", "Hello world!")
            .await;

        assert!(result.is_ok());
    }
}
