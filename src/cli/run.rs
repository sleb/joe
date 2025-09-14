use clap::Parser;
use octo::{AsciiRenderer, Cpu, Display, Memory, Renderer};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
pub struct RunCommand {
    /// Path to the ROM file to run
    #[arg(value_name = "ROM_FILE")]
    pub rom_file: PathBuf,

    /// Maximum number of CPU cycles to execute (prevents infinite loops)
    #[arg(short = 'c', long, default_value = "1000")]
    pub max_cycles: usize,

    /// Delay between CPU cycles in milliseconds
    #[arg(short = 'd', long, default_value = "100")]
    pub cycle_delay_ms: u64,

    /// Show CPU state after each cycle
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Use headless mode (no display output)
    #[arg(long)]
    pub headless: bool,
}

impl RunCommand {
    pub fn execute(self, disable_write_protection: bool) -> octo::Result<()> {
        println!("CHIP-8 Emulator - Running ROM");
        println!("==============================");

        // Check if ROM file exists
        if !self.rom_file.exists() {
            anyhow::bail!("ROM file '{}' not found", self.rom_file.display());
        }

        // Load ROM file
        let rom_data = std::fs::read(&self.rom_file)
            .map_err(|e| anyhow::anyhow!("Failed to read ROM file: {}", e))?;

        println!(
            "Loaded ROM: {} ({} bytes)",
            self.rom_file.display(),
            rom_data.len()
        );

        // Initialize emulator components
        let write_protection = !disable_write_protection;
        let mut memory = Memory::new(write_protection);
        let mut cpu = Cpu::new();
        let mut display = Display::new();

        // Load ROM into memory
        memory.load_rom(&rom_data)?;
        println!("ROM loaded at address 0x{:04X}", 0x200);

        // Choose renderer based on headless flag
        let renderer: Box<dyn Renderer> = if self.headless {
            println!("Running in headless mode (no display output)");
            Box::new(octo::HeadlessRenderer)
        } else {
            Box::new(AsciiRenderer)
        };

        // Run the emulator
        println!("\nStarting emulation...");
        if self.verbose {
            println!("Verbose mode enabled - showing CPU state each cycle");
        }
        println!(
            "Max cycles: {}, Cycle delay: {}ms",
            self.max_cycles, self.cycle_delay_ms
        );
        println!("Press Ctrl+C to stop\n");

        let mut cycles = 0;
        let cycle_delay = Duration::from_millis(self.cycle_delay_ms);

        loop {
            cycles += 1;

            if self.verbose {
                println!(
                    "Cycle {}: PC=0x{:04X}, I=0x{:04X}",
                    cycles,
                    cpu.get_pc(),
                    cpu.get_index()
                );
            }

            // Execute one CPU cycle
            match cpu.execute_cycle(&mut memory, &mut display) {
                Ok(()) => {
                    // Check for end conditions
                    if cycles >= self.max_cycles {
                        println!("Reached maximum cycles ({}), stopping", self.max_cycles);
                        break;
                    }

                    // Check for infinite loops (common pattern in CHIP-8 programs)
                    if cycles > 10 {
                        let pc = cpu.get_pc();
                        // Simple infinite loop detection - if PC hasn't changed for several cycles
                        // This is a heuristic and could be improved
                        static mut LAST_PC: u16 = 0;
                        static mut SAME_PC_COUNT: usize = 0;

                        unsafe {
                            if pc == LAST_PC {
                                SAME_PC_COUNT += 1;
                                if SAME_PC_COUNT > 5 {
                                    println!("Detected infinite loop at PC=0x{:04X}, stopping", pc);
                                    break;
                                }
                            } else {
                                SAME_PC_COUNT = 0;
                            }
                            LAST_PC = pc;
                        }
                    }
                }
                Err(e) => {
                    println!("Execution error at cycle {}: {}", cycles, e);
                    break;
                }
            }

            // Add delay between cycles
            if self.cycle_delay_ms > 0 {
                std::thread::sleep(cycle_delay);
            }
        }

        // Show final results
        println!("\nEmulation completed after {} cycles", cycles);

        if !self.headless {
            println!("\nFinal Display Output:");
            println!("{}", "=".repeat(70));
            renderer.render(&display);
            println!("{}", "=".repeat(70));
        }

        // Show statistics
        let stats = display.get_stats();
        println!("\nStatistics:");
        println!("  Cycles executed: {}", cycles);
        println!(
            "  Display pixels on: {}/{} ({}%)",
            stats.pixels_on,
            stats.pixels_total,
            if stats.pixels_total > 0 {
                (stats.pixels_on * 100) / stats.pixels_total
            } else {
                0
            }
        );

        println!("  Final CPU state:");
        println!("    PC: 0x{:04X}", cpu.get_pc());
        println!("    I:  0x{:04X}", cpu.get_index());

        // Show a few registers
        for i in 0..4 {
            if let Ok(value) = cpu.get_register(i) {
                if value != 0 {
                    println!("    V{}: 0x{:02X}", i, value);
                }
            }
        }

        if cpu.get_delay_timer() > 0 {
            println!("    Delay Timer: {}", cpu.get_delay_timer());
        }
        if cpu.get_sound_timer() > 0 {
            println!("    Sound Timer: {}", cpu.get_sound_timer());
        }

        println!("\nROM execution complete!");
        Ok(())
    }
}
