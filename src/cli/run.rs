use clap::Parser;
use joe::{AsciiRenderer, Cpu, Display, Memory, Renderer, RomSource, load_rom_data};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Parser)]
pub struct RunCommand {
    /// Path to the ROM file to run, or HTTP(S) URL to download ROM from
    /// Examples:
    ///   - Local file: roms/game.ch8
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

    /// Use headless mode (no display output)
    #[arg(long)]
    pub headless: bool,

    /// Show only final display state instead of continuous updates
    #[arg(long)]
    pub final_only: bool,
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
            Box::new(joe::HeadlessRenderer)
        } else {
            Box::new(AsciiRenderer)
        };

        // Run the emulator
        println!("\nStarting emulation...");
        if self.verbose {
            println!("Verbose mode enabled - showing CPU state each cycle");
        }
        if self.max_cycles > 0 {
            println!(
                "Max cycles: {}, Cycle delay: {}ms",
                self.max_cycles, self.cycle_delay_ms
            );
        } else {
            println!(
                "Running indefinitely, Cycle delay: {}ms",
                self.cycle_delay_ms
            );
        }
        println!("Press Ctrl+C to stop\n");

        // Set up Ctrl+C handler
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl+C handler");

        let mut cycles = 0;
        let cycle_delay = Duration::from_millis(self.cycle_delay_ms);
        let mut last_display_hash = 0u64;
        let mut last_render_time = Instant::now();
        let min_render_interval = Duration::from_millis(100); // Max 10 FPS for display updates

        loop {
            // Check if user pressed Ctrl+C
            if !running.load(Ordering::SeqCst) {
                println!("\nReceived Ctrl+C, stopping...");
                break;
            }
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
                    // Check for max cycles limit (if set)
                    if self.max_cycles > 0 && cycles >= self.max_cycles {
                        println!("Reached maximum cycles ({}), stopping", self.max_cycles);
                        break;
                    }

                    // Smart display rendering: only update if display changed or enough time passed
                    if !self.headless && !self.final_only {
                        let stats = display.get_stats();
                        let current_hash = stats.pixels_on as u64; // Simple hash based on pixel count
                        let now = Instant::now();

                        let display_changed = current_hash != last_display_hash;
                        let enough_time_passed =
                            now.duration_since(last_render_time) >= min_render_interval;

                        if display_changed || enough_time_passed {
                            // Clear screen and move cursor to top
                            print!("\x1B[2J\x1B[H");
                            println!("CHIP-8 Display (Cycle: {}):", cycles);
                            renderer.render(&display);

                            last_display_hash = current_hash;
                            last_render_time = now;
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

        // Show final results and statistics
        self.show_final_statistics(cycles, &cpu, &display, renderer.as_ref());
        Ok(())
    }

    /// Show final statistics and display state
    fn show_final_statistics(
        &self,
        cycles: usize,
        cpu: &Cpu,
        display: &Display,
        renderer: &dyn Renderer,
    ) {
        println!("\nEmulation completed after {} cycles", cycles);

        // Only show final display if we're in final-only mode (user hasn't seen it yet)
        // In continuous mode, the final display is already visible above
        if !self.headless && self.final_only {
            println!("\nFinal Display Output:");
            println!("{}", "=".repeat(70));
            renderer.render(display);
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
            if let Ok(value) = cpu.get_register(i)
                && value != 0
            {
                println!("    V{}: 0x{:02X}", i, value);
            }
        }

        if cpu.get_delay_timer() > 0 {
            println!("    Delay Timer: {}", cpu.get_delay_timer());
        }
        if cpu.get_sound_timer() > 0 {
            println!("    Sound Timer: {}", cpu.get_sound_timer());
        }

        println!("\nROM execution complete!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_signal_handler_setup() {
        // Test that we can create and use AtomicBool for signal handling
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        // Simulate signal handler behavior
        assert!(running.load(Ordering::SeqCst));
        r.store(false, Ordering::SeqCst);
        assert!(!running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_run_command_creation() {
        // Test that RunCommand can be created with default values
        let cmd = RunCommand {
            rom_source: "test.ch8".to_string(),
            max_cycles: 100,
            cycle_delay_ms: 16,
            verbose: false,
            headless: true,
            final_only: false,
        };

        assert_eq!(cmd.max_cycles, 100);
        assert_eq!(cmd.cycle_delay_ms, 16);
        assert!(!cmd.verbose);
        assert!(cmd.headless);
        assert!(!cmd.final_only);
    }
}
