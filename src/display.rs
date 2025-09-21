//! CHIP-8 Display System
//!
//! Implements the 64x32 monochrome display with XOR sprite drawing and collision detection.
//! Includes ratatui-based terminal renderer for rich interactive display.

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
        size as terminal_size,
    },
    tty::IsTty,
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::{
    collections::VecDeque,
    io::{self, Stdout, stdout},
    time::{Duration, Instant},
};
use thiserror::Error;

/// Display width in pixels
pub const DISPLAY_WIDTH: usize = 64;

/// Display height in pixels
pub const DISPLAY_HEIGHT: usize = 32;

/// Maximum sprite width (always 8 pixels in CHIP-8)
pub const SPRITE_WIDTH: usize = 8;

/// Display errors
#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("Sprite data is empty")]
    EmptySpriteData,

    #[error("Sprite too tall: {height} rows (max: {max_height})")]
    SpriteTooTall { height: usize, max_height: usize },
}

/// Control action requested by the renderer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlAction {
    /// No action requested
    None,
    /// Reset the emulator
    Reset,
    /// Toggle pause/resume (future feature)
    TogglePause,
    /// Quit the emulator
    Quit,
}

/// Display bus trait for CPU to interact with display system
///
/// This trait defines the logical display operations that the CPU needs,
/// independent of the physical rendering implementation.
pub trait DisplayBus {
    /// Clear the entire display (set all pixels to off)
    fn clear(&mut self);

    /// Draw a sprite at position (x, y) using XOR logic
    ///
    /// Returns true if any pixels were turned OFF (collision detected)
    fn draw_sprite(&mut self, x: u8, y: u8, sprite_data: &[u8]) -> Result<bool, DisplayError>;

    /// Get pixel state at coordinates (for testing and rendering)
    fn get_pixel(&self, x: usize, y: usize) -> bool;

    /// Set pixel state at coordinates (for testing)
    fn set_pixel(&mut self, x: usize, y: usize, on: bool);
}

/// Renderer errors
#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Terminal initialization failed: {0}")]
    TerminalInit(#[from] io::Error),

    #[error("Terminal too small: {width}x{height} (minimum: 80x24)")]
    TerminalTooSmall { width: u16, height: u16 },

    #[error("Not running in a TTY - emulator requires a terminal")]
    NotATty,

    #[error("Crossterm error: {0}")]
    CrosstermError(String),
}

