# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **BREAKING**: Project renamed from `octo` to `joe` in tribute to Joseph Weisbecker (CHIP-8's creator)
- Updated all command references: `octo run` → `joe run`, `octo analyze` → `joe analyze`, etc.
- Repository moved from `github.com/sleb/octo` to `github.com/sleb/joe`
- Package name changed from `octo` to `joe` in Cargo.toml

### Fixed

- Build script now gracefully falls back to Cargo.toml version when git versioning isn't available
- Resolves installation failures when using `cargo install --git` in environments with limited git metadata
- Maintains rich git version information for development builds while ensuring robust installation process

## [0.1.5] - 2025-09-13

### Added

- Real-time continuous display updates during ROM execution
- Smart display rendering that only updates when display changes or at reasonable intervals
- `--final-only` flag to show only the final display state instead of continuous updates
- Improved cycle delay defaults (16ms ≈ 60fps) for smooth real-time display
- Support for unlimited execution (removed artificial infinite loop detection)
- Graceful Ctrl+C handling that shows statistics and final display state before exiting
- Eliminated magic numbers in ASCII renderer with configurable pixel width system
- Pixel character configuration methods in Renderer trait (pixel_width, pixel_on_char, pixel_off_char)
- Comprehensive version management validation commands (validate-versions, sync-versions, fix-versions)
- Automated version consistency checking in release process

### Changed

- ROM execution now runs indefinitely by default (max-cycles=0) instead of stopping at 1000 cycles
- Infinite loops are now treated as expected behavior, not errors
- Display updates happen in real-time as the ROM executes, not just at completion
- ASCII renderer uses double-wide characters (██) for better pixel representation
- Default cycle delay reduced from 100ms to 16ms for smoother animation
- Updated project roadmap and architecture documentation to reflect current implementation status
- Removed redundant final display output in continuous mode (display already visible above)
- Versioning strategy updated to reflect actual development progression
- Release process enhanced with pre/post-release version validation
- Version management commands integrated into development workflow

### Removed

- Artificial infinite loop detection that incorrectly treated normal CHIP-8 behavior as bugs
- Static display rendering (display now updates continuously during execution)
- Magic number dependencies in ASCII renderer border calculations

### Technical

- Added ctrlc dependency for cross-platform signal handling
- Improved Renderer trait with configurable pixel representation
- Enhanced statistics display logic with reusable helper function
- Updated documentation to accurately reflect complete CHIP-8 instruction set implementation
- Improved version management SOP with validation commands to prevent inconsistencies
- Fixed version synchronization issues between Cargo.toml, README, and git tags

## [0.1.3] - 2025-09-13

### Added

- Complete display system with 64x32 framebuffer and XOR sprite drawing
- Collision detection for sprite operations with proper CHIP-8 behavior
- Coordinate wrapping at screen edges following CHIP-8 specification
- ASCII terminal renderer for development and testing
- Headless renderer for testing without output
- Working CLS (clear screen) and DRW (draw sprite) instruction implementation
- Successfully runs IBM Logo ROM demonstrating complete core functionality

### Changed

- Separated logical display operations from physical rendering concerns
- Refactored display architecture with DisplayBus and Renderer traits
- Updated CPU to integrate with display system for CLS and DRW instructions
- Enhanced error handling for display operations with descriptive messages

### Technical

- Added DisplayBus trait for logical display operations (clear, draw_sprite, get/set pixel)
- Added Renderer trait for presentation layer (ASCII, headless, future GUI)
- Updated CPU execute_cycle to accept both memory and display parameters
- Added comprehensive display tests covering XOR logic, collision detection, and coordinate wrapping
- Created working example program that runs IBM Logo ROM with ASCII output

### Architectural

- Established display/renderer separation contract in documentation
- Added trait-based architecture enabling multiple rendering backends
- Maintained clean separation between "what to display" and "how to display it"
- Future-proofed design for GUI renderers without changing display logic

## [0.1.2] - 2025-09-13

### Added

- Complete CPU module with instruction execution framework
- MemoryBus trait abstraction for clean CPU-memory interaction
- CPU state management: 16 registers (V0-VF), index register (I), program counter (PC), stack pointer (SP)
- Call stack implementation with 16 levels for subroutine management
- Timer system: delay timer and sound timer with 60Hz update capability
- Core CHIP-8 instructions implemented:
  - `6xkk` - Load immediate value into register (LD Vx, byte)
  - `7xkk` - Add immediate value to register (ADD Vx, byte)
  - `1nnn` - Jump to address (JP addr)
  - `2nnn` - Call subroutine (CALL addr)
  - `00EE` - Return from subroutine (RET)
  - `Annn` - Set index register (LD I, addr)
  - `00E0` - Clear display (CLS) - stubbed for future display module
  - `Dxyn` - Draw sprite (DRW Vx, Vy, nibble) - stubbed for future display module

### Changed

- Improved error handling with instruction location tracking
- CPU errors now provide rich context: instruction, memory location, and failure reason
- Error messages are clean and non-repetitive

### Technical

- CPU uses trait-based memory access for better testability and modularity
- Instruction fetch advances program counter as part of fetch contract
- Jump and call instructions properly manage program counter state
- Comprehensive unit tests covering all implemented CPU functionality

## [0.1.1] - 2025-09-13

### Added

- Complete CHIP-8 memory system with 4KB RAM
- Built-in hexadecimal font set (0-F) at address 0x050
- ROM loading with size validation and error handling
- Memory write protection for interpreter area (0x000-0x1FF)
- CLI flag `--disable-write-protection` to override memory protection
- Professional binary + library architecture (lib.rs + main.rs)
- Comprehensive unit and integration test suite
- Memory boundary checking and big-endian 16-bit operations

### Changed

- Refactored project to use library + binary pattern for better modularity
- Implemented lean testing philosophy - focused, non-redundant tests
- Updated documentation with testing tenets and development workflow
- Improved error handling with descriptive error messages using `thiserror`

### Technical

- Added `thiserror` dependency for structured error handling
- Centralized CHIP-8 constants in `lib.rs`
- Created proper integration test directory structure
- Enhanced CLI with global options support

## [0.1.0] - 2025-09-13

### Added

- Initial project structure with Cargo.toml and basic CLI
- Git-based semantic versioning system with build.rs
- Comprehensive CLI with subcommand architecture (`octo version`)
- Cross-platform build script for version management
- Automatic development version detection (e.g., "0.1.0-dev.5")
- Build-time validation of git tags vs project version
- Professional tooling with justfile for common development tasks
- MIT license and comprehensive README documentation

### Technical

- Implemented build.rs with git integration for version management
- Added clap for CLI argument parsing with derive API
- Created chrono-based build timestamp generation
- Set up automatic git hash, branch, and dirty status detection

[Unreleased]: https://github.com/sleb/octo/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/sleb/octo/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/sleb/octo/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/sleb/octo/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/sleb/octo/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/sleb/octo/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/sleb/octo/releases/tag/v0.1.0
