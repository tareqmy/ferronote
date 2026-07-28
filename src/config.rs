use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_notes_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ferronote")
}

fn default_extension() -> String {
    "md".to_string()
}

fn default_auto_save_delay_ms() -> u64 {
    1000
}

fn default_tab_size() -> u8 {
    4
}

fn default_sidebar_width_percent() -> u16 {
    30
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_sort() -> String {
    "modified_desc".to_string()
}

fn default_auto_purge_days() -> u32 {
    30
}

/// User preferences and application configuration persisted in `config.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// Directory where Markdown notes are stored.
    #[serde(default = "default_notes_dir")]
    pub notes_dir: PathBuf,

    /// Default file extension for newly created notes (e.g. `md`).
    #[serde(default = "default_extension")]
    pub default_extension: String,

    /// Delay in milliseconds before auto-saving editor changes.
    #[serde(default = "default_auto_save_delay_ms")]
    pub auto_save_delay_ms: u64,

    /// Tab character width in spaces.
    #[serde(default = "default_tab_size")]
    pub tab_size: u8,

    /// Sidebar note list width as a percentage of total width.
    #[serde(default = "default_sidebar_width_percent")]
    pub sidebar_width_percent: u16,

    /// UI theme palette name (`default`, `gruvbox`, `nord`, `dracula`).
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Default note list sorting mode (`modified_desc`, `title_asc`, `created_desc`).
    #[serde(default = "default_sort")]
    pub default_sort: String,

    /// Number of days before soft-deleted trash items are auto-purged.
    #[serde(default = "default_auto_purge_days")]
    pub auto_purge_days: u32,
}


impl Default for Config {
    fn default() -> Self {
        Self {
            notes_dir: default_notes_dir(),
            default_extension: default_extension(),
            auto_save_delay_ms: default_auto_save_delay_ms(),
            tab_size: default_tab_size(),
            sidebar_width_percent: default_sidebar_width_percent(),
            theme: default_theme(),
            default_sort: default_sort(),
            auto_purge_days: default_auto_purge_days(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.default_extension, "md");
        assert_eq!(config.auto_save_delay_ms, 1000);
        assert_eq!(config.tab_size, 4);
        assert_eq!(config.sidebar_width_percent, 30);
        assert_eq!(config.theme, "default");
        assert_eq!(config.default_sort, "modified_desc");
        assert_eq!(config.auto_purge_days, 30);
    }

    #[test]
    fn test_partial_json_deserialization() {
        let partial_json = r#"{"notes_dir": "/tmp/test"}"#;
        let config: Config = serde_json::from_str(partial_json).unwrap();
        assert_eq!(config.notes_dir, PathBuf::from("/tmp/test"));
        assert_eq!(config.default_extension, "md");
        assert_eq!(config.tab_size, 4);
    }

    #[test]
    fn test_config_save_and_reload() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.notes_dir = temp_dir.path().to_path_buf();
        config.theme = "nord".to_string();
        config.tab_size = 2;

        config.save().unwrap();

        let config_path = temp_dir.path().join("config.json");
        assert!(config_path.exists());

        let loaded_json = std::fs::read_to_string(config_path).unwrap();
        let reloaded_config: Config = serde_json::from_str(&loaded_json).unwrap();
        assert_eq!(reloaded_config.theme, "nord");
        assert_eq!(reloaded_config.tab_size, 2);
    }
}

