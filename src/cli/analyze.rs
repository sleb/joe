use clap::Parser;
use joe::{Memory, Result, analyze_instruction_usage, disassemble_rom, print_disassembly};
use std::path::PathBuf;

#[derive(Parser)]
pub struct AnalyzeCommand {
    /// Path to the ROM file to analyze
    #[arg(value_name = "ROM_FILE")]
    pub rom_file: PathBuf,

    /// Show detailed disassembly (default: summary only)
    #[arg(short, long)]
    pub disassemble: bool,

    /// Show instruction usage statistics
    #[arg(short, long)]
    pub stats: bool,
}

impl AnalyzeCommand {
    pub fn execute(self, disable_write_protection: bool) -> Result<()> {
        // Load ROM file
        let rom_data = std::fs::read(&self.rom_file).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read ROM file '{}': {}",
                self.rom_file.display(),
                e
            )
        })?;

        println!("Analyzing ROM: {}", self.rom_file.display());
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
