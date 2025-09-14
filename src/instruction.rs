//! CHIP-8 Instruction Definitions and Decoding
//!
//! This module provides a centralized definition of all CHIP-8 instructions
//! and decoding logic. This ensures consistency between CPU execution and
//! disassembly, following the DRY principle.

use thiserror::Error;

/// CHIP-8 instruction decode errors
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Unknown instruction: {opcode:#06x}")]
    UnknownInstruction { opcode: u16 },

    #[error("Invalid register index: {register} (must be 0-15)")]
    InvalidRegister { register: usize },
}

/// All CHIP-8 instructions with their operands
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    // System instructions
    /// CLS - Clear the display
    Cls,

    /// RET - Return from a subroutine
    Ret,

    /// SYS addr - Jump to a machine code routine at addr (rarely used)
    Sys { addr: u16 },

    // Flow control
    /// JP addr - Jump to location addr
    Jump { addr: u16 },

    /// CALL addr - Call subroutine at addr
    Call { addr: u16 },

    /// JP V0, addr - Jump to location addr + V0
    JumpV0 { addr: u16 },

    // Conditional skips
    /// SE Vx, byte - Skip next instruction if Vx = byte
    SkipEqImm { vx: usize, value: u8 },

    /// SNE Vx, byte - Skip next instruction if Vx != byte
    SkipNeImm { vx: usize, value: u8 },

    /// SE Vx, Vy - Skip next instruction if Vx = Vy
    SkipEqReg { vx: usize, vy: usize },

    /// SNE Vx, Vy - Skip next instruction if Vx != Vy
    SkipNeReg { vx: usize, vy: usize },

    // Load/Store register operations
    /// LD Vx, byte - Set Vx = byte
    LoadImm { vx: usize, value: u8 },

    /// LD Vx, Vy - Set Vx = Vy
    LoadReg { vx: usize, vy: usize },

    /// LD I, addr - Set I = addr
    SetIndex { addr: u16 },

    // Arithmetic and logic
    /// ADD Vx, byte - Set Vx = Vx + byte
    AddImm { vx: usize, value: u8 },

    /// ADD Vx, Vy - Set Vx = Vx + Vy, set VF = carry
    AddReg { vx: usize, vy: usize },

    /// SUB Vx, Vy - Set Vx = Vx - Vy, set VF = NOT borrow
    SubReg { vx: usize, vy: usize },

    /// SUBN Vx, Vy - Set Vx = Vy - Vx, set VF = NOT borrow
    SubnReg { vx: usize, vy: usize },

    /// OR Vx, Vy - Set Vx = Vx OR Vy
    OrReg { vx: usize, vy: usize },

    /// AND Vx, Vy - Set Vx = Vx AND Vy
    AndReg { vx: usize, vy: usize },

    /// XOR Vx, Vy - Set Vx = Vx XOR Vy
    XorReg { vx: usize, vy: usize },

    /// SHR Vx - Set Vx = Vx SHR 1, set VF = least significant bit
    ShrReg { vx: usize },

    /// SHL Vx - Set Vx = Vx SHL 1, set VF = most significant bit
    ShlReg { vx: usize },

    // Display
    /// DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    Draw { vx: usize, vy: usize, n: u8 },

    // Input
    /// SKP Vx - Skip next instruction if key with the value of Vx is pressed
    SkipKeyPressed { vx: usize },

    /// SKNP Vx - Skip next instruction if key with the value of Vx is not pressed
    SkipKeyNotPressed { vx: usize },

    // Random
    /// RND Vx, byte - Set Vx = random byte AND byte
    Random { vx: usize, mask: u8 },

    // Timer and memory operations
    /// LD Vx, DT - Set Vx = delay timer value
    LoadDelayTimer { vx: usize },

    /// LD DT, Vx - Set delay timer = Vx
    SetDelayTimer { vx: usize },

    /// LD ST, Vx - Set sound timer = Vx
    SetSoundTimer { vx: usize },

    /// LD Vx, K - Wait for a key press, store the value of the key in Vx
    WaitKey { vx: usize },

    /// ADD I, Vx - Set I = I + Vx
    AddIndex { vx: usize },

    /// LD F, Vx - Set I = location of sprite for digit Vx
    LoadFont { vx: usize },

    /// LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, and I+2
    StoreBcd { vx: usize },

    /// LD [I], Vx - Store registers V0 through Vx in memory starting at location I
    StoreRegisters { vx: usize },

    /// LD Vx, [I] - Read registers V0 through Vx from memory starting at location I
    LoadRegisters { vx: usize },
}

