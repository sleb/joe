# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/username/octo/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/username/octo/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/username/octo/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/username/octo/releases/tag/v0.1.0
