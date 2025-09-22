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

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Receiver;
use thiserror::Error;

/// Resolved key mappings for CHIP-8 input
#[derive(Debug, Clone)]
pub struct KeyMappings {
    /// Keyboard mapping from char to CHIP-8 key value (0-15)
    key_map: HashMap<char, u8>,
    /// Reverse mapping from CHIP-8 key to keyboard char
    reverse_key_map: HashMap<u8, char>,
}

impl KeyMappings {
    /// Create key mappings from raw mapping pairs
    pub fn from_pairs(mappings: &[(char, u8)]) -> Result<Self, InputError> {
        let mut key_map = HashMap::new();
        let mut reverse_key_map = HashMap::new();

        for &(keyboard_key, chip8_key) in mappings {
            if !is_valid_key(chip8_key) {
                return Err(InputError::InvalidKey { key: chip8_key });
            }

            key_map.insert(keyboard_key, chip8_key);
            let upper = keyboard_key.to_ascii_uppercase();
            if upper != keyboard_key {
                key_map.insert(upper, chip8_key);
            }
            reverse_key_map.insert(chip8_key, keyboard_key);
        }

        Ok(Self {
            key_map,
            reverse_key_map,
        })
    }

    /// Get the CHIP-8 key for a keyboard character
    pub fn get_chip8_key(&self, keyboard_key: char) -> Option<u8> {
        self.key_map.get(&keyboard_key).copied()
    }

    /// Get the keyboard character for a CHIP-8 key
    pub fn get_keyboard_key(&self, chip8_key: u8) -> Option<char> {
        self.reverse_key_map.get(&chip8_key).copied()
    }
}

/// Resolve key mappings from config or use defaults
pub fn resolve_key_mappings(
    config_mappings: Option<&HashMap<String, String>>,
) -> Result<KeyMappings, InputError> {
    match config_mappings {
        Some(mappings) => {
            let mut converted_mappings = Vec::new();

            for (chip8_key_str, keyboard_key_str) in mappings {
                // Parse CHIP-8 key from hex string
                let chip8_key = u8::from_str_radix(chip8_key_str, 16)
                    .map_err(|_| InputError::InvalidKey { key: 255 })?; // Use 255 as invalid placeholder

                if !is_valid_key(chip8_key) {
                    return Err(InputError::InvalidKey { key: chip8_key });
                }

                // Get first character from keyboard key string
                if let Some(keyboard_char) = keyboard_key_str.chars().next() {
                    converted_mappings.push((keyboard_char.to_ascii_lowercase(), chip8_key));
                }
            }

            KeyMappings::from_pairs(&converted_mappings)
        }
        None => {
            // Use default mappings
            let default_mappings = [
                ('1', 0x1),
                ('2', 0x2),
                ('3', 0x3),
                ('4', 0xC),
                ('q', 0x4),
                ('w', 0x5),
                ('e', 0x6),
                ('r', 0xD),
                ('a', 0x7),
                ('s', 0x8),
                ('d', 0x9),
                ('f', 0xE),
                ('z', 0xA),
                ('x', 0x0),
                ('c', 0xB),
                ('v', 0xF),
            ];
            KeyMappings::from_pairs(&default_mappings)
        }
    }
}

/// Validate that a key value is in the valid CHIP-8 range (0-15)
fn is_valid_key(key: u8) -> bool {
    key <= 0xF
}

/// Key events that can be sent through channels
#[derive(Debug, Clone, PartialEq)]
pub enum KeyEvent {
    /// A key was pressed
    Pressed(char),
    /// A key was released
    Released(char),
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
    fn is_key_pressed(&self, key: u8) -> Result<bool, InputError>;

    /// Try to get a key press without blocking - returns None if no key available
    fn try_get_key_press(&mut self) -> Option<u8>;

    /// Update the input state (called each frame)
    fn update(&mut self);

    /// Get a list of currently pressed keys
    fn get_pressed_keys(&self) -> Vec<u8>;
}

