//! CHIP-8 Input System
//!
//! Handles the 16-key hexadecimal keypad input for CHIP-8 programs.
//! The keypad layout is:
//!
//! ```text
//! CHIP-8 Keypad:     Keyboard Mapping:
//! ┌─┬─┬─┬─┐          ┌─┬─┬─┬─┐
//! │1│2│3│C│          │1│2│3│4│
//! ├─┼─┼─┼─┤          ├─┼─┼─┼─┤
//! │4│5│6│D│          │Q│W│E│R│
//! ├─┼─┼─┼─┤          ├─┼─┼─┼─┤
//! │7│8│9│E│          │A│S│D│F│
//! ├─┼─┼─┼─┤          ├─┼─┼─┼─┤
//! │A│0│B│F│          │Z│X│C│V│
//! └─┴─┴─┴─┘          └─┴─┴─┴─┘
//! ```

use std::collections::HashMap;
use thiserror::Error;

/// CHIP-8 key values (0-F hexadecimal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChipKey {
    Key0 = 0x0,
    Key1 = 0x1,
    Key2 = 0x2,
    Key3 = 0x3,
    Key4 = 0x4,
    Key5 = 0x5,
    Key6 = 0x6,
    Key7 = 0x7,
    Key8 = 0x8,
    Key9 = 0x9,
    KeyA = 0xA,
    KeyB = 0xB,
    KeyC = 0xC,
    KeyD = 0xD,
    KeyE = 0xE,
    KeyF = 0xF,
}

impl ChipKey {
    /// Convert from u8 to ChipKey
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(ChipKey::Key0),
            0x1 => Some(ChipKey::Key1),
            0x2 => Some(ChipKey::Key2),
            0x3 => Some(ChipKey::Key3),
            0x4 => Some(ChipKey::Key4),
            0x5 => Some(ChipKey::Key5),
            0x6 => Some(ChipKey::Key6),
            0x7 => Some(ChipKey::Key7),
            0x8 => Some(ChipKey::Key8),
            0x9 => Some(ChipKey::Key9),
            0xA => Some(ChipKey::KeyA),
            0xB => Some(ChipKey::KeyB),
            0xC => Some(ChipKey::KeyC),
            0xD => Some(ChipKey::KeyD),
            0xE => Some(ChipKey::KeyE),
            0xF => Some(ChipKey::KeyF),
            _ => None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Get all keys as an array
    pub fn all_keys() -> [ChipKey; 16] {
        [
            ChipKey::Key0,
            ChipKey::Key1,
            ChipKey::Key2,
            ChipKey::Key3,
            ChipKey::Key4,
            ChipKey::Key5,
            ChipKey::Key6,
            ChipKey::Key7,
            ChipKey::Key8,
            ChipKey::Key9,
            ChipKey::KeyA,
            ChipKey::KeyB,
            ChipKey::KeyC,
            ChipKey::KeyD,
            ChipKey::KeyE,
            ChipKey::KeyF,
        ]
    }
}

/// Errors that can occur during input operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum InputError {
    #[error("Invalid key value: {key} (must be 0-15)")]
    InvalidKey { key: u8 },
}

/// Trait for input handling - allows for different input backends
pub trait InputBus {
    /// Check if a specific key is currently pressed
    fn is_key_pressed(&self, key: ChipKey) -> bool;

    /// Check if a key by u8 value is currently pressed (for compatibility)
    fn is_key_pressed_u8(&self, key: u8) -> Result<bool, InputError> {
        match ChipKey::from_u8(key) {
            Some(chip_key) => Ok(self.is_key_pressed(chip_key)),
            None => Err(InputError::InvalidKey { key }),
        }
    }

    /// Wait for any key press and return the key value (blocking)
    fn wait_for_key_press(&mut self) -> Result<ChipKey, InputError>;

    /// Wait for any key press and return the key as u8 (for compatibility)
    fn wait_for_key_press_u8(&mut self) -> Result<u8, InputError> {
        self.wait_for_key_press().map(|key| key.to_u8())
    }

    /// Update the input state (called each frame)
    fn update(&mut self) -> Result<(), InputError>;

    /// Get a list of currently pressed keys
    fn get_pressed_keys(&self) -> Vec<ChipKey>;

    /// Get a list of currently pressed keys as u8 values (for compatibility)
    fn get_pressed_keys_u8(&self) -> Vec<u8> {
        self.get_pressed_keys()
            .iter()
            .map(|key| key.to_u8())
            .collect()
    }
}

