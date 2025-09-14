//! CHIP-8 CPU Implementation
//!
//! The CPU handles instruction fetch, decode, and execution. It maintains all processor
//! state including registers, program counter, stack, and timers.

use crate::constants::*;
use crate::memory::{MemoryBus, MemoryError};
use thiserror::Error;

/// CPU errors
#[derive(Debug, Error)]
pub enum CpuError {
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error("Stack overflow - cannot push more than {max_depth} levels")]
    StackOverflow { max_depth: usize },

    #[error("Stack underflow - cannot return from subroutine (stack empty)")]
    StackUnderflow,

    #[error("Invalid register index: {register} (must be 0-15)")]
    InvalidRegister { register: usize },

    #[error("Unknown instruction")]
    UnknownInstruction { instruction: u16 },

    #[error("Instruction {instruction:#06x} at {addr:#06x} failed: {source}")]
    InstructionExecutionFailed {
        instruction: u16,
        addr: u16,
        source: Box<CpuError>,
    },

    #[error("Program counter out of bounds: {pc:#06x}")]
    InvalidProgramCounter { pc: u16 },
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
    }

    /// Execute one CPU cycle: fetch, decode, and execute one instruction
    pub fn execute_cycle<M: MemoryBus>(&mut self, memory: &mut M) -> Result<(), CpuError> {
        // Keep instruction location local for error reporting
        let instruction_addr = self.pc;

        // Fetch instruction from memory at PC (advances PC as part of fetch contract)
        let instruction = self.fetch_instruction(memory)?;

        // Execute instruction - if it fails, wrap error with location info
        self.execute_instruction(instruction).map_err(|err| {
            CpuError::InstructionExecutionFailed {
                instruction,
                addr: instruction_addr,
                source: Box::new(err),
            }
        })?;

        Ok(())
    }

    /// Fetch a 16-bit instruction from memory at current PC
    fn fetch_instruction<M: MemoryBus>(&mut self, memory: &M) -> Result<u16, CpuError> {
        // Validate PC is in valid range
        if self.pc as usize >= MEMORY_SIZE - 1 {
            return Err(CpuError::InvalidProgramCounter { pc: self.pc });
        }

        // Read 16-bit instruction (big-endian)
        let high_byte = memory.read_byte(self.pc)?;
        let low_byte = memory.read_byte(self.pc + 1)?;
        let instruction = ((high_byte as u16) << 8) | (low_byte as u16);

        // Increment PC to next instruction
        self.pc += 2;

        Ok(instruction)
    }

    /// Decode and execute an instruction
    fn execute_instruction(&mut self, instruction: u16) -> Result<(), CpuError> {
        // Match on the first nibble to determine instruction family
        match instruction & 0xF000 {
            0x0000 => self.execute_system_instruction(instruction),
            0x1000 => self.execute_jump_instruction(instruction),
            0x2000 => self.execute_call_instruction(instruction),
            0x6000 => self.execute_load_instruction(instruction),
            0x7000 => self.execute_add_instruction(instruction),
            0xA000 => self.execute_set_index_instruction(instruction),
            0xD000 => self.execute_draw_instruction(instruction),
            _ => Err(CpuError::UnknownInstruction { instruction }),
        }
    }

    /// Execute system instructions (0x0???)
    fn execute_system_instruction(&mut self, instruction: u16) -> Result<(), CpuError> {
        match instruction {
            0x00E0 => {
                // CLS - Clear display
                // TODO: Implement when display module is ready
                Ok(())
            }
            0x00EE => {
                // RET - Return from subroutine
                self.return_from_subroutine()
            }
            _ => Err(CpuError::UnknownInstruction { instruction }),
        }
    }

    /// Execute jump instruction (1nnn)
    fn execute_jump_instruction(&mut self, instruction: u16) -> Result<(), CpuError> {
        // JP addr - Jump to location nnn
        let addr = instruction & 0x0FFF;
        self.pc = addr;
        Ok(())
    }

    /// Execute call instruction (2nnn)
    fn execute_call_instruction(&mut self, instruction: u16) -> Result<(), CpuError> {
        // CALL addr - Call subroutine at nnn
        let addr = instruction & 0x0FFF;
        self.call_subroutine(addr)
    }

    /// Execute load instruction (6xkk)
    fn execute_load_instruction(&mut self, instruction: u16) -> Result<(), CpuError> {
        // LD Vx, byte - Set Vx = kk
        let vx = ((instruction & 0x0F00) >> 8) as usize;
        let byte = (instruction & 0x00FF) as u8;

        if vx >= NUM_REGISTERS {
            return Err(CpuError::InvalidRegister { register: vx });
        }

        self.v[vx] = byte;
        Ok(())
    }

    /// Execute add instruction (7xkk)
    fn execute_add_instruction(&mut self, instruction: u16) -> Result<(), CpuError> {
        // ADD Vx, byte - Set Vx = Vx + kk
        let vx = ((instruction & 0x0F00) >> 8) as usize;
        let byte = (instruction & 0x00FF) as u8;

        if vx >= NUM_REGISTERS {
            return Err(CpuError::InvalidRegister { register: vx });
        }

        self.v[vx] = self.v[vx].wrapping_add(byte);
        Ok(())
    }

    /// Execute set index instruction (Annn)
    fn execute_set_index_instruction(&mut self, instruction: u16) -> Result<(), CpuError> {
        // LD I, addr - Set I = nnn
        let addr = instruction & 0x0FFF;
        self.i = addr;
        Ok(())
    }

    /// Execute draw instruction (Dxyn)
    fn execute_draw_instruction(&mut self, instruction: u16) -> Result<(), CpuError> {
        // DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I
        // at (Vx, Vy), set VF = collision
        let _vx = ((instruction & 0x0F00) >> 8) as usize;
        let _vy = ((instruction & 0x00F0) >> 4) as usize;
        let _n = (instruction & 0x000F) as u8;

        // TODO: Implement when display module is ready
        // For now, just clear collision flag
        self.v[0xF] = 0;
        Ok(())
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
        let mut memory = Memory::new(false);

        // LD V3, 0x42 (instruction: 0x6342)
        memory.write_word(PROGRAM_START_ADDR, 0x6342).unwrap();

        cpu.execute_cycle(&mut memory).unwrap();

        assert_eq!(cpu.get_register(3).unwrap(), 0x42);
        assert_eq!(cpu.get_pc(), PROGRAM_START_ADDR + 2);
    }

    #[test]
    fn test_add_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(false);

        // Set V2 = 0x10
        cpu.set_register(2, 0x10).unwrap();

        // ADD V2, 0x25 (instruction: 0x7225)
        memory.write_word(PROGRAM_START_ADDR, 0x7225).unwrap();

        cpu.execute_cycle(&mut memory).unwrap();

        assert_eq!(cpu.get_register(2).unwrap(), 0x35);
    }

    #[test]
    fn test_jump_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(false);

        // JP 0x300 (instruction: 0x1300)
        memory.write_word(PROGRAM_START_ADDR, 0x1300).unwrap();

        cpu.execute_cycle(&mut memory).unwrap();

        assert_eq!(cpu.get_pc(), 0x300);
    }

    #[test]
    fn test_call_and_return() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(false);

        // CALL 0x300 (instruction: 0x2300)
        memory.write_word(PROGRAM_START_ADDR, 0x2300).unwrap();

        cpu.execute_cycle(&mut memory).unwrap();

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
        let mut memory = Memory::new(false);

        // LD I, 0x300 (instruction: 0xA300)
        memory.write_word(PROGRAM_START_ADDR, 0xA300).unwrap();

        cpu.execute_cycle(&mut memory).unwrap();

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
        let mut memory = Memory::new(false);

        // Write an unknown instruction at program start
        memory.write_word(PROGRAM_START_ADDR, 0xF123).unwrap();

        let result = cpu.execute_cycle(&mut memory);

        // Should fail with execution error
        assert!(result.is_err());

        // PC should still have advanced (part of fetch contract)
        assert_eq!(cpu.get_pc(), PROGRAM_START_ADDR + 2);
    }
}
