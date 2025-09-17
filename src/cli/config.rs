//! Config Command
//!
//! Manages user configuration files for the CHIP-8 emulator.

use anyhow::Result;
use clap::{Args, Subcommand};
use joe::{Config, ConfigManager};
use std::process::Command;

/// Configuration management command
#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

/// Configuration management subcommands
#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// Initialize configuration file with defaults
    Init {
        /// Overwrite existing configuration file
        #[clap(long)]
        force: bool,
    },

    /// Show current configuration
    Show,

    /// Show configuration file path
    Path,

    /// Edit configuration file in default editor
    Edit,

    /// Reset configuration to defaults
    Reset {
        /// Skip confirmation prompt
        #[clap(short, long)]
        yes: bool,
    },
}

impl ConfigCommand {
    /// Execute the config command
    pub fn execute(&self) -> Result<()> {
        let config_manager = ConfigManager::new()?;

        match &self.command {
            ConfigSubcommand::Init { force } => {
                if config_manager.exists() && !force {
                    eprintln!("Configuration file already exists at: {}", config_manager.config_path().display());
                    eprintln!("Use --force to overwrite, or 'joe config show' to view current config");
                    return Ok(());
                }

                let config = Config::default();
                config_manager.save(&config)?;

                println!("✅ Configuration initialized at: {}", config_manager.config_path().display());
                println!("Use 'joe config edit' to customize settings");
            }

            ConfigSubcommand::Show => {
                let config = config_manager.load()?;
                let toml_str = toml::to_string_pretty(&config)?;
                println!("{}", toml_str);
            }

            ConfigSubcommand::Path => {
                println!("{}", config_manager.config_path().display());
            }

            ConfigSubcommand::Edit => {
                // Ensure config file exists
                if !config_manager.exists() {
                    let config = Config::default();
                    config_manager.save(&config)?;
                    println!("✅ Created new configuration file");
                }

                let config_path = config_manager.config_path();

                // Try to open with default editor
                let editor = std::env::var("EDITOR")
                    .or_else(|_| std::env::var("VISUAL"))
                    .unwrap_or_else(|_| {
                        if cfg!(target_os = "windows") {
                            "notepad".to_string()
                        } else if cfg!(target_os = "macos") {
                            "open".to_string()
                        } else {
                            "nano".to_string()
                        }
                    });

                let mut cmd = Command::new(&editor);

                // Special handling for macOS 'open' command
                if editor == "open" {
                    cmd.arg("-t");
                }

                cmd.arg(config_path);

                match cmd.status() {
                    Ok(status) => {
                        if status.success() {
                            println!("✅ Configuration file edited");
                        } else {
                            eprintln!("❌ Editor exited with non-zero status");
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to open editor '{}': {}", editor, e);
                        eprintln!("💡 Try setting EDITOR environment variable or edit manually:");
                        eprintln!("   {}", config_path.display());
                    }
                }
            }

            ConfigSubcommand::Reset { yes } => {
                if !yes {
                    print!("⚠️  This will reset your configuration to defaults. Continue? [y/N] ");
                    use std::io::{self, Write};
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;

                    if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                        println!("❌ Reset cancelled");
                        return Ok(());
                    }
                }

                config_manager.reset()?;
                println!("✅ Configuration reset to defaults");
                println!("📍 Config file: {}", config_manager.config_path().display());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_command_creation() {
        // Test that config subcommands can be created
        let init_cmd = ConfigSubcommand::Init { force: false };
        let show_cmd = ConfigSubcommand::Show;
        let path_cmd = ConfigSubcommand::Path;
        let edit_cmd = ConfigSubcommand::Edit;
        let reset_cmd = ConfigSubcommand::Reset { yes: true };

        // Just verify they can be constructed
        match init_cmd {
            ConfigSubcommand::Init { force } => assert!(!force),
            _ => panic!("Wrong variant"),
        }

        match show_cmd {
            ConfigSubcommand::Show => {},
            _ => panic!("Wrong variant"),
        }

        match path_cmd {
            ConfigSubcommand::Path => {},
            _ => panic!("Wrong variant"),
        }

        match edit_cmd {
            ConfigSubcommand::Edit => {},
            _ => panic!("Wrong variant"),
        }

        match reset_cmd {
            ConfigSubcommand::Reset { yes } => assert!(yes),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_config_show_execution() {
        // This test might fail in some CI environments without home directories
        if env::var("HOME").is_ok() || env::var("USERPROFILE").is_ok() {
            let cmd = ConfigSubcommand::Show;
            // Just verify it doesn't panic - actual execution would require filesystem access
            let _ = format!("{:?}", cmd);
        }
    }
}