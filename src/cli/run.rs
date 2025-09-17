use clap::Parser;
use joe::{
    AsciiRenderer, Emulator, EmulatorConfig, Renderer, RomSource, load_rom_data,
};

#[derive(Parser)]
pub struct RunCommand {
    /// Path to the ROM file to run, or HTTP(S) URL to download ROM from
    /// Examples:
    ///   - Local file: game.ch8
    ///   - Remote URL: https://example.com/rom.ch8
    #[arg(value_name = "ROM_SOURCE")]
    pub rom_source: String,

    /// Maximum number of CPU cycles to execute (0 = unlimited)
    #[arg(short = 'c', long, default_value = "0")]
    pub max_cycles: usize,

    /// Delay between CPU cycles in milliseconds (16ms ≈ 60fps)
    #[arg(short = 'd', long, default_value = "16")]
    pub cycle_delay_ms: u64,

    /// Show CPU state after each cycle
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

impl RunCommand {
    pub fn execute(self, disable_write_protection: bool) -> joe::Result<()> {
        println!("CHIP-8 Emulator - Running ROM");
        println!("==============================");

        // Detect source type and load ROM data
        let source = RomSource::from_string(&self.rom_source);

        println!(
            "Loading ROM from {}: {}",
            if source.is_url() { "URL" } else { "file" },
            source.description()
        );

        if source.is_url() {
            println!("Downloading ROM from remote server...");
        }

        // Load ROM data (from file or URL)
        let rom_data = load_rom_data(&self.rom_source)?;

        println!(
            "Loaded ROM: {} ({} bytes)",
            source.description(),
            rom_data.len()
        );

        // Configure the emulator
        let config = EmulatorConfig {
            max_cycles: self.max_cycles,
            cycle_delay_ms: self.cycle_delay_ms,
            verbose: self.verbose,
            write_protection: !disable_write_protection,
        };

        // Create and initialize emulator
        let mut emulator = Emulator::new(config);

        // Load ROM into emulator
        emulator.load_rom(&rom_data)?;
        println!("ROM loaded at address 0x{:04X}", 0x200);

        // Use ASCII renderer only
        let renderer: Box<dyn Renderer> = Box::new(AsciiRenderer);

        // Run the emulator
        emulator.run(renderer.as_ref())?;
        Ok(())
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_creation() {
        // Test that RunCommand can be created with default values
        let cmd = RunCommand {
            rom_source: "test.ch8".to_string(),
            max_cycles: 100,
            cycle_delay_ms: 16,
            verbose: false,
        };

        assert_eq!(cmd.max_cycles, 100);
        assert_eq!(cmd.cycle_delay_ms, 16);
        assert!(!cmd.verbose);
    }

    #[test]
    fn test_emulator_config_creation() {
        // Test that we can create EmulatorConfig from RunCommand parameters
        let cmd = RunCommand {
            rom_source: "test.ch8".to_string(),
            max_cycles: 200,
            cycle_delay_ms: 8,
            verbose: true,
        };

        let config = EmulatorConfig {
            max_cycles: cmd.max_cycles,
            cycle_delay_ms: cmd.cycle_delay_ms,
            verbose: cmd.verbose,
            write_protection: true,
        };

        assert_eq!(config.max_cycles, 200);
        assert_eq!(config.cycle_delay_ms, 8);
        assert!(config.verbose);
        assert!(config.write_protection);
    }
}
