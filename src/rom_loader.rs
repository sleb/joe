//! ROM loading functionality for both local files and remote URLs
//!
//! This module provides a unified interface for loading CHIP-8 ROM data
//! from either local filesystem paths or HTTP(S) URLs.

use anyhow::{Context, Result};
use std::time::Duration;

/// Configuration for ROM loading operations
#[derive(Debug, Clone)]
pub struct RomLoaderConfig {
    /// Timeout for HTTP requests
    pub http_timeout: Duration,
    /// Maximum ROM size in bytes
    pub max_rom_size: usize,
}

impl Default for RomLoaderConfig {
    fn default() -> Self {
        Self {
            http_timeout: Duration::from_secs(30),
            max_rom_size: 4096 - 512, // CHIP-8 memory minus interpreter area
        }
    }
}

/// Represents different types of ROM sources
#[derive(Debug, Clone, PartialEq)]
pub enum RomSource {
    /// Local filesystem path
    File(String),
    /// HTTP or HTTPS URL
    Url(String),
}

impl RomSource {
    /// Detect the source type from a string input
    pub fn from_string(input: &str) -> Self {
        if input.starts_with("http://") || input.starts_with("https://") {
            Self::Url(input.to_string())
        } else {
            Self::File(input.to_string())
        }
    }

    /// Get a human-readable description of the source
    pub fn description(&self) -> &str {
        match self {
            Self::File(path) => path,
            Self::Url(url) => url,
        }
    }

    /// Check if this is a URL source
    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    /// Check if this is a file source
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }
}

/// Load ROM data from either a file or URL
pub fn load_rom_data(input: &str) -> Result<Vec<u8>> {
    load_rom_data_with_config(input, &RomLoaderConfig::default())
}

/// Load ROM data with custom configuration
pub fn load_rom_data_with_config(input: &str, config: &RomLoaderConfig) -> Result<Vec<u8>> {
    let source = RomSource::from_string(input);

    let data = match source {
        RomSource::File(path) => load_from_file(&path)
            .with_context(|| format!("Failed to load ROM from file: {}", path))?,
        RomSource::Url(url) => load_from_url(&url, config)
            .with_context(|| format!("Failed to load ROM from URL: {}", url))?,
    };

    // Validate ROM size
    if data.len() > config.max_rom_size {
        anyhow::bail!(
            "ROM too large: {} bytes (max: {} bytes)",
            data.len(),
            config.max_rom_size
        );
    }

    if data.is_empty() {
        anyhow::bail!("ROM is empty");
    }

    Ok(data)
}

/// Load ROM data from a local file
fn load_from_file(path: &str) -> Result<Vec<u8>> {
    let path = std::path::Path::new(path);

    if !path.exists() {
        anyhow::bail!("ROM file '{}' not found", path.display());
    }

    if !path.is_file() {
        anyhow::bail!("'{}' is not a file", path.display());
    }

    std::fs::read(path).with_context(|| format!("Failed to read ROM file: {}", path.display()))
}

/// Load ROM data from a URL
fn load_from_url(url: &str, config: &RomLoaderConfig) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(config.http_timeout)
        .user_agent("joe-chip8-emulator/0.2.0")
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(url)
        .send()
        .context("Failed to send HTTP request")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "HTTP request failed with status {}: {}",
            response.status(),
            response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error")
        );
    }

    // Check content length if provided
    if let Some(content_length) = response.content_length() {
        if content_length as usize > config.max_rom_size {
            anyhow::bail!(
                "ROM too large: {} bytes (max: {} bytes)",
                content_length,
                config.max_rom_size
            );
        }
    }

    let bytes = response.bytes().context("Failed to read response body")?;

    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_source_detection() {
        // Test URL detection
        assert_eq!(
            RomSource::from_string("https://example.com/rom.ch8"),
            RomSource::Url("https://example.com/rom.ch8".to_string())
        );
        assert_eq!(
            RomSource::from_string("http://localhost:8080/test.ch8"),
            RomSource::Url("http://localhost:8080/test.ch8".to_string())
        );

        // Test file path detection
        assert_eq!(
            RomSource::from_string("roms/test.ch8"),
            RomSource::File("roms/test.ch8".to_string())
        );
        assert_eq!(
            RomSource::from_string("/absolute/path/rom.ch8"),
            RomSource::File("/absolute/path/rom.ch8".to_string())
        );
        assert_eq!(
            RomSource::from_string("./relative/rom.ch8"),
            RomSource::File("./relative/rom.ch8".to_string())
        );
    }

    #[test]
    fn test_rom_source_methods() {
        let url_source = RomSource::Url("https://example.com/rom.ch8".to_string());
        assert!(url_source.is_url());
        assert!(!url_source.is_file());
        assert_eq!(url_source.description(), "https://example.com/rom.ch8");

        let file_source = RomSource::File("roms/test.ch8".to_string());
        assert!(file_source.is_file());
        assert!(!file_source.is_url());
        assert_eq!(file_source.description(), "roms/test.ch8");
    }

    #[test]
    fn test_config_default() {
        let config = RomLoaderConfig::default();
        assert_eq!(config.http_timeout, Duration::from_secs(30));
        assert_eq!(config.max_rom_size, 4096 - 512);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_rom_data("nonexistent_file.ch8");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("not found") || error_msg.contains("Failed to load ROM from file")
        );
    }

    #[test]
    fn test_empty_input() {
        let result = load_from_file("");
        assert!(result.is_err());
    }

    // Note: We don't test actual HTTP requests in unit tests to avoid dependencies
    // on external services. Integration tests could test this with a local server.
}
