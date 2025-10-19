use color_eyre::{Result, eyre::eyre};
use matrix_sdk::Client;

use crate::models::user::User;

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
}
