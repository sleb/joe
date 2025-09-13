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

#### 1. CPU (`src/cpu.rs`)

- Instruction decoding and execution
- Register management
- Program counter and stack operations
- Timer management

#### 2. Memory (`src/memory.rs`)

- 4KB RAM management
- Font data storage
- ROM loading utilities

#### 3. Display (`src/display.rs`)

- 64x32 pixel framebuffer
- Sprite drawing with XOR logic
- Screen clearing functionality

#### 4. Input (`src/input.rs`)

- 16-key keypad mapping
- Key state management
- Keyboard event handling

#### 5. Audio (`src/audio.rs`)

- Sound timer management
- Beep generation

#### 6. Emulator (`src/emulator.rs`)

- Main emulation loop
- Component coordination
- Timing management

#### 7. Frontend (`src/frontend/`)

- Graphics rendering (likely using `minifb` or `pixels`)
- Input handling
- Audio output
- File loading interface

## Development Roadmap

### Phase 1: Core Foundation ✅ (v0.1.0)

- [x] Basic project structure
- [x] CLI argument parsing setup
- [x] Git-based semantic versioning system
- [ ] Memory system implementation
- [ ] CPU structure and basic instruction decoding

### Phase 2: Core Emulator (v0.2.0)

- [ ] Memory system implementation (4KB RAM, font data)
- [ ] CPU structure and register management
- [ ] Implement all 35 CHIP-8 instructions
- [ ] Stack management and program counter logic
- [ ] Basic test suite for CPU operations

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
- **Testing**: Unit tests for each component, integration tests for ROM compatibility

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

**v0.x.0 = Pre-release development**

- v0.1.0 = Project foundation (current)
- v0.2.0 = Core emulator working
- v0.3.0 = Display system
- v0.4.0 = Input and timing
- v0.5.0 = Audio and ROM loading
- v0.6.0 = Testing and compatibility
- v1.0.0 = Production ready

### Development Workflow

```bash
# Version info (shows current git-based version)
just version-detailed

# Development versions show commits since tag
# e.g., "0.1.0-dev.5" = 5 commits since v0.1.0

# Create a new release
just release minor    # 0.1.0 -> 0.2.0
just release patch    # 0.2.0 -> 0.2.1
just release major    # 0.6.0 -> 1.0.0
```

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

## Contributing

This is primarily a learning project, but suggestions and improvements are welcome!

### Development Tools Used

- **just**: Task runner for common commands (`brew install just`)
- **cargo-release**: Automated release management (optional)
- **Standard Rust toolchain**: rustc, cargo, clippy, rustfmt

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Status**: 🚧 Under Development
