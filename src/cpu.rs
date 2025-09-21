//! CHIP-8 CPU Implementation
//!
//! The CPU handles instruction fetch, decode, and execution. It maintains all processor
//! state including registers, program counter, stack, and timers.

use crate::constants::*;
use crate::display::{DisplayBus, DisplayError};
use crate::input::{InputBus, InputError};
use crate::instruction::{DecodeError, Instruction, decode_opcode};
use crate::memory::{MemoryBus, MemoryError};
use thiserror::Error;

/// CPU errors
#[derive(Debug, Error)]
pub enum CpuError {
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error("Instruction decode error: {0}")]
    Decode(#[from] DecodeError),

    #[error("Display error: {0}")]
    Display(#[from] DisplayError),

    #[error("Input error: {0}")]
    Input(#[from] InputError),

    #[error("Stack overflow - cannot push more than {max_depth} levels")]
    StackOverflow { max_depth: usize },

    #[error("Stack underflow - cannot return from subroutine (stack empty)")]
    StackUnderflow,

    #[error("Invalid register index: {register} (must be 0-15)")]
    InvalidRegister { register: usize },

    #[error("Instruction {instruction:#06x} at {addr:#06x} failed: {source}")]
    InstructionExecutionFailed {
        instruction: u16,
        addr: u16,
        source: Box<CpuError>,
    },

    #[error("Program counter out of bounds: {pc:#06x}")]
    InvalidProgramCounter { pc: u16 },
}

/// CPU execution state
#[derive(Debug, Clone, PartialEq)]
pub enum CpuState {
    /// Normal execution - fetch, decode, execute instructions
    Running,
    /// Waiting for a key press - stores which register (Vx) to store the key in
    WaitingForKey { vx: usize },
}

/// CHIP-8 CPU state
pub struct Cpu {
    /// 16 general-purpose 8-bit registers (V0-VF)
    /// VF is used as a flag register by some instructions
    v: [u8; NUM_REGISTERS],

    /// 16-bit index register (I)
    /// Used for memory operations
    i: u16,

    /// Program counter - points to current instruction
    pc: u16,

    /// Stack pointer - points to current stack level
    sp: u8,

    /// Call stack - stores return addresses for subroutines
    stack: [u16; STACK_SIZE],

    /// Delay timer - decrements at 60Hz until it reaches 0
    delay_timer: u8,

    /// Sound timer - decrements at 60Hz, beeps while > 0
    sound_timer: u8,

    /// Current execution state
    state: CpuState,
}

impl Cpu {
    /// Create a new CPU with default state
    pub fn new() -> Self {
        Self {
            v: [0; NUM_REGISTERS],
            i: 0,
            pc: PROGRAM_START_ADDR,
            sp: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            state: CpuState::Running,
        }
    }

    /// Reset CPU to initial state
    pub fn reset(&mut self) {
        self.v.fill(0);
        self.i = 0;
        self.pc = PROGRAM_START_ADDR;
        self.sp = 0;
        self.stack.fill(0);
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.state = CpuState::Running;
    }

    /// Execute one CPU cycle based on current execution state
    pub fn execute_cycle<M: MemoryBus, D: DisplayBus, I: InputBus>(
        &mut self,
        memory: &mut M,
        display: &mut D,
        input: &mut I,
    ) -> Result<(), CpuError> {
        match self.state {
            CpuState::Running => {
                // Normal execution: fetch, decode, execute
                let instruction_addr = self.pc;
                let instruction = self.fetch_instruction(memory)?;

                self.execute_instruction(instruction, memory, display, input)
                    .map_err(|err| CpuError::InstructionExecutionFailed {
                        instruction,
                        addr: instruction_addr,
                        source: Box::new(err),
                    })
            }
            CpuState::WaitingForKey { vx } => {
                // Blocked on key input - check if key is now available
                match input.try_get_key_press() {
                    Some(key) => {
                        self.v[vx] = key;
                        self.state = CpuState::Running;
                        Ok(())
                    }
                    None => {
                        // Still waiting - do nothing this cycle
                        Ok(())
                    }
                }
            }
        }
    }

    /// Fetch a 16-bit instruction from memory at current PC
    ///
    /// # Fetch Contract
    /// This method implements the CHIP-8 fetch contract:
    /// 1. Read 16-bit instruction from memory at current PC
    /// 2. Advance PC by 2 bytes (to next instruction)
    /// 3. Return the instruction for execution
    ///
    /// The PC advancement is UNCONDITIONAL - it always happens during fetch.
    /// Instructions that need to modify PC (jumps, calls, returns) will simply
    /// set PC to their target address, overriding the fetch advancement.
    ///
    /// This design keeps fetch/execute separation clean and predictable.
    fn fetch_instruction<M: MemoryBus>(&mut self, memory: &M) -> Result<u16, CpuError> {
        // Validate PC is in valid range
        if self.pc as usize >= MEMORY_SIZE - 1 {
            return Err(CpuError::InvalidProgramCounter { pc: self.pc });
        }

        // Read 16-bit instruction (big-endian)
        let high_byte = memory.read_byte(self.pc)?;
        let low_byte = memory.read_byte(self.pc + 1)?;
        let instruction = ((high_byte as u16) << 8) | (low_byte as u16);

        // Advance PC by 2 (part of fetch contract - ALWAYS happens)
        self.pc += 2;

        Ok(instruction)
    }

    /// Decode and execute an instruction
    fn execute_instruction<M: MemoryBus, D: DisplayBus, I: InputBus>(
        &mut self,
        opcode: u16,
        memory: &mut M,
        display: &mut D,
        input: &mut I,
    ) -> Result<(), CpuError> {
        // Decode the instruction using centralized decoding
        let instruction = decode_opcode(opcode)?;

        // Execute based on the decoded instruction
        match instruction {
            Instruction::Cls => {
                display.clear();
                Ok(())
            }
            Instruction::Ret => self.return_from_subroutine(),
            Instruction::Sys { .. } => {
                // System calls are rarely used in modern CHIP-8 programs
                Ok(())
            }
            Instruction::Jump { addr } => {
                self.pc = addr;
                Ok(())
            }
            Instruction::Call { addr } => self.call_subroutine(addr),
            Instruction::JumpV0 { addr } => {
                self.pc = addr + (self.v[0] as u16);
                Ok(())
            }
            Instruction::SkipEqImm { vx, value } => {
                if self.v[vx] == value {
                    self.pc += 2;
                }
                Ok(())
            }
            Instruction::SkipNeImm { vx, value } => {
                if self.v[vx] != value {
                    self.pc += 2;
                }
                Ok(())
            }
            Instruction::SkipEqReg { vx, vy } => {
                if self.v[vx] == self.v[vy] {
                    self.pc += 2;
                }
                Ok(())
            }
            Instruction::SkipNeReg { vx, vy } => {
                if self.v[vx] != self.v[vy] {
                    self.pc += 2;
                }
                Ok(())
            }
            Instruction::LoadImm { vx, value } => {
                self.v[vx] = value;
                Ok(())
            }
            Instruction::LoadReg { vx, vy } => {
                self.v[vx] = self.v[vy];
                Ok(())
            }
            Instruction::SetIndex { addr } => {
                self.i = addr;
                Ok(())
            }
            Instruction::AddImm { vx, value } => {
                self.v[vx] = self.v[vx].wrapping_add(value);
                Ok(())
            }
            Instruction::AddReg { vx, vy } => {
                let (result, overflow) = self.v[vx].overflowing_add(self.v[vy]);
                self.v[vx] = result;
                self.v[0xF] = if overflow { 1 } else { 0 };
                Ok(())
            }
            Instruction::SubReg { vx, vy } => {
                let (result, borrow) = self.v[vx].overflowing_sub(self.v[vy]);
                self.v[vx] = result;
                self.v[0xF] = if borrow { 0 } else { 1 };
                Ok(())
            }
            Instruction::SubnReg { vx, vy } => {
                let (result, borrow) = self.v[vy].overflowing_sub(self.v[vx]);
                self.v[vx] = result;
                self.v[0xF] = if borrow { 0 } else { 1 };
                Ok(())
            }
            Instruction::OrReg { vx, vy } => {
                self.v[vx] |= self.v[vy];
                Ok(())
            }
            Instruction::AndReg { vx, vy } => {
                self.v[vx] &= self.v[vy];
                Ok(())
            }
            Instruction::XorReg { vx, vy } => {
                self.v[vx] ^= self.v[vy];
                Ok(())
            }
            Instruction::ShrReg { vx } => {
                self.v[0xF] = self.v[vx] & 0x01;
                self.v[vx] >>= 1;
                Ok(())
            }
            Instruction::ShlReg { vx } => {
                self.v[0xF] = (self.v[vx] & 0x80) >> 7;
                self.v[vx] <<= 1;
                Ok(())
            }
            Instruction::Draw { vx, vy, n } => {
                // Get coordinates from registers
                let x = self.v[vx];
                let y = self.v[vy];

                // Read sprite data from memory starting at I register
                let mut sprite_data = Vec::new();
                for i in 0..n {
                    let byte = memory.read_byte(self.i + i as u16)?;
                    sprite_data.push(byte);
                }

                // Draw sprite and get collision flag
                let collision = display.draw_sprite(x, y, &sprite_data)?;
                self.v[0xF] = if collision { 1 } else { 0 };
                Ok(())
            }
            Instruction::SkipKeyPressed { vx } => {
                let key = self.v[vx] & 0x0F;
                if input.is_key_pressed(key)? {
                    self.pc += 2; // Skip next instruction
                }
                Ok(())
            }
            Instruction::SkipKeyNotPressed { vx } => {
                let key = self.v[vx] & 0x0F;
                if !input.is_key_pressed(key)? {
                    self.pc += 2; // Skip next instruction
                }
                Ok(())
            }
            Instruction::Random { vx, mask } => {
                // TODO: Use proper random number generator
                let random_value = 0x42; // Placeholder
                self.v[vx] = random_value & mask;
                Ok(())
            }
            Instruction::LoadDelayTimer { vx } => {
                self.v[vx] = self.delay_timer;
                Ok(())
            }
            Instruction::SetDelayTimer { vx } => {
                self.delay_timer = self.v[vx];
                Ok(())
            }
            Instruction::SetSoundTimer { vx } => {
                self.sound_timer = self.v[vx];
                Ok(())
            }
            Instruction::WaitKey { vx } => {
                // Try to get a key press immediately
                match input.try_get_key_press() {
                    Some(key) => {
                        self.v[vx] = key;
                        Ok(())
                    }
                    None => {
                        // No key available - transition to waiting state
                        self.state = CpuState::WaitingForKey { vx };
                        Ok(())
                    }
                }
            }
            Instruction::AddIndex { vx } => {
                self.i += self.v[vx] as u16;
                Ok(())
            }
            Instruction::LoadFont { vx } => {
                // Font sprites are stored starting at FONT_START_ADDR, each is 5 bytes
                self.i = FONT_START_ADDR + (self.v[vx] as u16 * 5);
                Ok(())
            }
            Instruction::StoreBcd { .. } => {
                // TODO: Implement BCD conversion
                Ok(())
            }
            Instruction::StoreRegisters { .. } => {
                // TODO: Implement register storage
                Ok(())
            }
            Instruction::LoadRegisters { .. } => {
                // TODO: Implement register loading
                Ok(())
            }
        }
    }

    /// Call a subroutine at the given address
    fn call_subroutine(&mut self, addr: u16) -> Result<(), CpuError> {
        if self.sp as usize >= STACK_SIZE {
            return Err(CpuError::StackOverflow {
                max_depth: STACK_SIZE,
            });
        }

        // Push current PC onto stack
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;

        // Jump to subroutine
        self.pc = addr;
        Ok(())
    }

    /// Return from current subroutine
    fn return_from_subroutine(&mut self) -> Result<(), CpuError> {
        if self.sp == 0 {
            return Err(CpuError::StackUnderflow);
        }

        // Pop return address from stack
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
        Ok(())
    }

    /// Update timers (should be called at 60Hz)
    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    /// Get the current value of a register
    pub fn get_register(&self, register: usize) -> Result<u8, CpuError> {
        if register >= NUM_REGISTERS {
            return Err(CpuError::InvalidRegister { register });
        }
        Ok(self.v[register])
    }

    /// Set the value of a register
    pub fn set_register(&mut self, register: usize, value: u8) -> Result<(), CpuError> {
        if register >= NUM_REGISTERS {
            return Err(CpuError::InvalidRegister { register });
        }
        self.v[register] = value;
        Ok(())
    }

    /// Get current program counter
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    /// Get current index register value
    pub fn get_index(&self) -> u16 {
        self.i
    }

    /// Get current delay timer value
    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }

    /// Set delay timer value
    pub fn set_delay_timer(&mut self, value: u8) {
        self.delay_timer = value;
    }

    /// Get current sound timer value
    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }

    /// Set sound timer value
    pub fn set_sound_timer(&mut self, value: u8) {
        self.sound_timer = value;
    }

    /// Check if sound should be playing (sound timer > 0)
    pub fn should_beep(&self) -> bool {
        self.sound_timer > 0
    }

    /// Get current CPU execution state
    pub fn get_state(&self) -> &CpuState {
        &self.state
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;
    use crate::{Display, MockInput};

    #[test]
    fn test_cpu_initialization() {
        let cpu = Cpu::new();

        // Check initial state
        assert_eq!(cpu.get_pc(), PROGRAM_START_ADDR);
        assert_eq!(cpu.get_index(), 0);
        assert_eq!(cpu.get_delay_timer(), 0);
        assert_eq!(cpu.get_sound_timer(), 0);

        // Check all registers are zero
        for i in 0..NUM_REGISTERS {
            assert_eq!(cpu.get_register(i).unwrap(), 0);
        }
    }

    #[test]
    fn test_reset() {
        let mut cpu = Cpu::new();

        // Modify state
        cpu.set_register(5, 0x42).unwrap();
        cpu.i = 0x300;
        cpu.pc = 0x400;
        cpu.set_delay_timer(30);

        // Reset and verify
        cpu.reset();
        assert_eq!(cpu.get_pc(), PROGRAM_START_ADDR);
        assert_eq!(cpu.get_index(), 0);
        assert_eq!(cpu.get_delay_timer(), 0);
        assert_eq!(cpu.get_register(5).unwrap(), 0);
    }

    #[test]
    fn test_load_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = crate::Display::new();
        let mut input = MockInput::new();

        // LD V3, 0x42 (instruction: 0x6342)
        memory.write_word(PROGRAM_START_ADDR, 0x6342).unwrap();

        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        assert_eq!(cpu.get_register(3).unwrap(), 0x42);
        assert_eq!(cpu.get_pc(), PROGRAM_START_ADDR + 2);
    }

    #[test]
    fn test_add_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = crate::Display::new();
        let mut input = MockInput::new();

        // Set V2 to 0x10
        cpu.v[2] = 0x10;

        // ADD V2, 0x25 (instruction: 0x7225)
        memory.write_word(PROGRAM_START_ADDR, 0x7225).unwrap();

        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        assert_eq!(cpu.get_register(2).unwrap(), 0x35);
    }

    #[test]
    fn test_jump_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = crate::Display::new();
        let mut input = MockInput::new();

        // JP 0x300 (instruction: 0x1300)
        memory.write_word(PROGRAM_START_ADDR, 0x1300).unwrap();

        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        assert_eq!(cpu.get_pc(), 0x300);
    }

    #[test]
    fn test_call_and_return() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = crate::Display::new();
        let mut input = MockInput::new();

        // CALL 0x300 (instruction: 0x2300)
        memory.write_word(PROGRAM_START_ADDR, 0x2300).unwrap();

        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // Should jump to 0x300 and push return address
        assert_eq!(cpu.get_pc(), 0x300);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], PROGRAM_START_ADDR + 2);

        // RET (instruction: 0x00EE)
        cpu.return_from_subroutine().unwrap();

        // Should return to original location
        assert_eq!(cpu.get_pc(), PROGRAM_START_ADDR + 2);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_set_index_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = crate::Display::new();
        let mut input = MockInput::new();

        // LD I, 0x300 (instruction: 0xA300)
        memory.write_word(PROGRAM_START_ADDR, 0xA300).unwrap();

        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        assert_eq!(cpu.get_index(), 0x300);
    }

    #[test]
    fn test_timer_updates() {
        let mut cpu = Cpu::new();

        cpu.set_delay_timer(5);
        cpu.set_sound_timer(3);

        // First update
        cpu.update_timers();
        assert_eq!(cpu.get_delay_timer(), 4);
        assert_eq!(cpu.get_sound_timer(), 2);
        assert!(cpu.should_beep());

        // Continue until sound timer reaches 0
        cpu.update_timers();
        cpu.update_timers();
        assert_eq!(cpu.get_sound_timer(), 0);
        assert!(!cpu.should_beep());
    }

    #[test]
    fn test_unknown_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Write an unknown instruction at program start
        memory.write_word(PROGRAM_START_ADDR, 0xF123).unwrap();

        let result = cpu.execute_cycle(&mut memory, &mut display, &mut input);

        // Should fail with execution error
        assert!(result.is_err());

        // PC should still have advanced (part of fetch contract)
        assert_eq!(cpu.get_pc(), PROGRAM_START_ADDR + 2);
    }

    #[test]
    fn test_cls_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = crate::Display::new();
        let mut input = MockInput::new();

        // Set some pixels first
        display.set_pixel(10, 5, true);
        display.set_pixel(20, 15, true);
        assert!(display.get_pixel(10, 5));
        assert!(display.get_pixel(20, 15));

        // CLS instruction (0x00E0)
        memory.write_word(PROGRAM_START_ADDR, 0x00E0).unwrap();

        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // All pixels should be cleared
        assert!(!display.get_pixel(10, 5));
        assert!(!display.get_pixel(20, 15));
    }

    #[test]
    fn test_draw_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = crate::Display::new();
        let mut input = MockInput::new();

        // Set up sprite data in memory
        let sprite_addr = 0x300;
        let sprite_data = [0b11110000, 0b10010000]; // 4x2 rectangle
        memory.write_byte(sprite_addr, sprite_data[0]).unwrap();
        memory.write_byte(sprite_addr + 1, sprite_data[1]).unwrap();

        // Set up CPU state for drawing
        cpu.v[0] = 10; // X coordinate
        cpu.v[1] = 5; // Y coordinate
        cpu.i = sprite_addr;

        // DRW V0, V1, 2 (instruction: 0xD012)
        memory.write_word(PROGRAM_START_ADDR, 0xD012).unwrap();

        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // Verify sprite was drawn correctly
        assert!(display.get_pixel(10, 5)); // Top-left
        assert!(display.get_pixel(13, 5)); // Top-right
        assert!(!display.get_pixel(14, 5)); // Should be off
        assert!(display.get_pixel(10, 6)); // Bottom-left
        assert!(!display.get_pixel(11, 6)); // Should be off (gap in sprite)
        assert!(display.get_pixel(13, 6)); // Bottom-right

        // No collision should occur (VF = 0)
        assert_eq!(cpu.get_register(0xF).unwrap(), 0);
    }

    #[test]
    fn test_draw_instruction_collision() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = crate::Display::new();
        let mut input = MockInput::new();

        // Set up existing pixel that will cause collision
        display.set_pixel(10, 5, true);

        // Set up sprite data that will overlap existing pixel
        let sprite_addr = 0x300;
        memory.write_byte(sprite_addr, 0b10000000).unwrap(); // Single pixel at top-left

        // Set up CPU state
        cpu.v[0] = 10; // X coordinate (matches existing pixel)
        cpu.v[1] = 5; // Y coordinate (matches existing pixel)
        cpu.i = sprite_addr;

        // DRW V0, V1, 1 (instruction: 0xD011)
        memory.write_word(PROGRAM_START_ADDR, 0xD011).unwrap();

        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // Pixel should be turned off due to XOR
        assert!(!display.get_pixel(10, 5));

        // Collision should be detected (VF = 1)
        assert_eq!(cpu.get_register(0xF).unwrap(), 1);
    }

    #[test]
    fn test_skip_key_pressed_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Set V0 to key 5
        cpu.set_register(0, 5).unwrap();

        // SKP V0 instruction (0xE09E)
        memory.write_word(PROGRAM_START_ADDR, 0xE09E).unwrap();

        // Key 5 is NOT pressed - should not skip
        let initial_pc = cpu.get_pc();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();
        assert_eq!(cpu.get_pc(), initial_pc + 2); // Normal increment

        // Test with key pressed (use fresh CPU instance)
        let mut cpu2 = Cpu::new();
        cpu2.set_register(0, 5).unwrap();
        input.press_key(5).unwrap();

        let initial_pc2 = cpu2.get_pc();
        cpu2.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();
        assert_eq!(cpu2.get_pc(), initial_pc2 + 4); // Should skip
    }

    #[test]
    fn test_skip_key_not_pressed_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Set V1 to key 7
        cpu.set_register(1, 7).unwrap();

        // SKNP V1 instruction (0xE1A1)
        memory.write_word(PROGRAM_START_ADDR, 0xE1A1).unwrap();

        // Key 7 is NOT pressed - should skip
        let initial_pc = cpu.get_pc();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();
        assert_eq!(cpu.get_pc(), initial_pc + 4); // Should skip

        // Test with key pressed (use fresh CPU instance)
        let mut cpu2 = Cpu::new();
        cpu2.set_register(1, 7).unwrap();
        input.press_key(7).unwrap();

        let initial_pc2 = cpu2.get_pc();
        cpu2.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();
        assert_eq!(cpu2.get_pc(), initial_pc2 + 2); // Should not skip
    }

    #[test]
    fn test_wait_for_key_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // LD V2, K instruction (0xF20A)
        memory.write_word(PROGRAM_START_ADDR, 0xF20A).unwrap();

        let initial_pc = cpu.get_pc();

        // No key available - should transition to WaitingForKey state
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // PC advances as part of instruction fetch, but CPU is now waiting for key
        assert_eq!(cpu.get_pc(), initial_pc + 2);
        assert_eq!(*cpu.get_state(), CpuState::WaitingForKey { vx: 2 });

        // Next cycle should still be waiting (no key available)
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();
        assert_eq!(*cpu.get_state(), CpuState::WaitingForKey { vx: 2 });

        // Press key 0xB and cycle again - should resume execution
        input.press_key(0xB).unwrap();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // Should have stored key value in V2 and resumed running state
        assert_eq!(cpu.get_register(2).unwrap(), 0xB);
        assert_eq!(*cpu.get_state(), CpuState::Running);
    }

    #[test]
    fn test_skip_key_pressed_skips_when_pressed() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Set V0 = 0x5 and press key 0x5
        cpu.set_register(0, 0x5).unwrap();
        input.press_key(0x5).unwrap();

        // SKP V0 instruction (0xE09E)
        memory.write_word(PROGRAM_START_ADDR, 0xE09E).unwrap();

        let initial_pc = cpu.get_pc();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // PC should have advanced by 4 (2 for fetch + 2 for skip)
        assert_eq!(cpu.get_pc(), initial_pc + 4);
    }

    #[test]
    fn test_skip_key_pressed_no_skip_when_not_pressed() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Set V0 = 0x5 but don't press key 0x5
        cpu.set_register(0, 0x5).unwrap();

        // SKP V0 instruction (0xE09E)
        memory.write_word(PROGRAM_START_ADDR, 0xE09E).unwrap();

        let initial_pc = cpu.get_pc();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // PC should have advanced by 2 (only fetch, no skip)
        assert_eq!(cpu.get_pc(), initial_pc + 2);
    }

    #[test]
    fn test_skip_key_not_pressed_skips_when_not_pressed() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Set V0 = 0x5 but don't press key 0x5
        cpu.set_register(0, 0x5).unwrap();

        // SKNP V0 instruction (0xE0A1)
        memory.write_word(PROGRAM_START_ADDR, 0xE0A1).unwrap();

        let initial_pc = cpu.get_pc();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // PC should have advanced by 4 (2 for fetch + 2 for skip)
        assert_eq!(cpu.get_pc(), initial_pc + 4);
    }

    #[test]
    fn test_skip_key_not_pressed_no_skip_when_pressed() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Set V0 = 0x5 and press key 0x5
        cpu.set_register(0, 0x5).unwrap();
        input.press_key(0x5).unwrap();

        // SKNP V0 instruction (0xE0A1)
        memory.write_word(PROGRAM_START_ADDR, 0xE0A1).unwrap();

        let initial_pc = cpu.get_pc();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // PC should have advanced by 2 (only fetch, no skip)
        assert_eq!(cpu.get_pc(), initial_pc + 2);
    }

    #[test]
    fn test_key_instructions_mask_high_bits() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Set V0 = 0x15 (high bits should be masked to 0x5)
        cpu.set_register(0, 0x15).unwrap();
        input.press_key(0x5).unwrap(); // Press the masked key

        // SKP V0 instruction (0xE09E) - should treat 0x15 as 0x5
        memory.write_word(PROGRAM_START_ADDR, 0xE09E).unwrap();

        let initial_pc = cpu.get_pc();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // Should skip because 0x15 & 0x0F = 0x5, and key 0x5 is pressed
        assert_eq!(cpu.get_pc(), initial_pc + 4);
    }

    #[test]
    fn test_wait_key_immediate_return_when_available() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // Press key 0xA before executing instruction
        input.press_key(0xA).unwrap();

        // LD V3, K instruction (0xF30A)
        memory.write_word(PROGRAM_START_ADDR, 0xF30A).unwrap();

        let initial_pc = cpu.get_pc();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        // Should store key immediately and stay in running state
        assert_eq!(cpu.get_register(3).unwrap(), 0xA);
        assert_eq!(*cpu.get_state(), CpuState::Running);
        assert_eq!(cpu.get_pc(), initial_pc + 2);
    }

    #[test]
    fn test_wait_key_state_persistence() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(true);
        let mut display = Display::new();
        let mut input = MockInput::new();

        // LD V1, K instruction (0xF10A)
        memory.write_word(PROGRAM_START_ADDR, 0xF10A).unwrap();

        // First cycle - no key available, should enter waiting state
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();
        assert_eq!(*cpu.get_state(), CpuState::WaitingForKey { vx: 1 });

        // Multiple cycles with no key - should remain in waiting state
        for _ in 0..5 {
            cpu.execute_cycle(&mut memory, &mut display, &mut input)
                .unwrap();
            assert_eq!(*cpu.get_state(), CpuState::WaitingForKey { vx: 1 });
        }

        // Press key and cycle - should resume and store key
        input.press_key(0xA).unwrap();
        cpu.execute_cycle(&mut memory, &mut display, &mut input)
            .unwrap();

        assert_eq!(cpu.get_register(1).unwrap(), 0xA);
        assert_eq!(*cpu.get_state(), CpuState::Running);
    }
}
