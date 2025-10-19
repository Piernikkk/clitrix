use matrix_sdk::ruma::{OwnedDeviceId, time::Duration};

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub device_id: OwnedDeviceId,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<Duration>,
}