impl Instruction {
    /// Get a human-readable mnemonic for this instruction
    pub fn mnemonic(&self) -> String {
        match self {
            Instruction::Cls => "CLS".to_string(),
            Instruction::Ret => "RET".to_string(),
            Instruction::Sys { addr } => format!("SYS {:03X}", addr),
            Instruction::Jump { addr } => format!("JP {:03X}", addr),
            Instruction::Call { addr } => format!("CALL {:03X}", addr),
            Instruction::JumpV0 { addr } => format!("JP V0, {:03X}", addr),
            Instruction::SkipEqImm { vx, value } => format!("SE V{:X}, {:02X}", vx, value),
            Instruction::SkipNeImm { vx, value } => format!("SNE V{:X}, {:02X}", vx, value),
            Instruction::SkipEqReg { vx, vy } => format!("SE V{:X}, V{:X}", vx, vy),
            Instruction::SkipNeReg { vx, vy } => format!("SNE V{:X}, V{:X}", vx, vy),
            Instruction::LoadImm { vx, value } => format!("LD V{:X}, {:02X}", vx, value),
            Instruction::LoadReg { vx, vy } => format!("LD V{:X}, V{:X}", vx, vy),
            Instruction::SetIndex { addr } => format!("LD I, {:03X}", addr),
            Instruction::AddImm { vx, value } => format!("ADD V{:X}, {:02X}", vx, value),
            Instruction::AddReg { vx, vy } => format!("ADD V{:X}, V{:X}", vx, vy),
            Instruction::SubReg { vx, vy } => format!("SUB V{:X}, V{:X}", vx, vy),
            Instruction::SubnReg { vx, vy } => format!("SUBN V{:X}, V{:X}", vx, vy),
            Instruction::OrReg { vx, vy } => format!("OR V{:X}, V{:X}", vx, vy),
            Instruction::AndReg { vx, vy } => format!("AND V{:X}, V{:X}", vx, vy),
            Instruction::XorReg { vx, vy } => format!("XOR V{:X}, V{:X}", vx, vy),
            Instruction::ShrReg { vx } => format!("SHR V{:X}", vx),
            Instruction::ShlReg { vx } => format!("SHL V{:X}", vx),
            Instruction::Draw { vx, vy, n } => format!("DRW V{:X}, V{:X}, {:X}", vx, vy, n),
            Instruction::SkipKeyPressed { vx } => format!("SKP V{:X}", vx),
            Instruction::SkipKeyNotPressed { vx } => format!("SKNP V{:X}", vx),
            Instruction::Random { vx, mask } => format!("RND V{:X}, {:02X}", vx, mask),
            Instruction::LoadDelayTimer { vx } => format!("LD V{:X}, DT", vx),
            Instruction::SetDelayTimer { vx } => format!("LD DT, V{:X}", vx),
            Instruction::SetSoundTimer { vx } => format!("LD ST, V{:X}", vx),
            Instruction::WaitKey { vx } => format!("LD V{:X}, K", vx),
            Instruction::AddIndex { vx } => format!("ADD I, V{:X}", vx),
            Instruction::LoadFont { vx } => format!("LD F, V{:X}", vx),
            Instruction::StoreBcd { vx } => format!("LD B, V{:X}", vx),
            Instruction::StoreRegisters { vx } => format!("LD [I], V{:X}", vx),
            Instruction::LoadRegisters { vx } => format!("LD V{:X}, [I]", vx),
        }
    }

