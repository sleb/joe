# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CPU module implementation (in progress)

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

[Unreleased]: https://github.com/username/octo/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/username/octo/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/username/octo/releases/tag/v0.1.0