/// CHIP-8 Input system managing the 16-key hexadecimal keypad
#[derive(Debug, Clone)]
pub struct Input {
    /// Current state of all 16 keys (true = pressed, false = released)
    key_states: [bool; 16],

    /// Keyboard mapping from char to CHIP-8 key value
    key_map: HashMap<char, ChipKey>,

    /// Reverse mapping from CHIP-8 key to keyboard char
    reverse_key_map: HashMap<ChipKey, char>,

    /// Buffer for input events
    input_buffer: Vec<char>,

    /// Whether we're currently waiting for a key press (for blocking input)
    waiting_for_key: bool,
}

impl Input {
    /// Create a new input system with default keyboard mapping
    pub fn new() -> Self {
        // Standard QWERTY keyboard mapping to CHIP-8 keypad
        let default_mappings = [
            ('1', ChipKey::Key1),
            ('2', ChipKey::Key2),
            ('3', ChipKey::Key3),
            ('4', ChipKey::KeyC),
            ('q', ChipKey::Key4),
            ('w', ChipKey::Key5),
            ('e', ChipKey::Key6),
            ('r', ChipKey::KeyD),
            ('a', ChipKey::Key7),
            ('s', ChipKey::Key8),
            ('d', ChipKey::Key9),
            ('f', ChipKey::KeyE),
            ('z', ChipKey::KeyA),
            ('x', ChipKey::Key0),
            ('c', ChipKey::KeyB),
            ('v', ChipKey::KeyF),
        ];

        Self::with_key_map(&default_mappings).expect("Default key mappings should be valid")
    }

    /// Create input system with custom key mapping
    pub fn with_key_map(mappings: &[(char, ChipKey)]) -> Result<Self, InputError> {
        let mut key_map = HashMap::new();
        let mut reverse_key_map = HashMap::new();

        for &(keyboard_key, chip8_key) in mappings {
            key_map.insert(keyboard_key, chip8_key);
            let upper = keyboard_key.to_ascii_uppercase();
            if upper != keyboard_key {
                key_map.insert(upper, chip8_key);
            }
            reverse_key_map.insert(chip8_key, keyboard_key);
        }

        Ok(Self {
            key_states: [false; 16],
            key_map,
            reverse_key_map,
            input_buffer: Vec::new(),
            waiting_for_key: false,
        })
    }

    /// Get the keyboard character mapped to a CHIP-8 key
    pub fn get_keyboard_key(&self, chip8_key: ChipKey) -> Option<char> {
        self.reverse_key_map.get(&chip8_key).copied()
    }

    /// Get the CHIP-8 key value for a keyboard character
    pub fn get_chip8_key(&self, keyboard_key: char) -> Option<ChipKey> {
        self.key_map.get(&keyboard_key).copied()
    }

    /// Press a key (for testing or programmatic input)
    pub fn press_key(&mut self, key: ChipKey) {
        self.key_states[key.to_u8() as usize] = true;
    }

    /// Press a key by u8 value (for compatibility)
    pub fn press_key_u8(&mut self, key: u8) -> Result<(), InputError> {
        match ChipKey::from_u8(key) {
            Some(chip_key) => {
                self.press_key(chip_key);
                Ok(())
            }
            None => Err(InputError::InvalidKey { key }),
        }
    }

    /// Release a key (for testing or programmatic input)
    pub fn release_key(&mut self, key: ChipKey) {
        self.key_states[key.to_u8() as usize] = false;
    }

    /// Release a key by u8 value (for compatibility)
    pub fn release_key_u8(&mut self, key: u8) -> Result<(), InputError> {
        match ChipKey::from_u8(key) {
            Some(chip_key) => {
                self.release_key(chip_key);
                Ok(())
            }
            None => Err(InputError::InvalidKey { key }),
        }
    }

    /// Process keyboard character input
    pub fn process_char_input(&mut self, ch: char) {
        if let Some(chip8_key) = self.get_chip8_key(ch) {
            self.key_states[chip8_key.to_u8() as usize] = true;
            // Add to buffer for key waiting (only when mapped)
            self.input_buffer.push(ch);
        }
    }

    /// Process key release
    pub fn process_char_release(&mut self, ch: char) {
        if let Some(chip8_key) = self.get_chip8_key(ch) {
            self.key_states[chip8_key.to_u8() as usize] = false;
        }
    }

    /// Clear all pressed keys
    pub fn clear_all_keys(&mut self) {
        self.key_states = [false; 16];
        self.input_buffer.clear();
    }