/// CHIP-8 Input system managing the 16-key hexadecimal keypad
#[derive(Debug)]
pub struct Input {
    /// Current state of all 16 keys (true = pressed, false = released)
    key_states: [bool; 16],

    /// Key mappings resolver
    key_mappings: KeyMappings,

    /// Buffer for input events
    input_buffer: Vec<char>,

    /// Whether the system is currently waiting for any key press
    waiting_for_key: bool,

    /// Optional receiver for key events from external sources (like display renderer)
    key_receiver: Option<Receiver<KeyEvent>>,
}

impl Input {
    /// Create a new input system with default keyboard mapping
    pub fn new() -> Self {
        let key_mappings =
            resolve_key_mappings(None).expect("Default key mappings should be valid");
        Self::with_mappings(key_mappings, None)
    }

    /// Create input system with resolved key mappings and optional receiver
    pub fn with_mappings(
        key_mappings: KeyMappings,
        key_receiver: Option<Receiver<KeyEvent>>,
    ) -> Self {
        Self {
            key_states: [false; 16],
            key_mappings,
            input_buffer: Vec::new(),
            waiting_for_key: false,
            key_receiver,
        }
    }

    /// Get the keyboard character mapped to a CHIP-8 key
    pub fn get_keyboard_key(&self, chip8_key: u8) -> Option<char> {
        self.key_mappings.get_keyboard_key(chip8_key)
    }

    /// Get the CHIP-8 key value for a keyboard character
    pub fn get_chip8_key(&self, keyboard_key: char) -> Option<u8> {
        self.key_mappings.get_chip8_key(keyboard_key)
    }

    /// Press a key (for testing or programmatic input)
    pub fn press_key(&mut self, key: u8) -> Result<(), InputError> {
        if !is_valid_key(key) {
            return Err(InputError::InvalidKey { key });
        }
        self.key_states[key as usize] = true;
        Ok(())
    }

    /// Release a key (for testing or programmatic input)
    pub fn release_key(&mut self, key: u8) -> Result<(), InputError> {
        if !is_valid_key(key) {
            return Err(InputError::InvalidKey { key });
        }
        self.key_states[key as usize] = false;
        Ok(())
    }

    /// Process keyboard character input
    pub fn process_char_input(&mut self, ch: char) {
        if let Some(chip8_key_value) = self.get_chip8_key(ch) {
            self.key_states[chip8_key_value as usize] = true;
            // Add to buffer for key waiting (only when mapped)
            self.input_buffer.push(ch);
        }
    }

    /// Process key release
    pub fn process_char_release(&mut self, ch: char) {
        if let Some(chip8_key_value) = self.get_chip8_key(ch) {
            self.key_states[chip8_key_value as usize] = false;
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
        let mapped_keys = (0..16u8)
            .filter(|&k| self.key_mappings.get_keyboard_key(k).is_some())
            .count();

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
                let key_value = match (row, col) {
                    (0, 0) => 0x1,
                    (0, 1) => 0x2,
                    (0, 2) => 0x3,
                    (0, 3) => 0xC,
                    (1, 0) => 0x4,
                    (1, 1) => 0x5,
                    (1, 2) => 0x6,
                    (1, 3) => 0xD,
                    (2, 0) => 0x7,
                    (2, 1) => 0x8,
                    (2, 2) => 0x9,
                    (2, 3) => 0xE,
                    (3, 0) => 0xA,
                    (3, 1) => 0x0,
                    (3, 2) => 0xB,
                    (3, 3) => 0xF,
                    _ => unreachable!(),
                };

                let state = if self.key_states[key_value as usize] {
                    "█"
                } else {
                    " "
                };
                let key_char = self.get_keyboard_key(key_value).unwrap_or('?');
                print!(" {}{:X}{} │", state, key_value, key_char);
            }
            println!();
            if row < 3 {
                println!("├─────┼─────┼─────┼─────┤");
            }
        }
        println!("└─────┴─────┴─────┴─────┘");

