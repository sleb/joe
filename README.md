# OCTO - CHIP-8 Emulator

A CHIP-8 emulator written in Rust as a learning project to explore low-level programming, emulation concepts, and Rust systems programming.

## Project Overview

CHIP-8 is an interpreted programming language developed in the 1970s for simple video games. It was designed to run on the COSMAC VIP and Telmac 1800 microcomputers, but became popular for creating simple games due to its ease of implementation.

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

#### 3. Display (`src/display.rs`) 🚧

- 64x32 pixel framebuffer (TODO)
- Sprite drawing with XOR logic (TODO)
- Screen clearing functionality (TODO)

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

#### 7. Frontend (`src/frontend/`) 🚧

- Graphics rendering (TODO - likely using `minifb` or `pixels`)
- Input handling (TODO)
- Audio output (TODO)
- File loading interface (TODO)

## Development Roadmap

### Phase 1: Core Foundation ✅ (v0.1.1)

- [x] Basic project structure and CLI
- [x] Git-based semantic versioning system
- [x] Memory system with 4KB RAM and font data
- [x] ROM loading with validation and write protection
- [x] Comprehensive testing framework and lean testing philosophy

### Phase 2: Core Emulator 🚧 (v0.2.0)

- [x] CPU structure and register management (V0-VF, I, PC, SP)
- [x] MemoryBus trait abstraction for clean CPU-memory interaction
- [x] Instruction fetch/decode/execute framework
- [x] Core instructions: load, add, jump, call/return, set index
- [x] Stack management and program counter logic
- [ ] Remaining CHIP-8 instructions (arithmetic, bitwise, etc.)
- [ ] ASCII display for basic sprite rendering
- [ ] Basic emulation loop to run simple ROMs

### Phase 3: Display System (v0.3.0)

- [ ] 64x32 framebuffer implementation
- [ ] Sprite drawing with XOR logic
- [ ] Display clear functionality
- [ ] Collision detection for sprite drawing
- [ ] Choose and integrate graphics library

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
- 🚧 **v0.2.0** - CPU foundation (registers, core instructions, execution framework) **[IN PROGRESS]**
- 🎯 **v0.3.0** - Complete CPU (all 35 instructions, display integration)
- 🎯 **v0.4.0** - Display system (64x32 framebuffer, sprite drawing)
- 🎯 **v0.5.0** - Input and timing (16-key keypad, 60Hz timing)
- 🎯 **v0.6.0** - Audio and ROM loading (sound timer, file loading)
- 🎯 **v0.7.0** - Testing and compatibility (ROM compatibility, debugging)
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
3. **Publishing**: `git push origin main --tags`

The build script automatically detects version mismatches and provides helpful warnings during development.

## Building and Running

```bash
# Quick development commands
just build              # Build the project
just test               # Run tests
just check              # Run all checks (fmt, lint, test)
just version            # Show version info

# Traditional cargo commands
cargo build
cargo run -- version
cargo test
cargo build --release

# Run with a ROM file (when implemented)
just run path/to/rom.ch8
cargo run -- run path/to/rom.ch8
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
