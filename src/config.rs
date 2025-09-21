//! Configuration Management
//!
//! Handles loading, saving, and managing user configuration files
//! stored in OS-appropriate directories.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Could not determine user configuration directory")]
    NoConfigDir,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

/// User configuration for the emulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Emulator settings
    pub emulator: EmulatorSettings,

    /// Display settings
    pub display: DisplaySettings,

    /// Input settings
    pub input: InputSettings,
}

/// Emulator-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorSettings {
    /// Maximum number of CPU cycles to execute (0 = unlimited)
    pub max_cycles: usize,

    /// Delay between CPU cycles in milliseconds
    pub cycle_delay_ms: u64,

    /// Enable verbose output
    pub verbose: bool,

    /// Enable memory write protection
    pub write_protection: bool,
}
/// Display-specific settings for ratatui renderer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// Character to use for pixels in ratatui rendering
    pub pixel_char: String,

    /// Pixel color theme (Green, White, Blue, etc.)
    pub pixel_color: String,

    /// Refresh rate in milliseconds for the display
    pub refresh_rate_ms: u64,

    /// Theme name for the overall UI
    pub theme: String,
}

/// Input-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    /// Custom key mappings (CHIP-8 key -> keyboard key)
    pub key_mappings: std::collections::HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut key_mappings = std::collections::HashMap::new();

        // Default CHIP-8 to keyboard mappings
        key_mappings.insert("0".to_string(), "X".to_string());
        key_mappings.insert("1".to_string(), "1".to_string());
        key_mappings.insert("2".to_string(), "2".to_string());
        key_mappings.insert("3".to_string(), "3".to_string());
        key_mappings.insert("4".to_string(), "Q".to_string());
        key_mappings.insert("5".to_string(), "W".to_string());
        key_mappings.insert("6".to_string(), "E".to_string());
        key_mappings.insert("7".to_string(), "A".to_string());
        key_mappings.insert("8".to_string(), "S".to_string());
        key_mappings.insert("9".to_string(), "D".to_string());
        key_mappings.insert("A".to_string(), "Z".to_string());
        key_mappings.insert("B".to_string(), "C".to_string());
        key_mappings.insert("C".to_string(), "4".to_string());
        key_mappings.insert("D".to_string(), "R".to_string());
        key_mappings.insert("E".to_string(), "F".to_string());
        key_mappings.insert("F".to_string(), "V".to_string());

        Self {
            emulator: EmulatorSettings {
                max_cycles: 0,
                cycle_delay_ms: 16,
                verbose: false,
                write_protection: true,
            },
            display: DisplaySettings {
                pixel_char: "██".to_string(),
                pixel_color: "Green".to_string(),
                refresh_rate_ms: 16,
                theme: "Default".to_string(),
            },
            input: InputSettings { key_mappings },
        }
    }
}

/// Configuration manager for handling config files
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self, ConfigError> {
        let proj_dirs = ProjectDirs::from("com", "sleb", "joe").ok_or(ConfigError::NoConfigDir)?;

        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("config.toml");

        // Ensure the config directory exists
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        Ok(Self { config_path })
    }

    /// Get the path to the configuration file
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Load configuration from file, creating default if it doesn't exist
    pub fn load(&self) -> Result<Config, ConfigError> {
        if !self.config_path.exists() {
            let default_config = Config::default();
            self.save(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, config: &Config) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset(&self) -> Result<Config, ConfigError> {
        let default_config = Config::default();
        self.save(&default_config)?;
        Ok(default_config)
    }

    /// Check if configuration file exists
    pub fn exists(&self) -> bool {
        self.config_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.emulator.max_cycles, 0);
        assert_eq!(config.emulator.cycle_delay_ms, 16);
        assert!(!config.emulator.verbose);
        assert!(config.emulator.write_protection);

        assert_eq!(config.display.pixel_char, "█");
        assert_eq!(config.display.pixel_color, "Green");
        assert_eq!(config.display.refresh_rate_ms, 16);
        assert_eq!(config.display.theme, "Default");

        assert!(!config.input.key_mappings.is_empty());
        assert_eq!(config.input.key_mappings.get("0"), Some(&"X".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.emulator.max_cycles, deserialized.emulator.max_cycles);
        assert_eq!(config.display.pixel_char, deserialized.display.pixel_char);
        assert_eq!(config.display.theme, deserialized.display.theme);
    }

    #[test]
    fn test_config_manager_creation() {
        // This test might fail in some CI environments without home directories
        if env::var("HOME").is_ok() || env::var("USERPROFILE").is_ok() {
            let manager = ConfigManager::new();
            assert!(manager.is_ok());
        }
    }
}
