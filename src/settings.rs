use color_eyre::eyre::{Result, eyre};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    username: String,
}

impl Settings {
    fn config_path() -> PathBuf {
        config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clitrix/config.toml")
    }

    fn load(&self) -> Result<Self> {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            toml::from_str(&data).map_err(|e| eyre!("Failed to parse settings: {}", e))
        } else {
            Err(eyre!("Settings file not found"))
        }
    }

    fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or(());
        }
        let data = toml::to_string(self)?;
        fs::write(path, data)?;
        Ok(())
    }
}