/// CHIP-8 Display implementation with 64x32 framebuffer
pub struct Display {
    /// 64x32 framebuffer: framebuffer[row][col] = pixel_on
    framebuffer: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Display {
    /// Create a new display with all pixels off
    pub fn new() -> Self {
        Self {
            framebuffer: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

    /// Get display statistics
    pub fn get_stats(&self) -> DisplayStats {
        let mut pixels_on = 0;
        for row in &self.framebuffer {
            for &pixel in row {
                if pixel {
                    pixels_on += 1;
                }
            }
        }

        DisplayStats {
            width: DISPLAY_WIDTH,
            height: DISPLAY_HEIGHT,
            pixels_on,
            pixels_total: DISPLAY_WIDTH * DISPLAY_HEIGHT,
        }
    }
}

impl DisplayBus for Display {
    fn clear(&mut self) {
        self.framebuffer = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    }

    fn draw_sprite(&mut self, x: u8, y: u8, sprite_data: &[u8]) -> Result<bool, DisplayError> {
        if sprite_data.is_empty() {
            return Err(DisplayError::EmptySpriteData);
        }

        if sprite_data.len() > 15 {
            return Err(DisplayError::SpriteTooTall {
                height: sprite_data.len(),
                max_height: 15,
            });
        }

        let mut collision = false;

        // Draw each row of the sprite
        for (row_offset, &sprite_byte) in sprite_data.iter().enumerate() {
            // Calculate wrapped coordinates
            let screen_y = ((y as usize) + row_offset) % DISPLAY_HEIGHT;

            // Draw each pixel in the row (8 pixels per byte)
            for bit_pos in 0..8 {
                let screen_x = ((x as usize) + bit_pos) % DISPLAY_WIDTH;

                // Extract pixel from sprite byte (MSB = leftmost pixel)
                let sprite_pixel = (sprite_byte >> (7 - bit_pos)) & 1 == 1;

                if sprite_pixel {
                    // XOR the pixel
                    let old_pixel = self.framebuffer[screen_y][screen_x];
                    let new_pixel = old_pixel ^ true; // XOR with sprite pixel (on)
                    self.framebuffer[screen_y][screen_x] = new_pixel;

                    // Collision occurs when pixel turns off (was on, now off)
                    if old_pixel && !new_pixel {
                        collision = true;
                    }
                }
            }
        }

        Ok(collision)
    }

    fn get_pixel(&self, x: usize, y: usize) -> bool {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            false
        } else {
            self.framebuffer[y][x]
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        if x < DISPLAY_WIDTH && y < DISPLAY_HEIGHT {
            self.framebuffer[y][x] = on;
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

/// Display system statistics
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayStats {
    pub width: usize,
    pub height: usize,
    pub pixels_on: usize,
    pub pixels_total: usize,
}

/// Configuration for the ratatui renderer
#[derive(Debug, Clone)]
pub struct RatatuiConfig {
    pub theme: String,
    pub show_cpu_registers: bool,
    pub show_performance_stats: bool,
    pub show_input_status: bool,
    pub show_memory_info: bool,
    pub pixel_char: String,
    pub pixel_color: String,
    pub border_style: String,
    pub refresh_rate_ms: u64,
}

impl Default for RatatuiConfig {
    fn default() -> Self {
        Self {
            theme: "classic".to_string(),
            show_cpu_registers: true,
            show_performance_stats: true,
            show_input_status: true,
            show_memory_info: true,
            pixel_char: "██".to_string(),
            pixel_color: "Green".to_string(),
            border_style: "rounded".to_string(),
            refresh_rate_ms: 16,
        }
    }
}

impl RatatuiConfig {
    /// Parse a color string into a ratatui Color
    pub fn parse_color(color_str: &str) -> Color {
        match color_str.to_lowercase().as_str() {
            "green" => Color::Green,
            "white" => Color::White,
            "blue" => Color::Blue,
            "red" => Color::Red,
            "yellow" => Color::Yellow,
            "cyan" => Color::Cyan,
            "magenta" => Color::Magenta,
            "gray" => Color::Gray,
            "dark_gray" => Color::DarkGray,
            _ => Color::Green, // Default fallback
        }
    }

    /// Create RatatuiConfig from user DisplaySettings
    pub fn from_display_settings(display_settings: &crate::config::DisplaySettings) -> Self {
        Self {
            theme: display_settings.theme.clone(),
            show_cpu_registers: true,
            show_performance_stats: true,
            show_input_status: true,
            show_memory_info: true,
            pixel_char: display_settings.pixel_char.clone(),
            pixel_color: display_settings.pixel_color.clone(),
            border_style: "rounded".to_string(),
            refresh_rate_ms: display_settings.refresh_rate_ms,
        }
    }
}

/// Ratatui-based terminal renderer for rich interactive display
pub struct RatatuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    config: RatatuiConfig,
    stats_history: VecDeque<(Instant, usize)>, // (timestamp, cycles) for FPS calculation
    last_render: Instant,
}

impl RatatuiRenderer {
    /// Create a new ratatui renderer
    pub fn new(config: RatatuiConfig) -> Result<Self, RendererError> {
        // Validate terminal capabilities upfront
        Self::validate_terminal()?;

        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            config,
            stats_history: VecDeque::with_capacity(100),
            last_render: Instant::now(),
        })
    }

    fn validate_terminal() -> Result<(), RendererError> {
        // Check if we're in a TTY
        if !IsTty::is_tty(&io::stdin()) {
            return Err(RendererError::NotATty);
        }

        // Check terminal size
        let (width, height) = terminal_size()?;
        if width < 80 || height < 20 {
            return Err(RendererError::TerminalTooSmall { width, height });
        }

        Ok(())
    }

    /// Render the display with emulator stats
    pub fn render(
        &mut self,
        display: &Display,
        cycles_executed: usize,
    ) -> Result<ControlAction, RendererError> {
        // Process any pending terminal events and get any control actions
        let control_action = self.handle_events()?;

        // Update stats history for FPS calculation
        let now = Instant::now();
        self.stats_history.push_back((now, cycles_executed));

        // Keep only recent history (last 2 seconds)
        while let Some((timestamp, _)) = self.stats_history.front() {
            if now.duration_since(*timestamp).as_secs() > 2 {
                self.stats_history.pop_front();
            } else {
                break;
            }
        }

        // Only render at configured rate to avoid excessive redraws
        if now.duration_since(self.last_render).as_millis() < self.config.refresh_rate_ms as u128 {
            return Ok(control_action);
        }
        self.last_render = now;

        // Render the UI
        let config = &self.config;
        let stats_history = &self.stats_history;
        self.terminal
            .draw(|f| Self::draw_ui_static(f, display, cycles_executed, config, stats_history))?;

        Ok(control_action)
    }

    fn handle_events(&mut self) -> Result<ControlAction, RendererError> {
        // Handle ratatui-specific control keys (non-blocking)
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            return Ok(ControlAction::Quit);
                        }
                        KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                            return Ok(ControlAction::Reset);
                        }
                        KeyCode::Char(' ') => {
                            return Ok(ControlAction::TogglePause);
                        }
                        KeyCode::Esc => {
                            return Ok(ControlAction::Quit);
                        }
                        _ => {
                            // Other keys are handled by the existing Input system
                            // We don't interfere with CHIP-8 game keys here
                        }
                    }
                }
            }
        }
        Ok(ControlAction::None)
    }

    fn draw_ui_static(
        f: &mut Frame,
        display: &Display,
        cycles_executed: usize,
        config: &RatatuiConfig,
        stats_history: &VecDeque<(Instant, usize)>,
    ) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(f.area());

        // Header
        Self::draw_header_static(f, chunks[0]);

        // Use the whole width for the display
        Self::draw_display_static(f, chunks[1], display, config);

        // Status bar
        Self::draw_status_bar_static(f, chunks[2], cycles_executed, stats_history, config);
    }

    fn draw_header_static(f: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled(
                "JOE ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("CHIP-8 Emulator v0.4.0", Style::default().fg(Color::White)),
        ]);

        let header = Paragraph::new(title)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        f.render_widget(header, area);
    }

    fn draw_display_static(f: &mut Frame, area: Rect, display: &Display, config: &RatatuiConfig) {
        let mut lines = Vec::new();
        let chip8_width = DISPLAY_WIDTH;
        let chip8_height = DISPLAY_HEIGHT;
        let area_width = area.width as usize;

        // Calculate the actual character width of each pixel
        let pixel_char_width = config.pixel_char.chars().count();
        let total_display_width = chip8_width * pixel_char_width;

        // Calculate horizontal padding for centering
        let pad_left = if area_width > total_display_width {
            (area_width - total_display_width) / 2
        } else {
            0
        };

        for y in 0..chip8_height {
            let mut line_spans = Vec::new();
            // Add left padding if needed
            for _ in 0..pad_left {
                line_spans.push(Span::raw(" "));
            }
            for x in 0..chip8_width {
                let pixel = display.get_pixel(x, y);
                let pixel_color = RatatuiConfig::parse_color(&config.pixel_color);
                line_spans.push(Span::styled(
                    &config.pixel_char,
                    if pixel {
                        Style::default().fg(pixel_color)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ));
            }
            lines.push(Line::from(line_spans));
        }

        let display_widget = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("CHIP-8 Display"),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(display_widget, area);
    }

    // Side panel removed

    fn draw_status_bar_static(
        f: &mut Frame,
        area: Rect,
        cycles_executed: usize,
        stats_history: &VecDeque<(Instant, usize)>,
        config: &RatatuiConfig,
    ) {
        let fps = Self::calculate_fps_static(stats_history);
        let status_text = Line::from(format!(
            "Running • Cycles: {} • FPS: {:.1} • Theme: {} | Controls: Ctrl+C=Quit, Space=Pause, Ctrl+R=Reset",
            cycles_executed, fps, config.theme
        ));

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        f.render_widget(status, area);
    }

    fn calculate_fps_static(stats_history: &VecDeque<(Instant, usize)>) -> f64 {
        if stats_history.len() < 2 {
            return 0.0;
        }

        let (oldest_time, oldest_cycles) = stats_history.front().unwrap();
        let (newest_time, newest_cycles) = stats_history.back().unwrap();

        let duration_secs = newest_time.duration_since(*oldest_time).as_secs_f64();
        if duration_secs > 0.0 {
            let cycles_diff = newest_cycles.saturating_sub(*oldest_cycles) as f64;
            cycles_diff / duration_secs
        } else {
            0.0
        }
    }
}

