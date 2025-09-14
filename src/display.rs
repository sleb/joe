//! CHIP-8 Display System
//!
//! Implements the 64x32 monochrome display with XOR sprite drawing and collision detection.
//! Separates logical display operations from physical rendering concerns.

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

/// Display bus trait for CPU to interact with display system
///
/// This trait defines the logical display operations independent of how
/// the display is physically rendered (ASCII, GUI, headless, etc.).
pub trait DisplayBus {
    /// Clear the entire display (set all pixels to off)
    fn clear(&mut self);

    /// Draw a sprite at position (x, y) using XOR logic
    ///
    /// Returns true if any pixels were turned OFF (collision detected)
    ///
    /// # CHIP-8 Sprite Drawing Contract
    /// - Sprites are always 8 pixels wide
    /// - Sprite height is determined by sprite_data length (1-15 bytes)
    /// - Each byte represents one row of 8 pixels (MSB = leftmost pixel)
    /// - Drawing uses XOR: on->off, off->on
    /// - Coordinates wrap around screen edges (modulo arithmetic)
    /// - Collision occurs when any pixel changes from on to off
    fn draw_sprite(&mut self, x: u8, y: u8, sprite_data: &[u8]) -> Result<bool, DisplayError>;

    /// Get pixel state at coordinates (for testing and rendering)
    fn get_pixel(&self, x: usize, y: usize) -> bool;

    /// Set pixel state at coordinates (for testing)
    fn set_pixel(&mut self, x: usize, y: usize, on: bool);
}

/// Renderer trait for physically outputting the display
///
/// This trait separates the concern of how to render the logical display
/// from the display logic itself. Different renderers can be implemented
/// for different output methods (ASCII terminal, GUI, headless testing, etc.).
pub trait Renderer {
    /// Render the current state of a display
    fn render(&self, display: &dyn DisplayBus);

    /// Get the character width of each pixel for this renderer
    fn pixel_width(&self) -> usize;

    /// Get the character representation for an "on" pixel
    fn pixel_on_char(&self) -> &str;

    /// Get the character representation for an "off" pixel
    fn pixel_off_char(&self) -> &str;
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

/// ASCII terminal renderer for development and testing
pub struct AsciiRenderer;

impl Renderer for AsciiRenderer {
    fn render(&self, display: &dyn DisplayBus) {
        let border_width = DISPLAY_WIDTH * self.pixel_width();
        println!("┌{}┐", "─".repeat(border_width));

        for y in 0..DISPLAY_HEIGHT {
            print!("│");
            for x in 0..DISPLAY_WIDTH {
                let pixel = display.get_pixel(x, y);
                print!(
                    "{}",
                    if pixel {
                        self.pixel_on_char()
                    } else {
                        self.pixel_off_char()
                    }
                );
            }
            println!("│");
        }

        println!("└{}┘", "─".repeat(border_width));
    }

    fn pixel_width(&self) -> usize {
        2 // Double-wide characters
    }

    fn pixel_on_char(&self) -> &str {
        "██"
    }

    fn pixel_off_char(&self) -> &str {
        "  "
    }
}

/// Headless renderer for testing (no output)
pub struct HeadlessRenderer;

impl Renderer for HeadlessRenderer {
    fn render(&self, _display: &dyn DisplayBus) {
        // No output - used for testing
    }

    fn pixel_width(&self) -> usize {
        0 // No visual output
    }

    fn pixel_on_char(&self) -> &str {
        ""
    }

    fn pixel_off_char(&self) -> &str {
        ""
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
    fn test_ascii_renderer() {
        let mut display = Display::new();
        let renderer = AsciiRenderer;

        // Set a simple pattern
        display.set_pixel(0, 0, true);
        display.set_pixel(1, 0, true);
        display.set_pixel(0, 1, true);

        // Render should not panic (we can't easily test output, but we can test it doesn't crash)
        renderer.render(&display);
    }

    #[test]
    fn test_headless_renderer() {
        let mut display = Display::new();
        let renderer = HeadlessRenderer;

        // Set some pixels
        display.set_pixel(10, 5, true);
        display.set_pixel(20, 15, true);

        // Headless renderer should do nothing (no output, no panic)
        renderer.render(&display);

        // Pixels should still be set (renderer doesn't modify display)
        assert!(display.get_pixel(10, 5));
        assert!(display.get_pixel(20, 15));
    }

    #[test]
    fn test_renderer_trait_object() {
        let display = Display::new();

        // Test that we can use renderers as trait objects
        let renderers: Vec<Box<dyn Renderer>> =
            vec![Box::new(AsciiRenderer), Box::new(HeadlessRenderer)];

        for renderer in renderers {
            renderer.render(&display); // Should not panic
        }
    }

    #[test]
    fn test_pixel_width() {
        let ascii_renderer = AsciiRenderer;
        let headless_renderer = HeadlessRenderer;

        // ASCII renderer uses double-wide characters
        assert_eq!(ascii_renderer.pixel_width(), 2);
        assert_eq!(ascii_renderer.pixel_on_char(), "██");
        assert_eq!(ascii_renderer.pixel_off_char(), "  ");

        // Headless renderer has no visual output
        assert_eq!(headless_renderer.pixel_width(), 0);
        assert_eq!(headless_renderer.pixel_on_char(), "");
        assert_eq!(headless_renderer.pixel_off_char(), "");
    }
}
