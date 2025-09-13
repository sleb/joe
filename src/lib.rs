//! OCTO - CHIP-8 Emulator Library
//!
//! A CHIP-8 emulator implementation in Rust providing core emulation functionality.
//! This library can be used standalone or with the CLI frontend.
//!
//! # Architecture
//!
//! The emulator is built with a modular architecture:
//! - [`Memory`] - 4KB RAM with font data and ROM loading
//! - [`Cpu`] - Instruction execution and register management (TODO)
//! - [`Display`] - 64x32 framebuffer with sprite operations (TODO)
//! - [`Input`] - 16-key keypad handling (TODO)
//! - [`Audio`] - Sound timer and beep generation (TODO)
//! - [`Emulator`] - Main coordination and timing (TODO)
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use octo::Memory;
//!
//! // Create memory system with write protection
//! let memory = Memory::new(true);
//!
//! // Load a ROM file
//! let rom_data = std::fs::read("game.ch8").unwrap();
//! let mut memory = Memory::new(true);
//! memory.load_rom(&rom_data).unwrap();
//!
//! // TODO: Create and run emulator when implemented
//! // use octo::Emulator;
//! // let mut emulator = Emulator::new(memory);
//! // emulator.run();
//! ```
//!
//! # Memory Layout
//!
//! CHIP-8 uses a 4KB memory layout:
//! ```text
//! 0x000-0x1FF: CHIP-8 interpreter area
//! 0x050-0x0A0: Built-in font set (0-F)
//! 0x200-0xFFF: Program ROM and work RAM
//! ```
//!
//! # Features
//!
//! - Authentic CHIP-8 instruction set (35 opcodes)
//! - 64x32 monochrome display with XOR sprite drawing
//! - 16-key hexadecimal keypad input
//! - Sound timer with beep generation
//! - ROM loading with size validation
//! - Memory write protection (configurable)
//! - Comprehensive error handling

pub mod memory;

// TODO: Add these modules as we implement them
// pub mod cpu;
// pub mod display;
// pub mod input;
// pub mod audio;
// pub mod emulator;

// Re-export main types for convenience
pub use memory::{Memory, MemoryError, MemoryStats};

// TODO: Re-export other main types as we add them
// pub use cpu::{Cpu, CpuError};
// pub use display::{Display, DisplayError};
// pub use emulator::{Emulator, EmulatorError};

/// Result type alias using anyhow for convenience
pub type Result<T> = anyhow::Result<T>;

/// CHIP-8 system constants
pub mod constants {
    /// Total memory size (4KB)
    pub const MEMORY_SIZE: usize = 4096;

    /// Font data starts at this address
    pub const FONT_START_ADDR: u16 = 0x050;

    /// Programs load starting at this address
    pub const PROGRAM_START_ADDR: u16 = 0x200;

    /// Display width in pixels
    pub const DISPLAY_WIDTH: usize = 64;

    /// Display height in pixels
    pub const DISPLAY_HEIGHT: usize = 32;

    /// Number of CPU registers (V0-VF)
    pub const NUM_REGISTERS: usize = 16;

    /// Stack depth (16 levels)
    pub const STACK_SIZE: usize = 16;

    /// Number of keys on keypad
    pub const NUM_KEYS: usize = 16;

    /// Target CPU frequency (Hz)
    pub const CPU_FREQUENCY: u32 = 500;

    /// Timer frequency (Hz)
    pub const TIMER_FREQUENCY: u32 = 60;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(true);
        let stats = memory.get_stats();

        assert_eq!(stats.total_size, constants::MEMORY_SIZE);
        assert_eq!(stats.font_start, constants::FONT_START_ADDR);
        assert_eq!(stats.program_start, constants::PROGRAM_START_ADDR);
        assert!(stats.write_protection_enabled);
    }

    #[test]
    fn test_constants_consistency() {
        // Ensure our constants make sense
        assert!(constants::FONT_START_ADDR < constants::PROGRAM_START_ADDR);
        assert!(constants::PROGRAM_START_ADDR < constants::MEMORY_SIZE as u16);
        assert!(constants::DISPLAY_WIDTH > 0);
        assert!(constants::DISPLAY_HEIGHT > 0);
        assert_eq!(constants::NUM_REGISTERS, 16);
        assert_eq!(constants::NUM_KEYS, 16);
    }
}
