mod version;

use clap::{Parser, Subcommand};
use octo::Result;

pub use version::VersionCommand;

#[derive(Parser)]
#[command(name = "octo")]
#[command(about = "A CHIP-8 emulator written in Rust")]
#[command(version)]
pub struct Cli {
    /// Disable memory write protection for interpreter area (0x000-0x1FF)
    /// WARNING: This allows potentially unsafe memory writes
    #[arg(long, global = true)]
    pub disable_write_protection: bool,

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

/// Global CLI options available to all commands
#[derive(Parser)]
pub struct GlobalOptions {
    /// Disable memory write protection for interpreter area (0x000-0x1FF)
    /// WARNING: This allows potentially unsafe memory writes
    #[arg(long, global = true)]
    pub disable_write_protection: bool,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Version(cmd) => cmd.execute(),
        }
    }
}
