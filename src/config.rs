use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub notes_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            notes_dir: home.join("ferronotes"),
        }
    }
}

impl Config {
    /// # Errors
    /// Returns an error if it fails to read or write the config file.
    pub fn load() -> Result<Self> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("ferronote").join("config.json");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: Self = serde_json::from_str(&content)?;
                return Ok(config);
            }
        }

        Ok(Self::default())
    }

    /// # Errors
    /// Returns an error if it fails to write the config file.
    pub fn save(&self) -> Result<()> {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("ferronote");
            if !app_config_dir.exists() {
                std::fs::create_dir_all(&app_config_dir)?;
            }

            let config_path = app_config_dir.join("config.json");
            let content = serde_json::to_string_pretty(self)?;
            std::fs::write(config_path, content)?;
        }
        Ok(())
    }
}
