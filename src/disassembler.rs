//! Simple CHIP-8 disassembler for analyzing ROMs
//!
//! This module provides basic disassembly capabilities to help understand
//! what instructions a ROM uses, which is useful for implementing missing opcodes.

use crate::constants::PROGRAM_START_ADDR;
use crate::instruction::{DecodeError, Instruction, decode_opcode};
use crate::memory::{Memory, MemoryError};
use thiserror::Error;

/// Disassembly errors
#[derive(Debug, Error)]
pub enum DisassemblyError {
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error("Instruction decode error: {0}")]
    Decode(#[from] DecodeError),
}

/// Disassemble a ROM and return a list of instructions with their addresses
pub fn disassemble_rom(memory: &Memory) -> Result<Vec<DisassembledInstruction>, DisassemblyError> {
    let mut instructions = Vec::new();
    let mut addr = PROGRAM_START_ADDR;

    // Scan through memory looking for valid instructions
    while addr < 0x1000 - 1 {
        // Try to read a word at this address
        if let Ok(opcode) = memory.read_word(addr) {
            // If it's all zeros, we've probably hit the end of the ROM
            if opcode == 0x0000 {
                break;
            }

            // Try to decode the opcode - if it fails, we've probably hit sprite data
            match decode_opcode(opcode) {
                Ok(decoded) => {
                    let instruction = DisassembledInstruction {
                        address: addr,
                        opcode,
                        instruction: decoded,
                    };
                    instructions.push(instruction);
                }
                Err(_) => {
                    // Hit unknown instruction (probably sprite data) - stop disassembly
                    break;
                }
            }

            addr += 2;
        } else {
            break;
        }
    }

    Ok(instructions)
}

/// Represents a disassembled instruction with its address and decoded form
#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    pub address: u16,
    pub opcode: u16,
    pub instruction: Instruction,
}

impl DisassembledInstruction {
    /// Get the mnemonic for this instruction
    pub fn mnemonic(&self) -> String {
        self.instruction.mnemonic()
    }
}

/// Print disassembly to stdout
pub fn print_disassembly(instructions: &[DisassembledInstruction]) {
    println!("Address  Opcode  Mnemonic");
    println!("------------------------");
    for instruction in instructions {
        println!(
            "{:04X}     {:04X}    {}",
            instruction.address,
            instruction.opcode,
            instruction.mnemonic()
        );
    }
}

/// Analyze what instruction families are used in a ROM
pub fn analyze_instruction_usage(instructions: &[DisassembledInstruction]) -> InstructionAnalysis {
    let mut analysis = InstructionAnalysis::default();

    for dis_instruction in instructions {
        match &dis_instruction.instruction {
            Instruction::Cls => {
                analysis.system += 1;
                analysis.cls = true;
            }
            Instruction::Ret => {
                analysis.system += 1;
                analysis.ret = true;
            }
            Instruction::Sys { .. } => {
                analysis.system += 1;
                analysis.sys = true;
            }
            Instruction::Jump { .. } => analysis.jump += 1,
            Instruction::Call { .. } => analysis.call += 1,
            Instruction::JumpV0 { .. } => analysis.jump_v0 += 1,
            Instruction::SkipEqImm { .. } => analysis.skip_eq_imm += 1,
            Instruction::SkipNeImm { .. } => analysis.skip_ne_imm += 1,
            Instruction::SkipEqReg { .. } => analysis.skip_eq_reg += 1,
            Instruction::SkipNeReg { .. } => analysis.skip_ne_reg += 1,
            Instruction::LoadImm { .. } => analysis.load_imm += 1,
            Instruction::AddImm { .. } => analysis.add_imm += 1,
            Instruction::LoadReg { .. } => {
                analysis.arithmetic += 1;
                analysis.load_reg = true;
            }
            Instruction::OrReg { .. } => {
                analysis.arithmetic += 1;
                analysis.or_reg = true;
            }
            Instruction::AndReg { .. } => {
                analysis.arithmetic += 1;
                analysis.and_reg = true;
            }
            Instruction::XorReg { .. } => {
                analysis.arithmetic += 1;
                analysis.xor_reg = true;
            }
            Instruction::AddReg { .. } => {
                analysis.arithmetic += 1;
                analysis.add_reg = true;
            }
            Instruction::SubReg { .. } => {
                analysis.arithmetic += 1;
                analysis.sub_reg = true;
            }
            Instruction::SubnReg { .. } => {
                analysis.arithmetic += 1;
                analysis.subn_reg = true;
            }
            Instruction::ShrReg { .. } => {
                analysis.arithmetic += 1;
                analysis.shr_reg = true;
            }
            Instruction::ShlReg { .. } => {
                analysis.arithmetic += 1;
                analysis.shl_reg = true;
            }
            Instruction::SetIndex { .. } => analysis.set_index += 1,
            Instruction::Random { .. } => analysis.random += 1,
            Instruction::Draw { .. } => analysis.draw += 1,
            Instruction::SkipKeyPressed { .. } | Instruction::SkipKeyNotPressed { .. } => {
                analysis.input += 1
            }
            Instruction::LoadDelayTimer { .. } => {
                analysis.misc += 1;
                analysis.load_delay = true;
            }
            Instruction::SetDelayTimer { .. } => {
                analysis.misc += 1;
                analysis.set_delay = true;
            }
            Instruction::SetSoundTimer { .. } => {
                analysis.misc += 1;
                analysis.set_sound = true;
            }
            Instruction::WaitKey { .. } => {
                analysis.misc += 1;
                analysis.wait_key = true;
            }
            Instruction::AddIndex { .. } => {
                analysis.misc += 1;
                analysis.add_index = true;
            }
            Instruction::LoadFont { .. } => {
                analysis.misc += 1;
                analysis.font_sprite = true;
            }
            Instruction::StoreBcd { .. } => {
                analysis.misc += 1;
                analysis.bcd = true;
            }
            Instruction::StoreRegisters { .. } => {
                analysis.misc += 1;
                analysis.store_regs = true;
            }
            Instruction::LoadRegisters { .. } => {
                analysis.misc += 1;
                analysis.load_regs = true;
            }
        }
    }

    analysis
}