    /// Clear the input buffer (for testing)
    pub fn clear_input_buffer(&mut self) {
        self.input_buffer.clear();
    }

    /// Get input statistics
    pub fn get_stats(&self) -> InputStats {
        let pressed_count = self.key_states.iter().filter(|&&pressed| pressed).count();
        let mapped_keys = self.reverse_key_map.len(); // Count unique CHIP-8 keys that have mappings

        InputStats {
            pressed_keys: pressed_count,
            total_keys: 16,
            mapped_keyboard_keys: mapped_keys,
            waiting_for_input: self.waiting_for_key,
        }
    }

    /// Print the current keypad state for debugging
    pub fn print_keypad_state(&self) {
        println!("CHIP-8 Keypad State:");
        println!("┌─────┬─────┬─────┬─────┐");
        for row in 0..4 {
            print!("│");
            for col in 0..4 {
                let key = match (row, col) {
                    (0, 0) => ChipKey::Key1,
                    (0, 1) => ChipKey::Key2,
                    (0, 2) => ChipKey::Key3,
                    (0, 3) => ChipKey::KeyC,
                    (1, 0) => ChipKey::Key4,
                    (1, 1) => ChipKey::Key5,
                    (1, 2) => ChipKey::Key6,
                    (1, 3) => ChipKey::KeyD,
                    (2, 0) => ChipKey::Key7,
                    (2, 1) => ChipKey::Key8,
                    (2, 2) => ChipKey::Key9,
                    (2, 3) => ChipKey::KeyE,
                    (3, 0) => ChipKey::KeyA,
                    (3, 1) => ChipKey::Key0,
                    (3, 2) => ChipKey::KeyB,
                    (3, 3) => ChipKey::KeyF,
                    _ => unreachable!(),
                };

                let state = if self.key_states[key.to_u8() as usize] {
                    "█"
                } else {
                    " "
                };
                let key_char = self.get_keyboard_key(key).unwrap_or('?');
                print!(" {}{:X}{} │", state, key.to_u8(), key_char);
            }
            println!();
            if row < 3 {
                println!("├─────┼─────┼─────┼─────┤");
            }
        }
        println!("└─────┴─────┴─────┴─────┘");

        let pressed_keys: Vec<ChipKey> = self.get_pressed_keys();
        if !pressed_keys.is_empty() {
            println!("Pressed keys: {:?}", pressed_keys);
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl InputBus for Input {
    fn is_key_pressed(&self, key: ChipKey) -> bool {
        self.key_states[key.to_u8() as usize]
    }

    fn wait_for_key_press(&mut self) -> Result<ChipKey, InputError> {
        self.waiting_for_key = true;

        // Check if any key is currently pressed
        for (i, &pressed) in self.key_states.iter().enumerate() {
            if pressed {
                self.waiting_for_key = false;
                return Ok(ChipKey::from_u8(i as u8).unwrap());
            }
        }

        // Check input buffer for recent key presses
        while let Some(ch) = self.input_buffer.pop() {
            if let Some(chip8_key) = self.get_chip8_key(ch) {
                self.waiting_for_key = false;
                return Ok(chip8_key);
            }
        }

        // No key available - in a real implementation, this would block
        // For now, we return an error to indicate no key is available
        Err(InputError::InvalidKey { key: 0xFF })
    }

    fn update(&mut self) -> Result<(), InputError> {
        // In a real terminal implementation, this would poll for keyboard events
        // For now, this is a no-op - input is driven by external calls to process_char_input
        Ok(())
    }

    fn get_pressed_keys(&self) -> Vec<ChipKey> {
        self.key_states
            .iter()
            .enumerate()
            .filter_map(|(i, &pressed)| {
                if pressed {
                    ChipKey::from_u8(i as u8)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Statistics about the current input state
#[derive(Debug, Clone, PartialEq)]
pub struct InputStats {
    /// Number of currently pressed keys
    pub pressed_keys: usize,
    /// Total number of CHIP-8 keys (always 16)
    pub total_keys: usize,
    /// Number of keyboard keys mapped to CHIP-8 keys
    pub mapped_keyboard_keys: usize,
    /// Whether the system is waiting for key input
    pub waiting_for_input: bool,
}

/// Mock input for testing - allows programmatic control of key states
#[derive(Debug, Clone)]
pub struct MockInput {
    key_states: [bool; 16],
    key_queue: Vec<ChipKey>,
}

impl MockInput {
    pub fn new() -> Self {
        Self {
            key_states: [false; 16],
            key_queue: Vec::new(),
        }
    }

    pub fn press_key(&mut self, key: ChipKey) {
        self.key_states[key.to_u8() as usize] = true;
        self.key_queue.push(key);
    }

    pub fn press_key_u8(&mut self, key: u8) -> Result<(), InputError> {
        match ChipKey::from_u8(key) {
            Some(chip_key) => {
                self.press_key(chip_key);
                Ok(())
            }
            None => Err(InputError::InvalidKey { key }),
        }
    }

    pub fn release_key(&mut self, key: ChipKey) {
        self.key_states[key.to_u8() as usize] = false;
    }

    pub fn release_key_u8(&mut self, key: u8) -> Result<(), InputError> {
        match ChipKey::from_u8(key) {
            Some(chip_key) => {
                self.release_key(chip_key);
                Ok(())
            }
            None => Err(InputError::InvalidKey { key }),
        }
    }

    pub fn clear_all(&mut self) {
        self.key_states = [false; 16];
        self.key_queue.clear();
    }
}

impl Default for MockInput {
    fn default() -> Self {
        Self::new()
    }
}

impl InputBus for MockInput {
    fn is_key_pressed(&self, key: ChipKey) -> bool {
        self.key_states[key.to_u8() as usize]
    }

    fn wait_for_key_press(&mut self) -> Result<ChipKey, InputError> {
        if let Some(key) = self.key_queue.pop() {
            Ok(key)
        } else {
            Err(InputError::InvalidKey { key: 0xFF })
        }
    }

    fn update(&mut self) -> Result<(), InputError> {
        Ok(())
    }

    fn get_pressed_keys(&self) -> Vec<ChipKey> {
        self.key_states
            .iter()
            .enumerate()
            .filter_map(|(i, &pressed)| {
                if pressed {
                    ChipKey::from_u8(i as u8)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_creation() {
        let input = Input::new();

        // All keys should be released initially
        for key in ChipKey::all_keys() {
            assert!(!input.is_key_pressed(key));
        }
    }

    #[test]
    fn test_key_mapping() {
        let input = Input::new();

        // Test standard mappings
        assert_eq!(input.get_chip8_key('1'), Some(ChipKey::Key1));
        assert_eq!(input.get_chip8_key('2'), Some(ChipKey::Key2));
        assert_eq!(input.get_chip8_key('3'), Some(ChipKey::Key3));
        assert_eq!(input.get_chip8_key('4'), Some(ChipKey::KeyC));

        assert_eq!(input.get_chip8_key('q'), Some(ChipKey::Key4));
        assert_eq!(input.get_chip8_key('Q'), Some(ChipKey::Key4)); // Case insensitive

        assert_eq!(input.get_chip8_key('x'), Some(ChipKey::Key0));
        assert_eq!(input.get_chip8_key('v'), Some(ChipKey::KeyF));

        // Test reverse mapping
        assert_eq!(input.get_keyboard_key(ChipKey::Key1), Some('1'));
        assert_eq!(input.get_keyboard_key(ChipKey::Key4), Some('q'));
        assert_eq!(input.get_keyboard_key(ChipKey::Key0), Some('x'));
        assert_eq!(input.get_keyboard_key(ChipKey::KeyF), Some('v'));
    }

    #[test]
    fn test_custom_key_mapping() {
        let mappings = [
            ('a', ChipKey::Key0),
            ('b', ChipKey::Key1),
            ('c', ChipKey::Key2),
            ('d', ChipKey::Key3),
        ];
        let input = Input::with_key_map(&mappings).unwrap();

        assert_eq!(input.get_chip8_key('a'), Some(ChipKey::Key0));
        assert_eq!(input.get_chip8_key('A'), Some(ChipKey::Key0)); // Case insensitive
        assert_eq!(input.get_chip8_key('d'), Some(ChipKey::Key3));

        // Old mappings should not work
        assert_eq!(input.get_chip8_key('1'), None);
        assert_eq!(input.get_chip8_key('q'), None);
    }

    #[test]
    fn test_chip_key_conversion() {
        // Test valid conversions
        assert_eq!(ChipKey::from_u8(0), Some(ChipKey::Key0));
        assert_eq!(ChipKey::from_u8(15), Some(ChipKey::KeyF));

        // Test invalid conversions
        assert_eq!(ChipKey::from_u8(16), None);
        assert_eq!(ChipKey::from_u8(255), None);

        // Test to_u8
        assert_eq!(ChipKey::Key0.to_u8(), 0);
        assert_eq!(ChipKey::KeyF.to_u8(), 15);
    }

    #[test]
    fn test_key_press_release() {
        let mut input = Input::new();

        // Press key 5
        input.press_key(ChipKey::Key5);
        assert!(input.is_key_pressed(ChipKey::Key5));
        assert!(!input.is_key_pressed(ChipKey::Key4));

        // Release key 5
        input.release_key(ChipKey::Key5);
        assert!(!input.is_key_pressed(ChipKey::Key5));
    }

    #[test]
    fn test_invalid_key_operations() {
        let mut input = Input::new();

        // Test invalid key press via u8 API
        assert!(matches!(
            input.press_key_u8(16),
            Err(InputError::InvalidKey { key: 16 })
        ));

        // Test invalid key release via u8 API
        assert!(matches!(
            input.release_key_u8(16),
            Err(InputError::InvalidKey { key: 16 })
        ));

        // Test invalid key check via u8 API
        assert!(matches!(
            input.is_key_pressed_u8(16),
            Err(InputError::InvalidKey { key: 16 })
        ));
    }

    #[test]
    fn test_char_input_processing() {
        let mut input = Input::new();

        // Process character input
        input.process_char_input('q');
        assert!(input.is_key_pressed(ChipKey::Key4)); // 'q' maps to Key4

        // Process character release
        input.process_char_release('q');
        assert!(!input.is_key_pressed(ChipKey::Key4));
    }

    #[test]
    fn test_clear_all_keys() {
        let mut input = Input::new();

        // Press several keys
        input.press_key(ChipKey::Key1);
        input.press_key(ChipKey::Key5);
        input.press_key(ChipKey::KeyA);

        assert!(input.is_key_pressed(ChipKey::Key1));
        assert!(input.is_key_pressed(ChipKey::Key5));
        assert!(input.is_key_pressed(ChipKey::KeyA));

        // Clear all keys
        input.clear_all_keys();

        // All keys should be released
        for key in ChipKey::all_keys() {
            assert!(!input.is_key_pressed(key));
        }
    }

    #[test]
    fn test_get_pressed_keys() {
        let mut input = Input::new();

        // No keys pressed initially
        assert!(input.get_pressed_keys().is_empty());

        // Press some keys
        input.press_key(ChipKey::Key2);
        input.press_key(ChipKey::Key7);
        input.press_key(ChipKey::KeyF);

        let pressed = input.get_pressed_keys();
        assert_eq!(pressed.len(), 3);
        assert!(pressed.contains(&ChipKey::Key2));
        assert!(pressed.contains(&ChipKey::Key7));
        assert!(pressed.contains(&ChipKey::KeyF));
    }

    #[test]
    fn test_input_stats() {
        let mut input = Input::new();

        let stats = input.get_stats();
        assert_eq!(stats.pressed_keys, 0);
        assert_eq!(stats.total_keys, 16);
        assert_eq!(stats.mapped_keyboard_keys, 16);
        assert!(!stats.waiting_for_input);

        // Press some keys
        input.press_key(ChipKey::Key1);
        input.press_key(ChipKey::Key5);

        let stats = input.get_stats();
        assert_eq!(stats.pressed_keys, 2);
    }

    #[test]
    fn test_mock_input() {
        let mut mock = MockInput::new();

        // Test key press
        mock.press_key(ChipKey::Key5);
        assert!(mock.is_key_pressed(ChipKey::Key5));

        // Test key release
        mock.release_key(ChipKey::Key5);
        assert!(!mock.is_key_pressed(ChipKey::Key5));

        // Clear queue from previous operations
        mock.clear_all();

        // Test key queue - pressing key adds to both state and queue
        mock.press_key(ChipKey::Key3);
        assert_eq!(mock.wait_for_key_press().unwrap(), ChipKey::Key3);

        // Queue should be empty now
        assert!(mock.wait_for_key_press().is_err());
    }

    #[test]
    fn test_wait_for_key_press() {
        let mut input = Input::new();

        // Add key to buffer
        input.process_char_input('w'); // Maps to Key5

        // Should return the key (either from pressed state or buffer)
        assert_eq!(input.wait_for_key_press().unwrap(), ChipKey::Key5);

        // Release the key and clear buffer, then should return error
        input.process_char_release('w');
        input.clear_input_buffer();
        assert!(input.wait_for_key_press().is_err());
    }
}
