//! Integration tests for the OCTO CHIP-8 emulator library
//!
//! Tests real workflows and edge cases that span multiple components.

use octo::{Memory, MemoryError, constants};

#[test]
fn test_complete_rom_loading_workflow() {
    let mut memory = Memory::new(true);

    // Load a realistic ROM with mixed instructions
    let rom_data = vec![0xA2, 0x2A, 0x60, 0x0C, 0x61, 0x08, 0xD0, 0x14];
    memory.load_rom(&rom_data).expect("Should load valid ROM");

    // Should be able to read back as both bytes and words
    assert_eq!(
        memory.read_word(constants::PROGRAM_START_ADDR).unwrap(),
        0xA22A
    );
    assert_eq!(
        memory.read_word(constants::PROGRAM_START_ADDR + 2).unwrap(),
        0x600C
    );

    // Test ROM size limits
    let oversized_rom =
        vec![0x42; constants::MEMORY_SIZE - constants::PROGRAM_START_ADDR as usize + 1];
    let result = memory.load_rom(&oversized_rom);
    assert!(matches!(result, Err(MemoryError::RomTooLarge { .. })));
}

#[test]
fn test_write_protection_with_reset() {
    let mut memory = Memory::new(true);

    // Load ROM, then reset and verify protection still works
    memory.load_rom(&[0x12, 0x34]).unwrap();
    memory.reset();

    // Protection should still be enabled after reset
    let result = memory.write_byte(0x100, 0x42);
    assert!(matches!(result, Err(MemoryError::WriteProtected { .. })));

    // But font should be reloaded correctly
    assert_eq!(memory.get_font_sprite(0).unwrap().len(), 5);
}
