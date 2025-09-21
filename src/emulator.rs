//! Emulator - Main CHIP-8 emulator coordination and execution
//!
//! The `Emulator` struct provides a high-level interface that encapsulates
//! all the core CHIP-8 components (CPU, Memory, Display, Input) and manages
//! their interactions. This simplifies usage and provides a clean API for
//! running CHIP-8 programs.

use crate::display::{ControlAction, RatatuiRenderer};
use crate::input::KeyEvent;
use crate::{Cpu, Display, Input, InputBus, Memory};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur during emulation
#[derive(Debug, Error)]
pub enum EmulatorError {
    #[error("CPU execution error: {0}")]
    Cpu(#[from] crate::cpu::CpuError),

    #[error("Memory error: {0}")]
    Memory(#[from] crate::memory::MemoryError),

    #[error("Display error: {0}")]
    Display(#[from] crate::display::DisplayError),

    #[error("Renderer error: {0}")]
    Renderer(#[from] crate::display::RendererError),

    #[error("Input error: {0}")]
    Input(#[from] crate::input::InputError),
}

/// Configuration options for the emulator
#[derive(Debug, Clone)]
pub struct EmulatorConfig {
    /// Maximum number of CPU cycles to execute (0 = unlimited)
    pub max_cycles: usize,

    /// Delay between CPU cycles in milliseconds
    pub cycle_delay_ms: u64,

    /// Show CPU state after each cycle
    pub verbose: bool,

    /// Enable memory write protection
    pub write_protection: bool,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            max_cycles: 0,
            cycle_delay_ms: 16, // ~60fps
            verbose: false,
            write_protection: true,
        }
    }
}

/// Statistics about emulator execution
#[derive(Debug, Clone)]
pub struct EmulatorStats {
    /// Total cycles executed
    pub cycles_executed: usize,

    /// Current CPU program counter
    pub program_counter: u16,

    /// Current CPU index register
    pub index_register: u16,

    /// Display statistics
    pub display_stats: crate::display::DisplayStats,

    /// Whether emulation is currently running
    pub is_running: bool,
}

/// Main CHIP-8 emulator that coordinates all components
pub struct Emulator {
    cpu: Cpu,
    memory: Memory,
    display: Display,
    input: Input,
    config: EmulatorConfig,
    cycles_executed: usize,
    is_running: Arc<AtomicBool>,
    last_display_hash: u64,
    last_render_time: Instant,
}

impl Emulator {
    /// Create a new emulator with the given configuration
    pub fn new(config: EmulatorConfig) -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(config.write_protection),
            display: Display::new(),
            input: Input::new(),
            config,
            cycles_executed: 0,
            is_running: Arc::new(AtomicBool::new(false)),
            last_display_hash: 0,
            last_render_time: Instant::now(),
        }
    }

    /// Create a new emulator with default configuration
    pub fn with_defaults() -> Self {
        Self::new(EmulatorConfig::default())
    }

    /// Load ROM data into the emulator's memory
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), EmulatorError> {
        self.memory.load_rom(rom_data)?;
        Ok(())
    }

