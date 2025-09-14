use clap::Parser;
use joe::{
    Memory, Result, RomSource, analyze_instruction_usage, disassemble_rom, load_rom_data,
    print_disassembly,
};

#[derive(Parser)]
pub struct AnalyzeCommand {
    /// Path to the ROM file to analyze, or HTTP(S) URL to download ROM from
    /// Examples:
    ///   - Local file: roms/game.ch8
    ///   - Remote URL: https://example.com/rom.ch8
    #[arg(value_name = "ROM_SOURCE")]
    pub rom_source: String,

    /// Show detailed disassembly (default: summary only)
    #[arg(short, long)]
    pub disassemble: bool,

    /// Show instruction usage statistics
    #[arg(short, long)]
    pub stats: bool,
}

impl AnalyzeCommand {
    pub fn execute(self, disable_write_protection: bool) -> Result<()> {
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

        println!("Analyzing ROM: {}", source.description());
        println!("ROM size: {} bytes", rom_data.len());

        // Create memory and load ROM
        let write_protection = !disable_write_protection;
        let mut memory = Memory::new(write_protection);
        memory.load_rom(&rom_data)?;

        // Disassemble the ROM
        let instructions =
            disassemble_rom(&memory).map_err(|e| anyhow::anyhow!("Disassembly failed: {}", e))?;

        if instructions.is_empty() {
            println!("No instructions found in ROM (empty or invalid)");
            return Ok(());
        }

        println!("Found {} instructions", instructions.len());

        // Show disassembly if requested
        if self.disassemble {
            println!("\nDisassembly:");
            println!("============");
            print_disassembly(&instructions);
        }

        // Show instruction analysis
        let analysis = analyze_instruction_usage(&instructions);

        // Always show summary unless user only wants disassembly
        if !self.disassemble || self.stats {
            analysis.print_summary();
        }

        Ok(())
    }
}
