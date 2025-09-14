//! CHIP-8 Memory System
//!
//! Implements the 4KB memory layout with built-in font data and ROM loading capabilities.
//! Provides write protection for the interpreter area with optional override.

use crate::constants::*;
use thiserror::Error;

/// Memory bus trait for CPU to interact with memory system
pub trait MemoryBus {
    /// Read a single byte from memory
    fn read_byte(&self, addr: u16) -> Result<u8, MemoryError>;

    /// Write a single byte to memory
    fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), MemoryError>;
}

/// End of interpreter area (write-protected by default)
pub const INTERPRETER_END_ADDR: u16 = 0x1FF;

/// Height of each font character in bytes
pub const FONT_HEIGHT: usize = 5;

/// Total size of the font set (16 characters × 5 bytes each)
pub const FONT_SET_SIZE: usize = 16 * FONT_HEIGHT;

/// Maximum ROM size (from PROGRAM_START_ADDR to end of memory)
pub const MAX_ROM_SIZE: usize = MEMORY_SIZE - PROGRAM_START_ADDR as usize;

/// Built-in hexadecimal font set (0-F)
/// Each character is 4×5 pixels, represented as 5 bytes
const FONT_SET: [u8; FONT_SET_SIZE] = [
    // 0
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 1
    0x20, 0x60, 0x20, 0x20, 0x70, // 2
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 3
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 4
    0x90, 0x90, 0xF0, 0x10, 0x10, // 5
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 6
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 7
    0xF0, 0x10, 0x20, 0x40, 0x40, // 8
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // A
    0xF0, 0x90, 0xF0, 0x90, 0x90, // B
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // C
    0xF0, 0x80, 0x80, 0x80, 0xF0, // D
    0xE0, 0x90, 0x90, 0x90, 0xE0, // E
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // F
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

/// Memory errors
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Address {addr:#06x} is out of bounds (max: {max:#06x})")]
    OutOfBounds { addr: u16, max: u16 },

    #[error("Cannot write to interpreter area at {addr:#06x} (write protection enabled)")]
    WriteProtected { addr: u16 },

    #[error("ROM too large: {size} bytes (max: {max_size} bytes)")]
    RomTooLarge { size: usize, max_size: usize },

    #[error("Invalid font digit: {digit} (must be 0-15)")]
    InvalidFontDigit { digit: u8 },

    #[error("Word read at {addr:#06x} would exceed memory bounds")]
    WordReadOutOfBounds { addr: u16 },

    #[error("Word write at {addr:#06x} would exceed memory bounds")]
    WordWriteOutOfBounds { addr: u16 },
}

/// CHIP-8 Memory system
pub struct Memory {
    /// 4KB RAM
    ram: [u8; MEMORY_SIZE],
    /// Write protection for interpreter area (0x000-0x1FF)
    write_protection_enabled: bool,
}

impl Memory {
    /// Create a new memory system with built-in font data
    pub fn new(write_protection_enabled: bool) -> Self {
        let mut memory = Self {
            ram: [0; MEMORY_SIZE],
            write_protection_enabled,
        };

        // Load font data at the standard location
        memory.load_font_data();
        memory
    }

    /// Load the built-in font set into memory at FONT_START_ADDR
    fn load_font_data(&mut self) {
        let start = FONT_START_ADDR as usize;
        let end = start + FONT_SET_SIZE;
        self.ram[start..end].copy_from_slice(&FONT_SET);
    }

    /// Read a single byte from memory
    pub fn read_byte(&self, addr: u16) -> Result<u8, MemoryError> {
        if addr as usize >= MEMORY_SIZE {
            return Err(MemoryError::OutOfBounds {
                addr,
                max: (MEMORY_SIZE - 1) as u16,
            });
        }

        Ok(self.ram[addr as usize])
    }

    /// Write a single byte to memory
    pub fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), MemoryError> {
        if addr as usize >= MEMORY_SIZE {
            return Err(MemoryError::OutOfBounds {
                addr,
                max: (MEMORY_SIZE - 1) as u16,
            });
        }

        // Check write protection for interpreter area
        if self.write_protection_enabled && addr <= INTERPRETER_END_ADDR {
            return Err(MemoryError::WriteProtected { addr });
        }

        self.ram[addr as usize] = value;
        Ok(())
    }

    /// Read a 16-bit word from memory (big-endian)
    pub fn read_word(&self, addr: u16) -> Result<u16, MemoryError> {
        if addr as usize + 1 >= MEMORY_SIZE {
            return Err(MemoryError::WordReadOutOfBounds { addr });
        }

        let bytes = [self.ram[addr as usize], self.ram[addr as usize + 1]];
        Ok(u16::from_be_bytes(bytes))
    }

    /// Write a 16-bit word to memory (big-endian)
    pub fn write_word(&mut self, addr: u16, value: u16) -> Result<(), MemoryError> {
        if addr as usize + 1 >= MEMORY_SIZE {
            return Err(MemoryError::WordWriteOutOfBounds { addr });
        }

        let bytes = value.to_be_bytes();
        self.write_byte(addr, bytes[0])?;
        self.write_byte(addr + 1, bytes[1])?;
        Ok(())
    }

    /// Load ROM data starting at PROGRAM_START_ADDR
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), MemoryError> {
        if rom_data.len() > MAX_ROM_SIZE {
            return Err(MemoryError::RomTooLarge {
                size: rom_data.len(),
                max_size: MAX_ROM_SIZE,
            });
        }

        let start = PROGRAM_START_ADDR as usize;
        let end = start + rom_data.len();
        self.ram[start..end].copy_from_slice(rom_data);

        Ok(())
    }

    /// Get font sprite data for a hexadecimal digit (0-F)
    /// Returns a slice of 5 bytes representing the 4×5 pixel font
    pub fn get_font_sprite(&self, digit: u8) -> Result<&[u8], MemoryError> {
        if digit > 0xF {
            return Err(MemoryError::InvalidFontDigit { digit });
        }

        let start = FONT_START_ADDR as usize + (digit as usize * FONT_HEIGHT);
        let end = start + FONT_HEIGHT;
        Ok(&self.ram[start..end])
    }

    /// Get the address of a font sprite for a hexadecimal digit (0-F)
    /// This is commonly used by the CHIP-8 interpreter
    pub fn get_font_sprite_addr(&self, digit: u8) -> Result<u16, MemoryError> {
        if digit > 0xF {
            return Err(MemoryError::InvalidFontDigit { digit });
        }

        Ok(FONT_START_ADDR + (digit as u16 * FONT_HEIGHT as u16))
    }

    /// Enable or disable write protection for the interpreter area
    pub fn set_write_protection(&mut self, enabled: bool) {
        self.write_protection_enabled = enabled;
    }

    /// Check if write protection is enabled
    pub fn is_write_protection_enabled(&self) -> bool {
        self.write_protection_enabled
    }

    /// Clear all memory (except font data)
    pub fn reset(&mut self) {
        // Clear everything
        self.ram.fill(0);
        // Reload font data
        self.load_font_data();
    }

    /// Get a read-only view of the entire memory
    /// Useful for debugging and testing
    pub fn as_slice(&self) -> &[u8] {
        &self.ram
    }

    /// Get memory usage statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_size: MEMORY_SIZE,
            font_start: FONT_START_ADDR,
            font_size: FONT_SET_SIZE,
            program_start: PROGRAM_START_ADDR,
            max_rom_size: MAX_ROM_SIZE,
            write_protection_enabled: self.write_protection_enabled,
        }
    }
}

