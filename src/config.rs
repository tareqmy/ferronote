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
            notes_dir: home.join(".ferronote"),
        }
    }
}

impl Config {
    /// # Errors
    /// Returns an error if it fails to read or write the config file.
    pub fn load() -> Result<Self> {
        let default_config = Self::default();
        let config_path = default_config.notes_dir.join("config.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Self = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            default_config.save()?;
            Ok(default_config)
        }
    }

    /// # Errors
    /// Returns an error if it fails to write the config file.
    pub fn save(&self) -> Result<()> {
        if !self.notes_dir.exists() {
            std::fs::create_dir_all(&self.notes_dir)?;
        }

        let config_path = self.notes_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}