    /// Check if this instruction is a conditional skip
    ///
    /// Note: CHIP-8 skip instructions work by advancing PC by an additional 2 bytes,
    /// effectively skipping the next instruction. The base PC advancement (by 2)
    /// happens during fetch as part of the fetch contract.
    pub fn is_skip_instruction(&self) -> bool {
        matches!(
            self,
            Instruction::SkipEqImm { .. }
                | Instruction::SkipNeImm { .. }
                | Instruction::SkipEqReg { .. }
                | Instruction::SkipNeReg { .. }
                | Instruction::SkipKeyPressed { .. }
                | Instruction::SkipKeyNotPressed { .. }
        )
    }
}

/// Decode a 16-bit opcode into an Instruction
///
/// # Architectural Note: Fetch Contract
///
/// The CPU fetch phase has a clear contract:
/// 1. Read 16-bit instruction from memory at PC
/// 2. Advance PC by 2 (to next instruction)
/// 3. Return the instruction for execution
///
/// Instructions that need to modify PC (jumps, calls, returns) simply set PC
/// to their target address. The PC advancement during fetch is NOT conditional
/// on the instruction type - it always happens.
///
/// This design is simpler than trying to track which instructions "modify PC"
/// and makes the fetch/execute cycle more predictable.
pub fn decode_opcode(opcode: u16) -> Result<Instruction, DecodeError> {
    // Extract common operands
    let addr = opcode & 0x0FFF;
    let vx = ((opcode & 0x0F00) >> 8) as usize;
    let vy = ((opcode & 0x00F0) >> 4) as usize;
    let byte = (opcode & 0x00FF) as u8;
    let nibble = (opcode & 0x000F) as u8;

    // Validate register indices
    if vx > 15 {
        return Err(DecodeError::InvalidRegister { register: vx });
    }
    if vy > 15 {
        return Err(DecodeError::InvalidRegister { register: vy });
    }

    match opcode & 0xF000 {
        0x0000 => match opcode {
            0x00E0 => Ok(Instruction::Cls),
            0x00EE => Ok(Instruction::Ret),
            _ => Ok(Instruction::Sys { addr }),
        },
        0x1000 => Ok(Instruction::Jump { addr }),
        0x2000 => Ok(Instruction::Call { addr }),
        0x3000 => Ok(Instruction::SkipEqImm { vx, value: byte }),
        0x4000 => Ok(Instruction::SkipNeImm { vx, value: byte }),
        0x5000 => match nibble {
            0x0 => Ok(Instruction::SkipEqReg { vx, vy }),
            _ => Err(DecodeError::UnknownInstruction { opcode }),
        },
        0x6000 => Ok(Instruction::LoadImm { vx, value: byte }),
        0x7000 => Ok(Instruction::AddImm { vx, value: byte }),
        0x8000 => match nibble {
            0x0 => Ok(Instruction::LoadReg { vx, vy }),
            0x1 => Ok(Instruction::OrReg { vx, vy }),
            0x2 => Ok(Instruction::AndReg { vx, vy }),
            0x3 => Ok(Instruction::XorReg { vx, vy }),
            0x4 => Ok(Instruction::AddReg { vx, vy }),
            0x5 => Ok(Instruction::SubReg { vx, vy }),
            0x6 => Ok(Instruction::ShrReg { vx }),
            0x7 => Ok(Instruction::SubnReg { vx, vy }),
            0xE => Ok(Instruction::ShlReg { vx }),
            _ => Err(DecodeError::UnknownInstruction { opcode }),
        },
        0x9000 => match nibble {
            0x0 => Ok(Instruction::SkipNeReg { vx, vy }),
            _ => Err(DecodeError::UnknownInstruction { opcode }),
        },
        0xA000 => Ok(Instruction::SetIndex { addr }),
        0xB000 => Ok(Instruction::JumpV0 { addr }),
        0xC000 => Ok(Instruction::Random { vx, mask: byte }),
        0xD000 => Ok(Instruction::Draw { vx, vy, n: nibble }),
        0xE000 => match byte {
            0x9E => Ok(Instruction::SkipKeyPressed { vx }),
            0xA1 => Ok(Instruction::SkipKeyNotPressed { vx }),
            _ => Err(DecodeError::UnknownInstruction { opcode }),
        },
        0xF000 => match byte {
            0x07 => Ok(Instruction::LoadDelayTimer { vx }),
            0x0A => Ok(Instruction::WaitKey { vx }),
            0x15 => Ok(Instruction::SetDelayTimer { vx }),
            0x18 => Ok(Instruction::SetSoundTimer { vx }),
            0x1E => Ok(Instruction::AddIndex { vx }),
            0x29 => Ok(Instruction::LoadFont { vx }),
            0x33 => Ok(Instruction::StoreBcd { vx }),
            0x55 => Ok(Instruction::StoreRegisters { vx }),
            0x65 => Ok(Instruction::LoadRegisters { vx }),
            _ => Err(DecodeError::UnknownInstruction { opcode }),
        },
        _ => Err(DecodeError::UnknownInstruction { opcode }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_basic_instructions() {
        // Test a few key instructions
        assert_eq!(decode_opcode(0x00E0).unwrap(), Instruction::Cls);
        assert_eq!(decode_opcode(0x00EE).unwrap(), Instruction::Ret);
        assert_eq!(
            decode_opcode(0x1234).unwrap(),
            Instruction::Jump { addr: 0x234 }
        );
        assert_eq!(
            decode_opcode(0x6342).unwrap(),
            Instruction::LoadImm { vx: 3, value: 0x42 }
        );
        assert_eq!(
            decode_opcode(0x7525).unwrap(),
            Instruction::AddImm { vx: 5, value: 0x25 }
        );
        assert_eq!(
            decode_opcode(0xA123).unwrap(),
            Instruction::SetIndex { addr: 0x123 }
        );
    }

    #[test]
    fn test_decode_arithmetic() {
        assert_eq!(
            decode_opcode(0x8120).unwrap(),
            Instruction::LoadReg { vx: 1, vy: 2 }
        );
        assert_eq!(
            decode_opcode(0x8341).unwrap(),
            Instruction::OrReg { vx: 3, vy: 4 }
        );
        assert_eq!(
            decode_opcode(0x8564).unwrap(),
            Instruction::AddReg { vx: 5, vy: 6 }
        );
    }

    #[test]
    fn test_decode_display() {
        assert_eq!(
            decode_opcode(0xD125).unwrap(),
            Instruction::Draw { vx: 1, vy: 2, n: 5 }
        );
    }

    #[test]
    fn test_decode_unknown_instruction() {
        assert!(matches!(
            decode_opcode(0xFF00),
            Err(DecodeError::UnknownInstruction { opcode: 0xFF00 })
        ));
    }

    #[test]
    fn test_mnemonic_generation() {
        assert_eq!(Instruction::Cls.mnemonic(), "CLS");
        assert_eq!(Instruction::Jump { addr: 0x234 }.mnemonic(), "JP 234");
        assert_eq!(
            Instruction::LoadImm { vx: 3, value: 0x42 }.mnemonic(),
            "LD V3, 42"
        );
        assert_eq!(
            Instruction::Draw { vx: 1, vy: 2, n: 5 }.mnemonic(),
            "DRW V1, V2, 5"
        );
    }

    #[test]
    fn test_skip_instruction_detection() {
        assert!(Instruction::SkipEqImm { vx: 0, value: 42 }.is_skip_instruction());
        assert!(Instruction::SkipNeReg { vx: 1, vy: 2 }.is_skip_instruction());
        assert!(Instruction::SkipKeyPressed { vx: 0 }.is_skip_instruction());
        assert!(!Instruction::Jump { addr: 0x200 }.is_skip_instruction());
        assert!(!Instruction::LoadImm { vx: 0, value: 42 }.is_skip_instruction());
    }
}