        let pressed_keys: Vec<u8> = self.get_pressed_keys();
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
    fn is_key_pressed(&self, key: u8) -> Result<bool, InputError> {
        if !is_valid_key(key) {
            return Err(InputError::InvalidKey { key });
        }
        Ok(self.key_states[key as usize])
    }

    fn try_get_key_press(&mut self) -> Option<u8> {
        // Check if any key is currently pressed
        let pressed_keys = self.get_pressed_keys();
        if let Some(&first_key) = pressed_keys.first() {
            return Some(first_key);
        }

        // Check input buffer for recent key presses
        while let Some(ch) = self.input_buffer.pop() {
            if let Some(chip8_key_value) = self.get_chip8_key(ch) {
                return Some(chip8_key_value);
            }
        }

        // No key available - perfectly normal condition
        None
    }

    fn update(&mut self) {
        // Process any pending key events from the channel
        let mut events_to_process = Vec::new();
        if let Some(receiver) = &self.key_receiver {
            while let Ok(event) = receiver.try_recv() {
                events_to_process.push(event);
            }
        }

        // Process events without borrowing issues
        for event in events_to_process {
            match event {
                KeyEvent::Pressed(ch) => self.process_char_input(ch),
                KeyEvent::Released(ch) => self.process_char_release(ch),
            }
        }
    }

    fn get_pressed_keys(&self) -> Vec<u8> {
        self.key_states
            .iter()
            .enumerate()
            .filter_map(|(i, &pressed)| if pressed { Some(i as u8) } else { None })
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
    key_queue: VecDeque<u8>,
}

impl MockInput {
    pub fn new() -> Self {
        Self {
            key_states: [false; 16],
            key_queue: VecDeque::new(),
        }
    }

    pub fn press_key(&mut self, key: u8) -> Result<(), InputError> {
        if !is_valid_key(key) {
            return Err(InputError::InvalidKey { key });
        }
        self.key_states[key as usize] = true;
        self.key_queue.push_back(key);
        Ok(())
    }