impl Drop for RatatuiRenderer {
    fn drop(&mut self) {
        // Clean up terminal state
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_display_is_clear() {
        let display = Display::new();
        let stats = display.get_stats();

        assert_eq!(stats.pixels_on, 0);
        assert_eq!(stats.pixels_total, DISPLAY_WIDTH * DISPLAY_HEIGHT);

        // Check all pixels are off
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                assert!(!display.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_clear_display() {
        let mut display = Display::new();

        // Set some pixels manually
        display.framebuffer[0][0] = true;
        display.framebuffer[10][20] = true;
        assert!(display.get_pixel(0, 0));
        assert!(display.get_pixel(20, 10));

        // Clear and verify all pixels are off
        display.clear();
        assert!(!display.get_pixel(0, 0));
        assert!(!display.get_pixel(20, 10));
        assert_eq!(display.get_stats().pixels_on, 0);
    }

    #[test]
    fn test_sprite_drawing_basic() {
        let mut display = Display::new();

        // Draw a simple 1x1 sprite (single pixel)
        let sprite = [0b10000000]; // Top-left pixel only
        let collision = display.draw_sprite(0, 0, &sprite).unwrap();

        assert!(!collision); // No collision on first draw
        assert!(display.get_pixel(0, 0)); // Pixel should be on
        assert!(!display.get_pixel(1, 0)); // Adjacent pixel should be off
    }

    #[test]
    fn test_sprite_drawing_xor() {
        let mut display = Display::new();

        // Draw sprite twice at same location to test XOR
        let sprite = [0b11110000]; // Left 4 pixels on

        // First draw - pixels turn on
        let collision1 = display.draw_sprite(0, 0, &sprite).unwrap();
        assert!(!collision1); // No collision (pixels turned on)
        assert!(display.get_pixel(0, 0));
        assert!(display.get_pixel(3, 0));
        assert!(!display.get_pixel(4, 0));

        // Second draw - pixels turn off (XOR)
        let collision2 = display.draw_sprite(0, 0, &sprite).unwrap();
        assert!(collision2); // Collision detected (pixels turned off)
        assert!(!display.get_pixel(0, 0));
        assert!(!display.get_pixel(3, 0));
    }

    #[test]
    fn test_sprite_drawing_collision_detection() {
        let mut display = Display::new();

        // Set up initial pixels
        display.framebuffer[0][1] = true; // Pixel at (1, 0) is on

        // Draw sprite that will turn off the existing pixel
        let sprite = [0b01000000]; // Second pixel from left
        let collision = display.draw_sprite(0, 0, &sprite).unwrap();

        assert!(collision); // Should detect collision
        assert!(!display.get_pixel(1, 0)); // Pixel should be off now
    }

    #[test]
    fn test_coordinate_wrapping() {
        let mut display = Display::new();

        // Draw sprite at edge coordinates that should wrap
        let sprite = [0b11111111]; // Full 8-pixel row

        // Draw at right edge - should wrap to left
        let collision = display.draw_sprite(62, 0, &sprite).unwrap();
        assert!(!collision);

        // Check wrapped pixels
        assert!(display.get_pixel(62, 0)); // Original position
        assert!(display.get_pixel(63, 0)); // Last column
        assert!(display.get_pixel(0, 0)); // Wrapped to first column
        assert!(display.get_pixel(5, 0)); // Wrapped pixels continue

        // Draw at bottom edge - should wrap to top
        display.clear();
        let tall_sprite = [0b10000000; 3]; // 3-row sprite, left pixel only
        let collision = display.draw_sprite(0, 31, &tall_sprite).unwrap();
        assert!(!collision);

        // Check wrapped rows
        assert!(display.get_pixel(0, 31)); // Original position
        assert!(display.get_pixel(0, 0)); // Wrapped to top
        assert!(display.get_pixel(0, 1)); // Second wrapped row
    }

    #[test]
    fn test_multi_row_sprite() {
        let mut display = Display::new();

        // Create a 3x3 "plus" pattern
        let sprite = [
            0b01000000, // Row 0:  .#......
            0b11100000, // Row 1:  ###.....
            0b01000000, // Row 2:  .#......
        ];

        let collision = display.draw_sprite(10, 5, &sprite).unwrap();
        assert!(!collision);

        // Verify the pattern
        assert!(display.get_pixel(11, 5)); // Top center
        assert!(display.get_pixel(10, 6)); // Middle left
        assert!(display.get_pixel(11, 6)); // Middle center
        assert!(display.get_pixel(12, 6)); // Middle right
        assert!(display.get_pixel(11, 7)); // Bottom center

        // Verify surrounding pixels are off
        assert!(!display.get_pixel(9, 5));
        assert!(!display.get_pixel(13, 6));
    }

    #[test]
    fn test_empty_sprite_error() {
        let mut display = Display::new();

        let result = display.draw_sprite(0, 0, &[]);
        assert!(matches!(result, Err(DisplayError::EmptySpriteData)));
    }

    #[test]
    fn test_sprite_too_tall_error() {
        let mut display = Display::new();

        let tall_sprite = [0xFF; 16]; // 16 rows (too tall)
        let result = display.draw_sprite(0, 0, &tall_sprite);
        assert!(matches!(
            result,
            Err(DisplayError::SpriteTooTall {
                height: 16,
                max_height: 15
            })
        ));
    }

    #[test]
    fn test_get_pixel_bounds() {
        let display = Display::new();

        // Valid coordinates
        assert!(!display.get_pixel(0, 0));
        assert!(!display.get_pixel(63, 31));

        // Out of bounds coordinates should return false
        assert!(!display.get_pixel(64, 0));
        assert!(!display.get_pixel(0, 32));
        assert!(!display.get_pixel(100, 100));
    }

    #[test]
    fn test_terminal_validation() {
        // We can't easily test terminal validation without mocking,
        // but we can test that the validation function exists
        // In a real terminal environment, this would properly validate
        let result = RatatuiRenderer::validate_terminal();
        // Result depends on test environment - could pass or fail
        match result {
            Ok(()) => {
                // Terminal validation passed - we're in a proper terminal
            }
            Err(RendererError::NotATty) => {
                // Expected when running in CI or non-TTY environment
            }
            Err(RendererError::TerminalTooSmall { .. }) => {
                // Expected if terminal is smaller than 80x24
            }
            Err(_) => {
                // Other errors are also acceptable in test environment
            }
        }
    }
}