/// Analysis of instruction usage in a ROM
#[derive(Debug, Default)]
pub struct InstructionAnalysis {
    // Instruction family counts
    pub system: usize,
    pub jump: usize,
    pub call: usize,
    pub skip_eq_imm: usize,
    pub skip_ne_imm: usize,
    pub skip_eq_reg: usize,
    pub load_imm: usize,
    pub add_imm: usize,
    pub arithmetic: usize,
    pub skip_ne_reg: usize,
    pub set_index: usize,
    pub jump_v0: usize,
    pub random: usize,
    pub draw: usize,
    pub input: usize,
    pub misc: usize,
    pub unknown: usize,

    // Specific instruction flags
    pub cls: bool,
    pub ret: bool,
    pub sys: bool,
    pub load_reg: bool,
    pub or_reg: bool,
    pub and_reg: bool,
    pub xor_reg: bool,
    pub add_reg: bool,
    pub sub_reg: bool,
    pub shr_reg: bool,
    pub subn_reg: bool,
    pub shl_reg: bool,
    pub load_delay: bool,
    pub wait_key: bool,
    pub set_delay: bool,
    pub set_sound: bool,
    pub add_index: bool,
    pub font_sprite: bool,
    pub bcd: bool,
    pub store_regs: bool,
    pub load_regs: bool,
}

impl InstructionAnalysis {
    /// Print a summary of instruction usage
    pub fn print_summary(&self) {
        println!("\nInstruction Analysis:");
        println!("====================");

        if self.system > 0 {
            println!("System instructions: {}", self.system);
        }
        if self.jump > 0 {
            println!("Jump instructions: {}", self.jump);
        }
        if self.call > 0 {
            println!("Call instructions: {}", self.call);
        }
        if self.skip_eq_imm > 0 {
            println!("Skip if equal (immediate): {}", self.skip_eq_imm);
        }
        if self.skip_ne_imm > 0 {
            println!("Skip if not equal (immediate): {}", self.skip_ne_imm);
        }
        if self.skip_eq_reg > 0 {
            println!("Skip if equal (register): {}", self.skip_eq_reg);
        }
        if self.load_imm > 0 {
            println!("Load immediate: {}", self.load_imm);
        }
        if self.add_imm > 0 {
            println!("Add immediate: {}", self.add_imm);
        }
        if self.arithmetic > 0 {
            println!("Arithmetic instructions: {}", self.arithmetic);
        }
        if self.skip_ne_reg > 0 {
            println!("Skip if not equal (register): {}", self.skip_ne_reg);
        }
        if self.set_index > 0 {
            println!("Set index register: {}", self.set_index);
        }
        if self.jump_v0 > 0 {
            println!("Jump with V0 offset: {}", self.jump_v0);
        }
        if self.random > 0 {
            println!("Random number: {}", self.random);
        }
        if self.draw > 0 {
            println!("Draw sprite: {}", self.draw);
        }
        if self.input > 0 {
            println!("Input instructions: {}", self.input);
        }
        if self.misc > 0 {
            println!("Miscellaneous F-type: {}", self.misc);
        }
        if self.unknown > 0 {
            println!("Unknown instructions: {}", self.unknown);
        }

        println!("\nSpecific Instructions Needed:");
        println!("=============================");
        if self.cls {
            println!("✓ CLS (Clear screen) - stubbed");
        }
        if self.ret {
            println!("✓ RET (Return) - implemented");
        }
        if self.sys {
            println!("- SYS (System call) - rare, may skip");
        }
        if self.load_reg {
            println!("- LD Vx, Vy (Load register)");
        }
        if self.or_reg {
            println!("- OR Vx, Vy (Bitwise OR)");
        }
        if self.and_reg {
            println!("- AND Vx, Vy (Bitwise AND)");
        }
        if self.xor_reg {
            println!("- XOR Vx, Vy (Bitwise XOR)");
        }
        if self.add_reg {
            println!("- ADD Vx, Vy (Add registers)");
        }
        if self.sub_reg {
            println!("- SUB Vx, Vy (Subtract)");
        }
        if self.shr_reg {
            println!("- SHR Vx (Shift right)");
        }
        if self.subn_reg {
            println!("- SUBN Vx, Vy (Subtract reverse)");
        }
        if self.shl_reg {
            println!("- SHL Vx (Shift left)");
        }
        if self.font_sprite {
            println!("- LD F, Vx (Load font sprite)");
        }
        if self.bcd {
            println!("- LD B, Vx (Binary-coded decimal)");
        }
        if self.store_regs {
            println!("- LD [I], Vx (Store registers)");
        }
        if self.load_regs {
            println!("- LD Vx, [I] (Load registers)");
        }
        if self.add_index {
            println!("- ADD I, Vx (Add to index)");
        }
        if self.set_delay {
            println!("- LD DT, Vx (Set delay timer)");
        }
        if self.set_sound {
            println!("- LD ST, Vx (Set sound timer)");
        }
        if self.load_delay {
            println!("- LD Vx, DT (Load delay timer)");
        }
        if self.wait_key {
            println!("- LD Vx, K (Wait for key)");
        }
    }
}
