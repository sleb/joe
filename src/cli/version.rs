use clap::Args;

#[derive(Args)]
pub struct VersionCommand {
    /// Show detailed build information
    #[arg(long, short = 'd')]
    pub detailed: bool,
}

impl VersionCommand {
    pub fn execute(self) -> anyhow::Result<()> {
        if self.detailed {
            self.print_detailed_version();
        } else {
            self.print_simple_version();
        }
        Ok(())
    }

    fn print_simple_version(&self) {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("BUILD_VERSION"));
    }

    fn print_detailed_version(&self) {
        println!("{} - CHIP-8 Emulator", env!("CARGO_PKG_NAME"));
        println!("Version: {}", env!("BUILD_VERSION"));

        let is_release = env!("BUILD_IS_RELEASE") == "true";
        if is_release {
            println!("Release: ✅ Official release");
        } else {
            let commits_since = env!("BUILD_COMMITS_SINCE_TAG");
            println!(
                "Release: 🚧 Development build ({} commits since tag)",
                commits_since
            );
        }

        println!("Full Version: {}", env!("BUILD_VERSION_FULL"));
        println!("Authors: {}", env!("CARGO_PKG_AUTHORS"));
        println!("License: {}", env!("CARGO_PKG_LICENSE"));
        println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
        println!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));
        println!();

        // Build information from our build script
        println!("Build Information:");
        println!("  Build Time: {}", env!("BUILD_TIMESTAMP"));
        println!("  Target: {}", env!("BUILD_TARGET"));
        println!("  Profile: {}", env!("BUILD_PROFILE"));
        println!();

        // Git information from our build script
        println!("Git Information:");
        let git_hash = env!("BUILD_GIT_HASH");
        let git_branch = env!("BUILD_GIT_BRANCH");
        let git_dirty = env!("BUILD_GIT_DIRTY") == "true";

        println!(
            "  Hash: {}{}",
            git_hash,
            if git_dirty { " (dirty)" } else { "" }
        );
        println!("  Branch: {}", git_branch);

        let base_tag = env!("BUILD_BASE_TAG");
        if !base_tag.is_empty() {
            println!("  Base Tag: {}", base_tag);

            if is_release {
                println!("  Status: 🎯 Built from exact tag");
            } else {
                println!("  Status: 🔧 Development build from tag");
            }
        } else {
            println!("  Base Tag: (none)");
            println!("  Status: ⚠️ No version tag found");
        }
        println!();

        // System information
        println!("System Information:");
        println!("  OS: {}", std::env::consts::OS);
        println!("  Architecture: {}", std::env::consts::ARCH);
        println!(
            "  Rust Version: {}",
            option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("unknown")
        );
        println!();

        println!("Supported Features:");
        println!("  - CHIP-8 instruction set (35 opcodes)");
        println!("  - 64x32 monochrome display");
        println!("  - 16-key input system");
        println!("  - Audio/beep support");
        println!("  - ROM loading (.ch8 files)");
    }
}