/// Memory system statistics
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStats {
    pub total_size: usize,
    pub font_start: u16,
    pub font_size: usize,
    pub program_start: u16,
    pub max_rom_size: usize,
    pub write_protection_enabled: bool,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new(true) // Write protection enabled by default
    }
}

impl MemoryBus for Memory {
    fn read_byte(&self, addr: u16) -> Result<u8, MemoryError> {
        self.read_byte(addr)
    }

    fn write_byte(&mut self, addr: u16, value: u8) -> Result<(), MemoryError> {
        self.write_byte(addr, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_memory_has_font_data() {
        let memory = Memory::new(true);

        // Check that font data is loaded
        let zero_sprite = memory.get_font_sprite(0).unwrap();
        assert_eq!(zero_sprite, &[0xF0, 0x90, 0x90, 0x90, 0xF0]);

        let f_sprite = memory.get_font_sprite(0xF).unwrap();
        assert_eq!(f_sprite, &[0xF0, 0x80, 0xF0, 0x80, 0x80]);
    }

    #[test]
    fn test_byte_read_write() {
        let mut memory = Memory::new(false); // Disable write protection

        // Test write and read
        memory.write_byte(0x300, 0x42).unwrap();
        assert_eq!(memory.read_byte(0x300).unwrap(), 0x42);
    }

    #[test]
    fn test_word_read_write() {
        let mut memory = Memory::new(false);

        // Test big-endian word operations
        memory.write_word(0x300, 0x1234).unwrap();
        assert_eq!(memory.read_word(0x300).unwrap(), 0x1234);

        // Verify individual bytes are stored correctly (big-endian)
        assert_eq!(memory.read_byte(0x300).unwrap(), 0x12);
        assert_eq!(memory.read_byte(0x301).unwrap(), 0x34);
    }

    #[test]
    fn test_write_protection() {
        let mut memory = Memory::new(true); // Enable write protection

        // Should fail to write to interpreter area
        let result = memory.write_byte(0x100, 0x42);
        assert!(matches!(
            result,
            Err(MemoryError::WriteProtected { addr: 0x100 })
        ));

        // Should succeed in program area
        memory.write_byte(0x300, 0x42).unwrap();
        assert_eq!(memory.read_byte(0x300).unwrap(), 0x42);
    }

    #[test]
    fn test_rom_loading() {
        let mut memory = Memory::new(true);
        let rom_data = vec![0x12, 0x34, 0x56, 0x78];

        memory.load_rom(&rom_data).unwrap();

        // Verify ROM is loaded at correct address
        assert_eq!(memory.read_byte(PROGRAM_START_ADDR).unwrap(), 0x12);
        assert_eq!(memory.read_byte(PROGRAM_START_ADDR + 1).unwrap(), 0x34);
        assert_eq!(memory.read_byte(PROGRAM_START_ADDR + 2).unwrap(), 0x56);
        assert_eq!(memory.read_byte(PROGRAM_START_ADDR + 3).unwrap(), 0x78);
    }

    #[test]
    fn test_rom_too_large() {
        let mut memory = Memory::new(true);
        let large_rom = vec![0; MAX_ROM_SIZE + 1];

        let result = memory.load_rom(&large_rom);
        assert!(matches!(
            result,
            Err(MemoryError::RomTooLarge { size, max_size })
            if size == MAX_ROM_SIZE + 1 && max_size == MAX_ROM_SIZE
        ));
    }

    #[test]
    fn test_bounds_checking() {
        let memory = Memory::new(true);

        // Test read out of bounds
        let result = memory.read_byte(MEMORY_SIZE as u16);
        assert!(matches!(result, Err(MemoryError::OutOfBounds { .. })));

        // Test word read out of bounds
        let result = memory.read_word((MEMORY_SIZE - 1) as u16);
        assert!(matches!(
            result,
            Err(MemoryError::WordReadOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_font_sprite_addresses() {
        let memory = Memory::new(true);

        // Test all hexadecimal digits
        for digit in 0..=0xF {
            let addr = memory.get_font_sprite_addr(digit).unwrap();
            let expected_addr = FONT_START_ADDR + (digit as u16 * FONT_HEIGHT as u16);
            assert_eq!(addr, expected_addr);
        }

        // Test invalid digit
        let result = memory.get_font_sprite_addr(0x10);
        assert!(matches!(
            result,
            Err(MemoryError::InvalidFontDigit { digit: 0x10 })
        ));
    }
}