    /// Start the emulation loop
    pub fn run(&mut self) -> Result<(), EmulatorError> {
        // Load user configuration
        let user_config = crate::config::ConfigManager::new()
            .and_then(|manager| manager.load())
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
                crate::config::Config::default()
            });

        // Create key event channel
        let (key_sender, key_receiver) = mpsc::channel::<KeyEvent>();

        // Create input system with channel receiver
        self.input = Input::with_key_receiver(key_receiver);

        // Create renderer with key sender
        let ratatui_config =
            crate::display::RatatuiConfig::from_display_settings(&user_config.display);
        let renderer = RatatuiRenderer::new_with_key_sender(ratatui_config, Some(key_sender))?;

        self.run_with_renderer(Some(renderer))
    }

    /// Run the emulator without terminal UI (headless mode)
    pub fn run_headless(&mut self) -> Result<(), EmulatorError> {
        self.run_with_renderer(None)
    }

    /// Core emulation loop with optional renderer
    fn run_with_renderer(
        &mut self,
        mut renderer: Option<RatatuiRenderer>,
    ) -> Result<(), EmulatorError> {
        self.is_running.store(true, Ordering::SeqCst);
        self.cycles_executed = 0;

        // Set up Ctrl+C handler
        let running = self.is_running.clone();
        ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl+C handler");

        let cycle_delay = Duration::from_millis(self.config.cycle_delay_ms);

        // Print appropriate startup message
        if renderer.is_some() {
            println!("Starting emulation...");
        } else {
            println!("Starting emulation in headless mode...");
        }

        if self.config.verbose {
            println!("Verbose mode enabled - showing CPU state each cycle");
        }
        if self.config.max_cycles > 0 {
            println!(
                "Max cycles: {}, Cycle delay: {}ms",
                self.config.max_cycles, self.config.cycle_delay_ms
            );
        } else {
            println!(
                "Running indefinitely, Cycle delay: {}ms",
                self.config.cycle_delay_ms
            );
        }
        println!("Press Ctrl+C to stop\n");

        loop {
            // Check if user pressed Ctrl+C
            if !self.is_running.load(Ordering::SeqCst) {
                println!("\nReceived Ctrl+C, stopping...");
                break;
            }

            self.cycles_executed += 1;

            if self.config.verbose {
                println!(
                    "Cycle {}: PC=0x{:04X}, I=0x{:04X}",
                    self.cycles_executed,
                    self.cpu.get_pc(),
                    self.cpu.get_index()
                );
            }

            // Poll input backend (only needed for renderer mode)
            if renderer.is_some() {
                self.input.update();
            }

            // Execute one CPU cycle
            match self
                .cpu
                .execute_cycle(&mut self.memory, &mut self.display, &mut self.input)
            {
                Ok(()) => {
                    // Check for max cycles limit (if set)
                    if self.config.max_cycles > 0 && self.cycles_executed >= self.config.max_cycles
                    {
                        println!(
                            "Reached maximum cycles ({}), stopping",
                            self.config.max_cycles
                        );
                        break;
                    }

                    // Handle display rendering and control actions (only if renderer exists)
                    if let Some(ref mut r) = renderer {
                        match r.render(&self.display, self.cycles_executed)? {
                            ControlAction::Quit => {
                                println!("\nReceived quit command, stopping...");
                                break;
                            }
                            ControlAction::Reset => {
                                println!("\nResetting emulator...");
                                self.reset();
                            }
                            ControlAction::TogglePause => {
                                // TODO: Implement pause functionality
                                println!("\nPause/Resume functionality not yet implemented");
                            }
                            ControlAction::None => {
                                // Continue normal execution
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Execution error at cycle {}: {}", self.cycles_executed, e);
                    return Err(EmulatorError::Cpu(e));
                }
            }

            // Add delay between cycles
            if self.config.cycle_delay_ms > 0 {
                std::thread::sleep(cycle_delay);
            }
        }

        self.is_running.store(false, Ordering::SeqCst);

        // Show final results and statistics
        self.show_final_statistics();
        Ok(())
    }

    /// Execute a single cycle without the full emulation loop
    pub fn step(&mut self) -> Result<(), EmulatorError> {
        self.input.update();
        self.cpu
            .execute_cycle(&mut self.memory, &mut self.display, &mut self.input)?;
        self.cycles_executed += 1;
        Ok(())
    }

    /// Get current emulator statistics
    pub fn get_stats(&self) -> EmulatorStats {
        EmulatorStats {
            cycles_executed: self.cycles_executed,
            program_counter: self.cpu.get_pc(),
            index_register: self.cpu.get_index(),
            display_stats: self.display.get_stats(),
            is_running: self.is_running.load(Ordering::SeqCst),
        }
    }

    /// Stop the emulation loop
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Get a reference to the display
    pub fn display(&self) -> &Display {
        &self.display
    }

    /// Get a reference to the CPU
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Get a reference to the memory
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// Get a reference to the input
    pub fn input(&self) -> &Input {
        &self.input
    }

    /// Reset the emulator to initial state
    pub fn reset(&mut self) {
        self.cpu = Cpu::new();
        self.memory = Memory::new(self.config.write_protection);
        self.display = Display::new();
        self.input = Input::new();
        self.cycles_executed = 0;
        self.is_running.store(false, Ordering::SeqCst);
        self.last_display_hash = 0;
        self.last_render_time = Instant::now();
    }

    /// Show final statistics and display state
    fn show_final_statistics(&self) {
        println!(
            "\nEmulation completed after {} cycles",
            self.cycles_executed
        );

        // Final display is already visible above from continuous updates

        // Show statistics
        let stats = self.display.get_stats();
        println!("\nStatistics:");
        println!("  Cycles executed: {}", self.cycles_executed);
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
        println!("    PC: 0x{:04X}", self.cpu.get_pc());
        println!("    I:  0x{:04X}", self.cpu.get_index());

        // Show a few registers
        for i in 0..4 {
            if let Ok(value) = self.cpu.get_register(i) {
                if value != 0 {
                    println!("    V{}: 0x{:02X}", i, value);
                }
            }
        }

        if self.cpu.get_delay_timer() > 0 {
            println!("    Delay Timer: {}", self.cpu.get_delay_timer());
        }
        if self.cpu.get_sound_timer() > 0 {
            println!("    Sound Timer: {}", self.cpu.get_sound_timer());
        }

        println!("\nROM execution complete!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_creation() {
        let emulator = Emulator::with_defaults();
        let stats = emulator.get_stats();

        assert_eq!(stats.cycles_executed, 0);
        assert_eq!(stats.program_counter, 0x200); // Default PC
        assert!(!stats.is_running);
    }

    #[test]
    fn test_emulator_config() {
        let config = EmulatorConfig {
            max_cycles: 100,
            cycle_delay_ms: 10,
            verbose: true,
            write_protection: false,
        };

        let emulator = Emulator::new(config.clone());
        assert_eq!(emulator.config.max_cycles, 100);
        assert_eq!(emulator.config.cycle_delay_ms, 10);
        assert!(emulator.config.verbose);
        assert!(!emulator.config.write_protection);
    }

    #[test]
    fn test_emulator_reset() {
        let mut emulator = Emulator::with_defaults();

        // Execute a few steps to change state
        let _ = emulator.step();
        let _ = emulator.step();

        // Reset should restore initial state
        emulator.reset();
        let stats = emulator.get_stats();

        assert_eq!(stats.cycles_executed, 0);
        assert!(!stats.is_running);
    }

    #[test]
    fn test_rom_loading() {
        let mut emulator = Emulator::with_defaults();
        let rom_data = vec![0xA2, 0x2A, 0x60, 0x0C]; // Simple test ROM

        assert!(emulator.load_rom(&rom_data).is_ok());
    }
}
