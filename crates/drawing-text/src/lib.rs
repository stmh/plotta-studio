//! Single-line font support for plotta-studio
//!
//! This crate provides vector stroke font rendering for pen plotters.
//! It supports multiple font formats:
//!
//! - **Hershey fonts**: Classic public domain stroke fonts from 1967
//! - **VSF (Vector Stroke Font)**: Modern JSON-based format with bezier support
//! - **UFO 3**: Industry standard font format (future)
//!
//! # Example
//!
//! ```rust,ignore
//! use drawing_text::{HersheyFont, Font, TextRenderer, TextOptions};
//!
//! // Load built-in Hershey Simplex font
//! let font = drawing_text::hershey::load_simplex().unwrap();
//!
//! // Create text renderer
//! let renderer = TextRenderer::new();
//!
//! // Render text to strokes
//! let options = TextOptions::new(24.0).at((100.0, 100.0));
//! let layout = renderer.layout("Hello, World!", &font, &options);
//! let strokes = layout.to_strokes(drawing_core::Style::default(), 0.1);
//! ```

pub mod error;
pub mod font;
pub mod hershey;
pub mod svgfont;
pub mod types;
pub mod vsf;

// Re-export main types
pub use error::FontError;
pub use font::{Font, FontFormat, FontLoader, FontSource};
pub use hershey::HersheyFont;
pub use svgfont::SvgFont;
pub use types::{
    Contour, ContourSegment, FontMetrics, Glyph, PositionedGlyph, TextAlign, TextLayout,
    TextOptions,
};
pub use vsf::VsfFont;

use drawing_core::Point;

/// Text renderer for laying out and rendering text using fonts
#[derive(Debug, Clone, Default)]
pub struct TextRenderer {
    /// Tolerance for curve flattening
    pub tolerance: f64,
}

impl TextRenderer {
    /// Create a new text renderer with default settings
    pub fn new() -> Self {
        Self { tolerance: 0.1 }
    }

    /// Set curve flattening tolerance
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Layout text using the given font and options
    pub fn layout(&self, text: &str, font: &dyn Font, options: &TextOptions) -> TextLayout {
        let metrics = font.metrics();
        let scale = options.size / metrics.units_per_em;

        let mut glyphs = Vec::new();
        let mut x = options.position.x;
        let y = options.position.y;

        let mut prev_char: Option<char> = None;
        let mut line_widths = Vec::new();
        let mut current_line_width = 0.0;
        let mut line_count = 1;

        // First pass: calculate layout
        for c in text.chars() {
            if c == '\n' {
                line_widths.push(current_line_width);
                current_line_width = 0.0;
                line_count += 1;
                prev_char = None;
                continue;
            }

            if let Some(glyph) = font.glyph(c) {
                // Apply kerning
                if let Some(prev) = prev_char {
                    let kern = font.kerning(prev, c) * scale;
                    current_line_width += kern;
                }

                // Add letter spacing
                if prev_char.is_some() {
                    current_line_width += options.letter_spacing * options.size;
                }

                // Add word spacing for spaces
                if c == ' ' {
                    current_line_width += options.word_spacing * options.size;
                }

                current_line_width += glyph.advance_width * scale;
                prev_char = Some(c);
            }
        }
        line_widths.push(current_line_width);

        // Second pass: position glyphs with alignment
        let mut line_index = 0;
        let line_height = metrics.line_height() * scale * options.line_height.unwrap_or(1.0);
        prev_char = None;

        for c in text.chars() {
            if c == '\n' {
                line_index += 1;
                x = options.position.x;
                prev_char = None;
                continue;
            }

            if let Some(glyph) = font.glyph(c) {
                // Calculate alignment offset for this line
                let line_width = line_widths.get(line_index).copied().unwrap_or(0.0);
                let align_offset = match options.align {
                    TextAlign::Left => 0.0,
                    TextAlign::Center => -line_width / 2.0,
                    TextAlign::Right => -line_width,
                };

                // Apply kerning
                if let Some(prev) = prev_char {
                    x += font.kerning(prev, c) * scale;
                }

                // Add letter spacing
                if prev_char.is_some() {
                    x += options.letter_spacing * options.size;
                }

                // Add word spacing for spaces
                if c == ' ' {
                    x += options.word_spacing * options.size;
                }

                // Position glyph
                if c != ' ' {
                    // Don't add space glyphs, just advance
                    let glyph_y = y + (line_index as f64) * line_height;
                    glyphs.push(PositionedGlyph {
                        glyph: glyph.clone(),
                        position: Point::new(x + align_offset, glyph_y),
                        scale,
                    });
                }

                x += glyph.advance_width * scale;
                prev_char = Some(c);
            }
        }

        // Calculate bounds
        let bounds = if glyphs.is_empty() {
            None
        } else {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;

            for pg in &glyphs {
                if let Some(glyph_bounds) = pg.glyph.bounds() {
                    let x0 = pg.position.x + glyph_bounds.x0 * pg.scale;
                    let y0 = pg.position.y - glyph_bounds.y1 * pg.scale; // Flip Y
                    let x1 = pg.position.x + glyph_bounds.x1 * pg.scale;
                    let y1 = pg.position.y - glyph_bounds.y0 * pg.scale; // Flip Y

                    min_x = min_x.min(x0);
                    min_y = min_y.min(y0);
                    max_x = max_x.max(x1);
                    max_y = max_y.max(y1);
                }
            }

            if min_x < max_x && min_y < max_y {
                Some(drawing_core::Rect::new(min_x, min_y, max_x, max_y))
            } else {
                None
            }
        };

        TextLayout {
            glyphs,
            bounds,
            line_count,
        }
    }

    /// Measure text without rendering
    pub fn measure(
        &self,
        text: &str,
        font: &dyn Font,
        options: &TextOptions,
    ) -> Option<drawing_core::Rect> {
        self.layout(text, font, options).bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_renderer_creation() {
        let renderer = TextRenderer::new();
        assert_eq!(renderer.tolerance, 0.1);

        let renderer = renderer.with_tolerance(0.5);
        assert_eq!(renderer.tolerance, 0.5);
    }

    #[test]
    fn test_layout_empty_text() {
        let font = hershey::load_simplex().unwrap();
        let renderer = TextRenderer::new();
        let options = TextOptions::new(24.0);
        let layout = renderer.layout("", &font, &options);

        assert!(layout.glyphs.is_empty());
        assert!(layout.bounds.is_none());
        assert_eq!(layout.line_count, 1);
    }

    #[test]
    fn test_layout_simple_text() {
        let font = hershey::load_simplex().unwrap();
        let renderer = TextRenderer::new();
        let options = TextOptions::new(24.0).at((100.0, 100.0));
        let layout = renderer.layout("ABC", &font, &options);

        assert_eq!(layout.glyphs.len(), 3);
        assert!(layout.bounds.is_some());
    }

    #[test]
    fn test_layout_multiline() {
        let font = hershey::load_simplex().unwrap();
        let renderer = TextRenderer::new();
        let options = TextOptions::new(24.0);
        let layout = renderer.layout("AB\nCD", &font, &options);

        assert_eq!(layout.line_count, 2);
    }
}
