# OCTO - CHIP-8 Emulator

A CHIP-8 emulator written in Rust as a learning project to explore low-level programming, emulation concepts, and Rust systems programming.

## Project Overview

CHIP-8 is an interpreted programming language developed in the 1970s for simple video games. It was designed to run on the COSMAC VIP and Telmac 1800 microcomputers, but became popular for creating simple games due to its ease of implementation.

### Current Status

🎉 **The emulator successfully runs CHIP-8 ROMs!**

We can now execute the IBM Logo ROM, demonstrating:

- ✅ Complete instruction decoding and execution
- ✅ Sprite drawing with XOR logic and collision detection
- ✅ Memory management with ROM loading
- ✅ ASCII terminal rendering for development

Try it yourself: `octo run roms/ibm-logo.ch8`

## Installation

### From Latest Release (Recommended)

```bash
# Install the latest release from GitHub
cargo install --git https://github.com/sleb/octo --tag v0.1.3

# Verify installation
octo version
```

### Updating

```bash
# Update to the latest release
cargo install --git https://github.com/sleb/octo --tag v0.1.3 --force

# Or uninstall and reinstall
cargo uninstall octo
cargo install --git https://github.com/sleb/octo --tag v0.1.3
```

### From Specific Version

```bash
# Install a specific version (replace v0.1.2 with desired version)
cargo install --git https://github.com/sleb/octo --tag v0.1.2
```

### From Source (Development)

```bash
# Clone the repository
git clone https://github.com/sleb/octo.git
cd octo

# Install the binary from source
cargo install --path .

# Verify installation
octo version
```

