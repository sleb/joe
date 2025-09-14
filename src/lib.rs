//! JOE - CHIP-8 Emulator Library
//!
//! A working CHIP-8 emulator implementation in Rust providing core emulation functionality.
//! This library can be used standalone or with the CLI frontend.
//!
//! # Current Status
//!
//! 🎉 **The emulator successfully runs CHIP-8 ROMs!**
//!
//! Currently implemented and working:
//! - Complete CPU with instruction decoding and execution
//! - Memory system with ROM loading and font data
//! - Display system with XOR sprite drawing and collision detection
//! - ASCII terminal rendering for development
//!
//! # Architecture
//!
//! The emulator is built with a modular architecture:
//! - [`Memory`] - 4KB RAM with font data and ROM loading ✅
//! - [`Cpu`] - Instruction execution and register management ✅
//! - [`Display`] - 64x32 framebuffer with sprite operations ✅
//! - [`Renderer`] - ASCII and headless rendering backends ✅
//! - [`Input`] - 16-key keypad handling (TODO)
//! - [`Audio`] - Sound timer and beep generation (TODO)
//! - [`Emulator`] - Main coordination and timing (TODO)
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use joe::{Memory, Cpu, Display, AsciiRenderer, Renderer};
//!
//! // Create emulator components
//! let mut memory = Memory::new(true);
//! let mut cpu = Cpu::new();
//! let mut display = Display::new();
//! let renderer = AsciiRenderer;
//!
//! // Load a ROM file
//! let rom_data = std::fs::read("game.ch8").unwrap();
//! memory.load_rom(&rom_data).unwrap();
//!
//! // Execute instructions
//! cpu.execute_cycle(&mut memory, &mut display).unwrap();
//!
//! // Render the display
//! renderer.render(&display);
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
//! - ✅ Authentic CHIP-8 instruction set (core instructions implemented)
//! - ✅ 64x32 monochrome display with XOR sprite drawing and collision detection
//! - ✅ ROM loading from local files and remote URLs with size validation and error handling
//! - ✅ Memory write protection (configurable)
//! - ✅ ASCII terminal rendering for development
//! - ✅ Trait-based architecture for extensible rendering backends
//! - ✅ Comprehensive error handling with rich context
//! - 🚧 16-key hexadecimal keypad input (TODO)
//! - 🚧 Sound timer with beep generation (TODO)
//! - 🚧 Complete instruction set (remaining opcodes)

pub mod cpu;
pub mod disassembler;
pub mod display;
pub mod instruction;
pub mod memory;
pub mod rom_loader;
// pub mod input;
// pub mod audio;
// pub mod emulator;

// Re-export main types for convenience
pub use cpu::{Cpu, CpuError};
pub use disassembler::{
    InstructionAnalysis, analyze_instruction_usage, disassemble_rom, print_disassembly,
};
pub use display::{
    AsciiRenderer, Display, DisplayBus, DisplayError, DisplayStats, HeadlessRenderer, Renderer,
};
pub use instruction::{DecodeError, Instruction, decode_opcode};
pub use memory::{Memory, MemoryBus, MemoryError, MemoryStats};
pub use rom_loader::{RomLoaderConfig, RomSource, load_rom_data, load_rom_data_with_config};
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

// No tests in lib.rs - unit tests are in individual modules,
