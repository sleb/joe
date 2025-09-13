mod version;

use clap::{Parser, Subcommand};

pub use version::VersionCommand;

#[derive(Parser)]
#[command(name = "octo")]
#[command(about = "A CHIP-8 emulator written in Rust")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show version information
    Version(VersionCommand),
    // Future commands:
    // /// Run a CHIP-8 ROM file
    // Run(RunCommand),
    // /// Run a ROM with debugging features
    // Debug(DebugCommand),
    // /// Show information about a ROM file
    // Info(InfoCommand),
    // /// Run built-in tests
    // Test(TestCommand),
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Version(cmd) => cmd.execute(),
        }
    }
}