> **Note:** Check the [releases page](https://github.com/sleb/octo/releases) for the latest version number and replace `v0.1.3` in the commands above with the newest release tag.

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Git (for installation from GitHub)

### Goals

- Build a fully functional CHIP-8 emulator from scratch
- Learn Rust systems programming concepts
- Understand emulation and virtual machine architecture
- Create a clean, well-documented codebase
- Support loading and running classic CHIP-8 ROMs

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
octo run roms/ibm-logo.ch8
```

You should see the classic IBM logo rendered in ASCII art in your terminal!

## Examples

### Running ROMs

```bash
# Run the IBM Logo ROM with default settings
octo run roms/ibm-logo.ch8

# Run with verbose output showing CPU state each cycle
octo run roms/ibm-logo.ch8 --verbose

# Run faster (no delay between cycles)
octo run roms/ibm-logo.ch8 --cycle-delay-ms 0

# Run in headless mode (no display output, useful for testing)
octo run roms/ibm-logo.ch8 --headless

# Set maximum cycles to prevent runaway programs
octo run roms/ibm-logo.ch8 --max-cycles 500
```

### Analyzing ROMs

```bash
# Show disassembly and instruction analysis
octo analyze roms/ibm-logo.ch8 --disassemble --stats

# Quick analysis (shows what instructions are needed)
octo analyze roms/ibm-logo.ch8
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

#### 4. Input (`src/input.rs`) 🚧

- 16-key keypad mapping (TODO)
- Key state management (TODO)
- Keyboard event handling (TODO)

#### 5. Audio (`src/audio.rs`) 🚧

- Sound timer management (TODO)
- Beep generation (TODO)

#### 6. Emulator (`src/emulator.rs`) 🚧

- Main emulation loop (TODO)
- Component coordination (TODO)
- Timing management (TODO)

#### 7. Rendering (`src/display.rs`) ✅

- ASCII terminal renderer for development
- Headless renderer for testing
- Extensible renderer trait for future GUI implementations
- Clean separation between display logic and presentation

## Development Roadmap

### Phase 1: Core Foundation ✅ (v0.1.1)

- [x] Basic project structure and CLI
- [x] Git-based semantic versioning system
- [x] Memory system with 4KB RAM and font data
- [x] ROM loading with validation and write protection
- [x] Comprehensive testing framework and lean testing philosophy

### Phase 2: Core Emulator ✅ (v0.2.0)

- [x] CPU structure and register management (V0-VF, I, PC, SP)
- [x] MemoryBus trait abstraction for clean CPU-memory interaction
- [x] Instruction fetch/decode/execute framework
- [x] Centralized instruction decoding (single source of truth)
- [x] Core instructions: load, add, jump, call/return, set index, draw, clear
- [x] Stack management and program counter logic
- [x] Display system with XOR sprite drawing and collision detection
- [x] ASCII renderer for development and testing
- [x] Successfully runs IBM logo ROM

### Phase 3: Extended Instruction Set (v0.3.0)

- [ ] Remaining CHIP-8 instructions (arithmetic, bitwise, conditional skips)
- [ ] Timer operations (delay timer, sound timer)
- [ ] Input handling (key press detection)
- [ ] Binary-coded decimal (BCD) operations
- [ ] Register dump/load operations

### Phase 4: Input and Timing (v0.4.0)

- [ ] 16-key keypad input handling
- [ ] Map keyboard to CHIP-8 keypad
- [ ] Delay timer implementation
- [ ] Sound timer implementation
- [ ] 60Hz timing accuracy

### Phase 5: Audio and ROM Loading (v0.5.0)

- [ ] Basic audio output/beep generation
- [ ] ROM file loading (.ch8 files)
- [ ] Error handling for invalid ROMs
- [ ] File loading interface

### Phase 6: Testing and Compatibility (v0.6.0)

- [ ] Test with classic CHIP-8 games
- [ ] ROM compatibility validation
- [ ] Debugging tools and logging
- [ ] Performance optimization

### Phase 7: Production Ready (v1.0.0)

- [ ] Enhanced debugging features
- [ ] Configuration options
- [ ] Comprehensive documentation
- [ ] Installation and distribution
- [ ] Stable API and features

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

## Version Management

We use a **Rust build script** (`build.rs`) for automatic version management and validation:

### Automatic Features

- **Build-time validation**: Warns if `Cargo.toml` version doesn't match git tags
- **Rich version info**: Git hash, branch, build time, dirty status automatically included
- **Cross-platform**: Pure Rust, no shell script dependencies
- **Cargo integration**: Rebuilds when git state changes

### Versioning Strategy

**Pre-1.0 Development Versions:**

- ✅ **v0.1.1** - Foundation complete (CLI, memory system, testing framework)
- ✅ **v0.2.0** - Core emulator complete (CPU, display, basic instructions, runs IBM logo)
- 🎯 **v0.3.0** - Extended instruction set (all 35 instructions, timers, input)
- 🎯 **v0.4.0** - GUI rendering (SDL/pixels integration, better user experience)
- 🎯 **v0.5.0** - Audio and enhanced features (sound timer, debugging tools)
- 🎯 **v0.6.0** - ROM compatibility and testing (test suite, multiple ROMs)
- 🎯 **v0.7.0** - Performance and polish (optimization, configuration)
- 🎯 **v1.0.0** - Production ready (stable API, documentation, distribution)

**Patch Versions (v0.x.y):**

- Infrastructure improvements, bug fixes, documentation
- Regular incremental updates between major features

### Development Workflow

**Daily Development:**

```bash
# Check current version (shows development state)
just version-detailed
# e.g., "0.1.0-dev.4" = 4 commits since v0.1.0 tag

# Work on features, commit regularly
git add -A && git commit -m "feat: implement CPU registers"

# Run tests before committing
just check
```

**Versioning Guidelines:**

```bash
# Patch releases (v0.1.x): Infrastructure, docs, bug fixes
just release patch    # 0.1.0 -> 0.1.1

# Minor releases (v0.x.0): New major features/components
just release minor    # 0.1.1 -> 0.2.0

# Major release (v1.0.0): Production ready
just release major    # 0.6.0 -> 1.0.0
```

**When to Version:**

- **Patch (0.1.x)**: After completing infrastructure improvements
- **Minor (0.x.0)**: After completing major emulator components (CPU, Display, etc.)
- **Major (1.0.0)**: When emulator is production-ready with stable API

### Release Process

1. **Development**: Work normally, build script warns of any version issues
2. **Release**: Use `just release TYPE` for automated version bump and tagging
3. **README Updates**: The release process automatically updates installation instructions with the new version
4. **Publishing**: `git push origin main --tags`

The build script automatically detects version mismatches and provides helpful warnings during development.

## Using the Emulator

Once installed, use these commands:

```bash
# Run CHIP-8 ROMs
octo run roms/ibm-logo.ch8
octo run rom-file.ch8 --verbose
octo run rom-file.ch8 --headless --cycle-delay-ms 0

# Analyze ROMs
octo analyze roms/ibm-logo.ch8
octo analyze rom-file.ch8 --disassemble

# System information
octo version
octo --help
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

# README version management (for releases)
just update-readme-version 0.1.4  # Update installation instructions
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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Status**: 🚧 Under Development
