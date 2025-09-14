mod analyze;
mod run;
mod version;

use clap::{Parser, Subcommand};
use joe::Result;

pub use analyze::AnalyzeCommand;
pub use run::RunCommand;
pub use version::VersionCommand;

#[derive(Parser)]
#[command(name = "joe")]
#[command(about = "A CHIP-8 emulator written in Rust - tribute to Joseph Weisbecker")]
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
    /// Analyze a ROM file and show disassembly
    Analyze(AnalyzeCommand),
    /// Run a CHIP-8 ROM file
    Run(RunCommand),
    // Future commands:
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
            Commands::Analyze(cmd) => cmd.execute(self.disable_write_protection),
            Commands::Run(cmd) => cmd.execute(self.disable_write_protection),
        }
    }
}