    pub fn release_key(&mut self, key: u8) -> Result<(), InputError> {
        if !is_valid_key(key) {
            return Err(InputError::InvalidKey { key });
        }
        self.key_states[key as usize] = false;
        Ok(())
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
    fn is_key_pressed(&self, key: u8) -> Result<bool, InputError> {
        if !is_valid_key(key) {
            return Err(InputError::InvalidKey { key });
        }
        Ok(self.key_states[key as usize])
    }

    fn try_get_key_press(&mut self) -> Option<u8> {
        self.key_queue.pop_front()
    }

    fn update(&mut self) {
        // No-op for mock
    }

    fn get_pressed_keys(&self) -> Vec<u8> {
        self.key_states
            .iter()
            .enumerate()
            .filter_map(|(i, &pressed)| if pressed { Some(i as u8) } else { None })
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
        for i in 0..16 {
            assert!(!input.is_key_pressed(i).unwrap());
        }
    }

    #[test]
    fn test_key_mapping() {
        let input = Input::new();

        // Test standard mappings
        assert_eq!(input.get_chip8_key('1'), Some(0x1));
        assert_eq!(input.get_chip8_key('2'), Some(0x2));
        assert_eq!(input.get_chip8_key('3'), Some(0x3));
        assert_eq!(input.get_chip8_key('4'), Some(0xC));

        assert_eq!(input.get_chip8_key('q'), Some(0x4));
        assert_eq!(input.get_chip8_key('Q'), Some(0x4)); // Case insensitive

        assert_eq!(input.get_chip8_key('x'), Some(0x0));
        assert_eq!(input.get_chip8_key('v'), Some(0xF));

        // Test reverse mapping
        assert_eq!(input.get_keyboard_key(0x1), Some('1'));
        assert_eq!(input.get_keyboard_key(0x4), Some('q'));
        assert_eq!(input.get_keyboard_key(0x0), Some('x'));
        assert_eq!(input.get_keyboard_key(0xF), Some('v'));
    }

    #[test]
    fn test_custom_key_mapping() {
        let mappings = [('a', 0x0), ('b', 0x1), ('c', 0x2), ('d', 0x3)];
        let key_mappings = KeyMappings::from_pairs(&mappings).unwrap();
        let input = Input::with_mappings(key_mappings, None);

        assert_eq!(input.get_chip8_key('a'), Some(0x0));
        assert_eq!(input.get_chip8_key('A'), Some(0x0)); // Case insensitive
        assert_eq!(input.get_chip8_key('d'), Some(0x3));

        // Old mappings should not work
        assert_eq!(input.get_chip8_key('1'), None);
        assert_eq!(input.get_chip8_key('q'), None);
    }

    #[test]
    fn test_key_validation() {
        // Test valid keys
        for i in 0..=15 {
            assert!(is_valid_key(i));
        }

        // Test invalid keys
        assert!(!is_valid_key(16));
        assert!(!is_valid_key(255));
    }

    #[test]
    fn test_key_press_release() {
        let mut input = Input::new();

        // Press key 5
        input.press_key(5).unwrap();
        assert!(input.is_key_pressed(5).unwrap());
        assert!(!input.is_key_pressed(4).unwrap());

        // Release key 5
        input.release_key(5).unwrap();
        assert!(!input.is_key_pressed(5).unwrap());
    }

    #[test]
    fn test_invalid_key_operations() {
        let mut input = Input::new();

        // Test invalid key press
        assert!(matches!(
            input.press_key(16),
            Err(InputError::InvalidKey { key: 16 })
        ));

        // Test invalid key release
        assert!(matches!(
            input.release_key(16),
            Err(InputError::InvalidKey { key: 16 })
        ));

        // Test invalid key check
        assert!(matches!(
            input.is_key_pressed(16),
            Err(InputError::InvalidKey { key: 16 })
        ));
    }

    #[test]
    fn test_char_input_processing() {
        let mut input = Input::new();

        // Process character input
        input.process_char_input('q');
        assert!(input.is_key_pressed(4).unwrap()); // 'q' maps to Key4

        // Process character release
        input.process_char_release('q');
        assert!(!input.is_key_pressed(4).unwrap());
    }

    #[test]
    fn test_clear_all_keys() {
        let mut input = Input::new();

        // Press several keys
        input.press_key(1).unwrap();
        input.press_key(5).unwrap();
        input.press_key(10).unwrap();

        assert!(input.is_key_pressed(1).unwrap());
        assert!(input.is_key_pressed(5).unwrap());
        assert!(input.is_key_pressed(10).unwrap());

        // Clear all keys
        input.clear_all_keys();

        // All keys should be released
        for i in 0..16 {
            assert!(!input.is_key_pressed(i).unwrap());
        }
    }

    #[test]
    fn test_get_pressed_keys() {
        let mut input = Input::new();

        // No keys pressed initially
        assert!(input.get_pressed_keys().is_empty());

        // Press some keys
        input.press_key(2).unwrap();
        input.press_key(7).unwrap();
        input.press_key(15).unwrap();

        let pressed = input.get_pressed_keys();
        assert_eq!(pressed.len(), 3);
        assert!(pressed.contains(&2));
        assert!(pressed.contains(&7));
        assert!(pressed.contains(&15));
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
        input.press_key(1).unwrap();
        input.press_key(5).unwrap();

        let stats = input.get_stats();
        assert_eq!(stats.pressed_keys, 2);
    }

    #[test]
    fn test_try_get_key_press() {
        let mut input = Input::new();

        // Add key to buffer
        input.process_char_input('w'); // Maps to Key5

        // Should return the key (either from pressed state or buffer)
        assert_eq!(input.try_get_key_press().unwrap(), 5);

        // Release the key and clear buffer, then should return None
        input.process_char_release('w');
        input.clear_input_buffer();
        assert!(input.try_get_key_press().is_none());
    }
}
