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
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }

    fn print_detailed_version(&self) {
        println!("{} - CHIP-8 Emulator", env!("CARGO_PKG_NAME"));
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
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
        let git_dirty = env!("BUILD_GIT_DIRTY");

        if git_hash != "unknown" {
            println!(
                "  Hash: {}{}",
                git_hash,
                if git_dirty == "true" { " (dirty)" } else { "" }
            );
            println!("  Branch: {}", git_branch);

            let git_tag = env!("BUILD_GIT_TAG");
            if !git_tag.is_empty() {
                println!("  Latest Tag: {}", git_tag);

                // Version consistency check
                let tag_version = env!("BUILD_TAG_VERSION");
                let cargo_version = env!("CARGO_PKG_VERSION");
                if tag_version == cargo_version {
                    println!("  Version Status: ✅ In sync with tag");
                } else if !tag_version.is_empty() {
                    println!(
                        "  Version Status: ⚠️  Cargo.toml ({}) differs from tag ({})",
                        cargo_version, tag_version
                    );
                }
            } else {
                println!("  Latest Tag: (none)");
                println!(
                    "  Version Status: 💡 Ready to tag v{}",
                    env!("CARGO_PKG_VERSION")
                );
            }
        } else {
            println!("  Status: Not in git repository");
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
