# JOE - CHIP-8 Emulator

A CHIP-8 emulator written in Rust as a learning project to explore low-level programming, emulation concepts, and Rust systems programming.

> **📢 Important**: This project was recently renamed from `octo` to `joe` to avoid conflicts with the existing [Octo IDE](https://github.com/JohnEarnest/Octo). See [Migration Guide](#migration-from-octo) below.

## Project Overview

CHIP-8 is an interpreted programming language developed in the 1970s for simple video games. It was designed to run on the COSMAC VIP and Telmac 1800 microcomputers, but became popular for creating simple games due to its ease of implementation.

### Current Status

🎉 **The emulator successfully runs CHIP-8 ROMs!**

We can now execute the IBM Logo ROM, demonstrating:

- ✅ Complete instruction decoding and execution
- ✅ Sprite drawing with XOR logic and collision detection
- ✅ Memory management with ROM loading
- ✅ ASCII terminal rendering for development

Try it yourself: `joe run roms/ibm-logo.ch8`

## Installation

### From Latest Release (Recommended)

```bash
# Install the latest release from GitHub
cargo install --git https://github.com/sleb/joe --tag v0.2.0

# Verify installation
joe version
```

### Updating

```bash
# Update to the latest release
cargo install --git https://github.com/sleb/joe --tag v0.2.0 --force

# Or uninstall and reinstall
cargo uninstall joe
cargo install --git https://github.com/sleb/joe --tag v0.2.0
```

### From Specific Version

```bash
# Install a specific version (replace v0.1.2 with desired version)
cargo install --git https://github.com/sleb/joe --tag v0.1.2
```

### From Source (Development)

```bash
# Clone the repository
git clone https://github.com/sleb/joe.git
cd joe

# Install the binary from source
cargo install --path .

# Verify installation
joe version
```

> **Note:** Check the [releases page](https://github.com/sleb/joe/releases) for the latest version number and replace version tags in the commands above with the newest release.

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Git (for installation from GitHub)

> **✅ Robust Installation:** The build system automatically handles both git-enabled and git-free environments. When installing via `cargo install --git`, the emulator gracefully falls back to standard versioning if git metadata isn't available, ensuring installation always succeeds.

## Migration from `octo`

If you previously installed this emulator as `octo`, you'll need to migrate to the new `joe` name:

```bash
# 1. Uninstall the old version
cargo uninstall octo

# 2. Install the new version
cargo install --git https://github.com/sleb/joe --tag v0.2.0

# 3. Update any scripts or aliases
# Old: octo run rom.ch8
# New: joe run rom.ch8
```

**Why the rename?** We discovered an existing, well-established CHIP-8 project called [Octo](https://github.com/JohnEarnest/Octo) by John Earnest. To avoid confusion, we renamed our project to **JOE**.

### Goals

**✅ Achieved:**

- Built a fully functional CHIP-8 emulator from scratch
- Learned Rust systems programming concepts (traits, ownership, testing)
- Understood emulation and virtual machine architecture
- Created a clean, well-documented codebase with professional CLI
- Support loading and running classic CHIP-8 ROMs with real-time display

**🎯 Next Objectives:**

- Add input system for interactive games
- Implement audio output for sound effects
- Create GUI interface for better user experience
- Achieve compatibility with more classic CHIP-8 ROMs

## CHIP-8 System Specifications

### Hardware Overview

- **CPU**: Custom 8-bit processor
- **Memory**: 4KB RAM (4096 bytes)
- **Display**: 64x32 pixel monochrome screen
- **Input**: 16-key hexadecimal keypad
- **Sound**: Single tone beeper
- **Timers**: 60Hz delay timer and sound timer

### Memory Layout

```
0x000-0x1FF: CHIP-8 interpreter (contains font set in emu)
0x050-0x0A0: Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF: Program ROM and work RAM
```

### Registers

- **V0-VF**: 16 general-purpose 8-bit registers
- **I**: 16-bit index register
- **PC**: Program counter
- **SP**: Stack pointer
- **Stack**: 16 levels of 16-bit values

### Instruction Set

35 opcodes, all 2 bytes long and stored big-endian

- System calls
- Jump/call instructions
- Conditional skips
- Register operations
- Memory operations
- Display operations
- Input operations
- Timer operations

## Quick Start

To see the emulator in action with the IBM Logo ROM:

```bash
# After installation (see Installation section above)
joe run roms/ibm-logo.ch8
```

You'll see the classic IBM logo being drawn in real-time with ASCII art in your terminal! The emulator shows continuous display updates as the ROM executes, just like a real CHIP-8 system. Use `Ctrl+C` to stop execution and see statistics.

## Examples

### Running ROMs

```bash
# Run the IBM Logo ROM with default settings (continuous display updates)
joe run roms/ibm-logo.ch8

# Run with slower updates to see the logo being drawn step by step
# Press Ctrl+C anytime to stop and see statistics
joe run roms/ibm-logo.ch8 --cycle-delay-ms 500

# Run with verbose output showing CPU state each cycle
joe run roms/ibm-logo.ch8 --verbose

# Show only the final display state (no continuous updates)
joe run roms/ibm-logo.ch8 --final-only

# Run in headless mode (no display output, useful for testing)
joe run roms/ibm-logo.ch8 --headless

# Set maximum cycles (0 = unlimited, programs can run indefinitely)
joe run roms/ibm-logo.ch8 --max-cycles 100

# Fast execution with final display only
joe run roms/ibm-logo.ch8 --cycle-delay-ms 0 --final-only
```

### Analyzing ROMs

```bash
# Show disassembly and instruction analysis
joe analyze roms/ibm-logo.ch8 --disassemble --stats

# Quick analysis (shows what instructions are needed)
joe analyze roms/ibm-logo.ch8
```

## Project Architecture

### Core Components

#### 1. CPU (`src/cpu.rs`) ✅

- Instruction decoding and execution framework
- Register management (V0-VF, I, PC, SP)
- Program counter and stack operations
- Timer management (delay and sound timers)
- Core instructions: load, add, jump, call/return, set index

#### 2. Memory (`src/memory.rs`) ✅

- 4KB RAM management with bounds checking
- Built-in font data (0x050-0x0A0)
- ROM loading with validation
- Write protection for interpreter area
- MemoryBus trait abstraction for CPU integration

#### 3. Display (`src/display.rs`) ✅

- 64x32 pixel framebuffer with XOR sprite drawing
- Collision detection for sprite operations
- Coordinate wrapping at screen edges
- Separation of logical display from rendering concerns

#### 4. CLI and Emulation (`src/cli/run.rs`) ✅

- Complete emulation loop with cycle execution
- Real-time display updates and smart rendering
- Comprehensive ROM execution with statistics
- Signal handling (Ctrl+C) with graceful shutdown
- Multiple execution modes (continuous, final-only, headless)
- Cycle timing and delay management

#### 5. Instruction Set (`src/instruction.rs`) ✅

- Complete CHIP-8 instruction set (all 35 opcodes)
- Centralized instruction decoding with single source of truth
- Full arithmetic, logical, display, input, and timer operations
- Comprehensive instruction analysis and disassembly
- Mnemonic generation for debugging

#### 6. Rendering (`src/display.rs`) ✅

- ASCII terminal renderer for development
- Headless renderer for testing
- Extensible renderer trait for future GUI implementations
- Clean separation between display logic and presentation

## Current Status & Next Steps

### ✅ **Completed: Core CHIP-8 Emulator (v0.2.0+)**

**What Works Now:**

- **Complete instruction set**: All 35 CHIP-8 opcodes implemented and tested
- **Full emulation**: ROM loading, execution, display, and statistics
- **Professional CLI**: `joe run` and `joe analyze` commands with comprehensive options
- **Real-time display**: Continuous ASCII rendering with smart update logic
- **Memory system**: 4KB RAM, font data, ROM loading with validation
- **Display system**: 64x32 framebuffer with XOR sprite drawing and collision detection
- **CPU architecture**: Complete register management, stack, timers, program counter
- **Signal handling**: Graceful Ctrl+C with statistics display
- **Multiple renderers**: ASCII terminal and headless modes
- **Testing**: Comprehensive unit and integration test coverage

**Runs Real ROMs:**

- IBM Logo ROM executes perfectly with real-time display updates
- Instruction analysis and disassembly tools
- Statistics tracking and performance monitoring

### 🚧 **Next Phase: Enhanced Features**

**Input System (High Priority):**

- [ ] 16-key keypad input handling
- [ ] Keyboard mapping to CHIP-8 keypad
- [ ] Key press/release detection for input instructions

**Audio System:**

- [ ] Sound timer implementation with actual audio output
- [ ] Beep generation (simple tone or platform audio)

**Enhanced User Experience:**

- [ ] GUI renderer (SDL, pixels, or similar)
- [ ] Configuration file support
- [ ] Save/load state functionality
- [ ] Debugging tools and step-through execution

**ROM Compatibility:**

- [ ] Test with classic CHIP-8 games (Tetris, Pong, etc.)
- [ ] Compatibility validation and fixes
- [ ] ROM collection and examples

### 🎯 **Vision: v1.0.0 Production Release**

- [ ] Complete input and audio systems
- [ ] GUI interface alongside CLI
- [ ] Extensive ROM compatibility
- [ ] Performance optimization
- [ ] Comprehensive documentation
- [ ] Package distribution (crates.io, GitHub releases)

## Technical Decisions

### Dependencies

- **anyhow**: Error handling and context
- **clap**: CLI argument parsing
- **TBD**: Graphics library (considering `minifb`, `pixels`, or `macroquad`)
- **TBD**: Audio library (considering `rodio` or `cpal`)

### Architecture Choices

- **Modular design**: Separate modules for each major component
- **Trait-based interfaces**: Allow for different frontend implementations
- **Error handling**: Use `Result` types and proper error propagation
- **Lean testing**: Focused unit tests, minimal integration tests for real workflows

### Architectural Contracts

These are key design decisions and contracts that developers need to understand when working with the codebase:

#### CPU Fetch Contract

**Rule**: Fetch ALWAYS advances PC by 2 bytes, unconditionally.

```rust
// During fetch phase:
let instruction = memory.read_word(pc);
pc += 2;  // ALWAYS happens, regardless of instruction type

// During execute phase:
match instruction {
    Jump { addr } => pc = addr,        // Simply override PC
    Call { addr } => { push(pc); pc = addr; }  // Use current PC, then override
    SkipEq { .. } if condition => pc += 2,     // Additional skip on top of fetch
    _ => { /* PC already advanced during fetch */ }
}
```

**Rationale**: This keeps fetch/execute separation clean and predictable. Instructions that need to modify PC (jumps, calls, returns) simply set PC to their target address. Skip instructions add an additional +2 on top of the fetch advancement.

**Alternative Considered**: Conditional PC advancement based on instruction type was rejected as more complex and error-prone.

#### Instruction Decoding Contract

**Rule**: Single source of truth for opcode meanings via centralized `decode_opcode()` function.

```rust
// ✅ Correct: Use centralized decoding
let instruction = decode_opcode(opcode)?;
match instruction {
    Instruction::LoadImm { vx, value } => self.v[vx] = value,
    // ... handle all instruction types
}

// ❌ Incorrect: Independent opcode matching
match opcode & 0xF000 {
    0x6000 => { /* duplicate decoding logic */ }
}
```

**Rationale**: CPU execution and disassembly both need to understand opcodes, but should never disagree on their meaning. The `Instruction` enum serves as the canonical definition of what each opcode means.

#### Memory Bus Contract

**Rule**: CPU accesses memory only through the `MemoryBus` trait, never directly.

```rust
// ✅ Correct: Use trait abstraction
fn execute_cycle<M: MemoryBus>(&mut self, memory: &mut M) -> Result<()>

// ❌ Incorrect: Direct memory dependency
fn execute_cycle(&mut self, memory: &mut Memory) -> Result<()>
```

**Rationale**: This enables testing with mock memory, different memory implementations, and keeps the CPU decoupled from specific memory types.

#### Display/Renderer Separation Contract

**Rule**: Logical display operations are separate from physical rendering.

```rust
// ✅ Correct: Separated concerns
pub trait DisplayBus {
    fn clear(&mut self);
    fn draw_sprite(&mut self, x: u8, y: u8, data: &[u8]) -> Result<bool>;
    // No render() method - that's not the display's job
}

pub trait Renderer {
    fn render(&self, display: &dyn DisplayBus);
}

// ✅ Usage: Display logic + chosen renderer
let mut display = Display::new();
display.draw_sprite(10, 5, &sprite_data)?;
AsciiRenderer.render(&display);  // or GuiRenderer, HeadlessRenderer, etc.

// ❌ Incorrect: Mixed responsibilities
impl Display {
    fn draw_sprite_and_render_ascii(&mut self) { /* mixed concerns */ }
}
```

**Rationale**: Display logic (XOR, collision detection, coordinate wrapping) is the same regardless of output method. Separating rendering allows multiple presentation methods (ASCII, GUI, headless testing) without duplicating display logic.

#### Error Handling Contract

**Rule**: Errors include rich context and location information when possible.

```rust
// ✅ Correct: Rich error context
CpuError::InstructionExecutionFailed {
    instruction: 0xF123,
    addr: 0x0200,
    source: DecodeError::UnknownInstruction { opcode: 0xF123 }
}

// ❌ Incorrect: Generic or context-free errors
CpuError::ExecutionFailed
```

**Rationale**: Debugging emulation issues requires knowing exactly where and why something failed. Generic errors make debugging much harder.

### Performance Considerations

- Target 60Hz execution with accurate timing
- Efficient sprite drawing and collision detection
- Minimal allocations in the main emulation loop

### Version Management

- **Automated versioning**: Rust build script provides git-based version info
- **Release commands**: Use `just release patch|minor|major` for version bumps
- **Consistency validation**: `just validate-versions` checks alignment across files
- **Fallback support**: Build gracefully handles git-free installation environments

## Controls (Planned)

CHIP-8 uses a 16-key keypad. Our keyboard mapping:

```
CHIP-8 Key    Keyboard
1 2 3 C       1 2 3 4
4 5 6 D  =>   Q W E R
7 8 9 E       A S D F
A 0 B F       Z X C V
```

## Resources and References

### CHIP-8 Documentation

- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [CHIP-8 Wikipedia](https://en.wikipedia.org/wiki/CHIP-8)
- [Mastering CHIP-8](https://github.com/mattmikolay/chip-8/wiki)

### Test ROMs

- [CHIP-8 Test Suite](https://github.com/Timendus/chip8-test-suite)
- [Classic CHIP-8 Games](https://www.zophar.net/pdroms/chip8.html)

## Roadmap

### ✅ **Core Emulator Complete (v0.2.0)**

- Full CHIP-8 instruction set (35 opcodes)
- 4KB memory system with ROM loading
- 64x32 display with ASCII rendering
- Professional CLI with run/analyze commands
- Real-time execution with statistics
- Robust installation and version management

### 🎯 **Next Major Features**

- **Input System** - 16-key keypad support for interactive games
- **Audio System** - Sound timer implementation and beep generation
- **GUI Interface** - Visual rendering with SDL/pixels integration
- **Enhanced Tools** - Debugging features, save states, configuration
- **ROM Compatibility** - Extensive testing with popular CHIP-8 games
- **Production Release** - Stable API and package distribution

## Version History

- **v0.2.0** (2025-09-14) - Project renamed to `joe`, robust installation system
- **v0.1.5** (2025-09-13) - Real-time display updates, signal handling, enhanced UX
- **v0.1.3** (2025-09-13) - Display system complete, IBM Logo ROM working
- **v0.1.2** (2025-09-13) - CPU instruction execution, core opcodes implemented
- **v0.1.1** (2025-09-13) - Memory system, ROM loading, write protection
- **v0.1.0** (2025-09-13) - Initial project structure, CLI foundation, git versioning

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

## Using the Emulator

Once installed, use these commands:

```bash
# Run CHIP-8 ROMs
joe run roms/ibm-logo.ch8
joe run rom-file.ch8 --verbose
joe run rom-file.ch8 --headless --cycle-delay-ms 0

# Analyze ROMs
joe analyze roms/ibm-logo.ch8
joe analyze rom-file.ch8 --disassemble

# System information
joe version
joe --help
```

## Development

For developers working on the emulator:

```bash
# Development commands
just build              # Build the project
just test               # Run tests
just check              # Run all checks (fmt, lint, test)
just version            # Show version info

# Traditional cargo commands
cargo build
cargo test
cargo build --release

# Run during development (without installing)
cargo run -- run roms/ibm-logo.ch8
cargo run -- analyze roms/ibm-logo.ch8 --disassemble
```

## Development Setup

```bash
# Install development tools
just dev-setup

# Check project status
just status

# Common development cycle
just check              # Before committing
just build && just test # Verify everything works
```

## Testing Philosophy

We follow a **lean testing approach** that avoids redundancy and over-testing:

### Testing Tenets

1. **Purpose-driven**: Every test should have a clear, unique purpose
2. **Avoid redundancy**: Don't test the same thing in multiple places
3. **Focus on behavior**: Test what the code does, not how it's implemented
4. **Real scenarios**: Integration tests should reflect actual usage patterns
5. **No hardcoded validation**: Don't test constants with more constants

### Test Structure

- **Unit Tests** (`src/module.rs`): Fast, focused tests of individual functions
- **Integration Tests** (`tests/`): Real workflows spanning multiple components
- **No Library Tests**: Avoid intermediate testing layers that duplicate coverage

### Examples

```rust
// ✅ Good: Tests real behavior
#[test]
fn test_rom_loading_workflow() { /* ... */ }

// ❌ Avoid: Testing constants with constants
#[test]
fn test_memory_size_is_4096() {
    assert_eq!(MEMORY_SIZE, 4096);  // Pointless
}

// ❌ Avoid: Redundant API testing
#[test]
fn test_memory_api_in_lib() { /* already tested in memory.rs */ }
```

## Contributing

This is primarily a learning project, but suggestions and improvements are welcome!

### Development Tools Used

- **just**: Task runner for common commands (`brew install just`)
- **cargo-release**: Automated release management (optional)
- **Standard Rust toolchain**: rustc, cargo, clippy, rustfmt

## Release History

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes and version history.

## Why `joe`?

The name **JOE** is a tribute to **Joseph Weisbecker**, who created the CHIP-8 interpreted programming language at RCA in 1977 for the COSMAC VIP computer. We chose this name after discovering that our original name `octo` conflicted with the existing [Octo CHIP-8 IDE](https://github.com/JohnEarnest/Octo) by John Earnest. Rather than pick an arbitrary name, we decided to honor the original creator of the system we're emulating.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Status**: 🚧 Under Development
