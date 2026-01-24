//! Text layout and rendering types
//!
//! # Coordinate Systems
//!
//! This module handles conversion between two coordinate systems:
//!
//! - **Font space**: Y-axis points UP (positive Y = ascenders, negative Y = descenders).
//!   This is the standard for font files (TrueType, OpenType, SVG fonts).
//!
//! - **Plotter/Drawing space**: Y-axis points DOWN (positive Y = lower on page).
//!   This matches screen coordinates and our drawing system.
//!
//! The [`font_y_to_drawing`] function centralizes this conversion to ensure consistency.

use crate::font_registry::FontRef;
use crate::stroke::Stroke;
use crate::style::ResolvedStyle;
use kurbo::{Point, Rect};
use log::warn;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Convert a Y coordinate from font space (Y-up) to drawing space (Y-down).
///
/// Font glyphs use Y-up coordinates where:
/// - Positive Y = ascenders (parts above baseline, like 'h', 'l')
/// - Negative Y = descenders (parts below baseline, like 'p', 'g')
///
/// Our drawing system uses Y-down coordinates (like screen coordinates).
/// This function negates Y to flip the coordinate system.
///
/// # Arguments
/// * `font_y` - Y coordinate in font space
/// * `scale` - Scale factor (typically size / units_per_em)
/// * `baseline_y` - Y position of the baseline in drawing space
#[inline]
fn font_y_to_drawing(font_y: f64, scale: f64, baseline_y: f64) -> f64 {
    baseline_y - font_y * scale
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Text rendering options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOptions {
    /// Font size in drawing units
    pub size: f64,
    /// Baseline start position
    pub position: Point,
    /// Text alignment
    pub align: TextAlign,
    /// Override line spacing (multiplier, 1.0 = normal)
    pub line_height: Option<f64>,
    /// Additional spacing between glyphs (in em units)
    pub letter_spacing: f64,
    /// Additional spacing between words (in em units)
    pub word_spacing: f64,
    /// Tolerance for curve flattening (lower = finer curves, higher = coarser)
    pub tolerance: f64,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            size: 12.0,
            position: Point::ORIGIN,
            align: TextAlign::Left,
            line_height: None,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            tolerance: 0.5,
        }
    }
}

impl TextOptions {
    /// Create new text options with given size
    pub fn new(size: f64) -> Self {
        Self {
            size,
            ..Default::default()
        }
    }

    /// Set position
    pub fn at(mut self, position: impl Into<Point>) -> Self {
        self.position = position.into();
        self
    }

    /// Set alignment
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Set line height multiplier
    pub fn line_height(mut self, height: f64) -> Self {
        self.line_height = Some(height);
        self
    }

    /// Set letter spacing
    pub fn letter_spacing(mut self, spacing: f64) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Set word spacing
    pub fn word_spacing(mut self, spacing: f64) -> Self {
        self.word_spacing = spacing;
        self
    }

    /// Set curve flattening tolerance (lower = finer curves, higher = coarser)
    ///
    /// Default is 0.5. Use lower values (e.g., 0.1) for high-quality output
    /// or higher values (e.g., 1.0) for faster rendering with fewer points.
    pub fn tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }
}

/// A positioned glyph ready for rendering
///
/// Stores minimal data (character, position, scale) rather than cloning
/// the full glyph. The actual glyph data is looked up from the font
/// stored in `TextLayout` when rendering.
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    /// The character (used to look up glyph from font)
    pub char: char,
    /// Position (baseline origin)
    pub position: Point,
    /// Scale factor (size / units_per_em)
    pub scale: f64,
}

/// Result of text layout
///
/// Contains the font reference and positioned glyphs. The font is stored
/// to enable glyph lookup during rendering without requiring the caller
/// to pass the font again.
#[derive(Clone)]
pub struct TextLayout {
    /// The font used for this layout (keeps font alive)
    pub font: FontRef,
    /// Positioned glyphs
    pub glyphs: Vec<PositionedGlyph>,
    /// Bounding box of the entire text
    pub bounds: Option<Rect>,
    /// Number of lines
    pub line_count: usize,
}

impl std::fmt::Debug for TextLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextLayout")
            .field("font", &self.font.name())
            .field("glyphs", &self.glyphs.len())
            .field("bounds", &self.bounds)
            .field("line_count", &self.line_count)
            .finish()
    }
}

impl TextLayout {
    /// Convert entire layout to strokes
    pub fn to_strokes(&self, style: ResolvedStyle, tolerance: f64) -> Vec<Stroke> {
        let font = &self.font;
        self.glyphs
            .iter()
            .filter_map(|pg| {
                font.glyph(pg.char).map(|glyph| {
                    glyph
                        .contours
                        .iter()
                        .map(|contour| {
                            // Scale and translate points, converting from font space to drawing space
                            let scaled_points: Vec<Point> = contour
                                .flatten(tolerance)
                                .iter()
                                .map(|p| {
                                    Point::new(
                                        p.x * pg.scale + pg.position.x,
                                        font_y_to_drawing(p.y, pg.scale, pg.position.y),
                                    )
                                })
                                .collect();

                            let mut stroke = Stroke::new(scaled_points, style);
                            if contour.closed {
                                stroke = stroke.closed();
                            }
                            stroke
                        })
                        .collect::<Vec<_>>()
                })
            })
            .flatten()
            .collect()
    }
}

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
    pub fn layout(&self, text: &str, font: FontRef, options: &TextOptions) -> TextLayout {
        let metrics = font.metrics();
        let scale = options.size / metrics.units_per_em;

        let mut glyphs = Vec::new();
        let mut x = options.position.x;
        let y = options.position.y;

        let mut prev_char: Option<char> = None;
        let mut line_widths = Vec::new();
        let mut current_line_width = 0.0;
        let mut line_count = 1;

        // Fallback space width: 1/3 of em-square
        //
        // This heuristic is based on typical Latin font metrics where the space
        // character is usually between 1/4 and 1/3 of the em-square. We use 1/3
        // as a reasonable default that works well for most text fonts.
        //
        // Reference: Most fonts define space width as roughly 250-333 units in
        // a 1000 UPM font, which is 25-33% of em.
        let fallback_space_width = metrics.units_per_em / 3.0;

        // Track if we've already warned about missing space glyph for this layout
        static WARNED_MISSING_SPACE: AtomicBool = AtomicBool::new(false);

        // First pass: calculate layout
        for c in text.chars() {
            if c == '\n' {
                line_widths.push(current_line_width);
                current_line_width = 0.0;
                line_count += 1;
                prev_char = None;
                continue;
            }

            // Handle space specially - use fallback if font doesn't have space glyph
            if c == ' ' {
                let space_width = font.glyph(' ').map(|g| g.advance_width).unwrap_or_else(|| {
                    // Warn once about missing space glyph (helps debug font issues)
                    if !WARNED_MISSING_SPACE.swap(true, Ordering::Relaxed) {
                        warn!(
                            "Font '{}' has no space glyph, using fallback width (1/3 em = {:.1} units)",
                            font.name(),
                            fallback_space_width
                        );
                    }
                    fallback_space_width
                });
                current_line_width += space_width * scale;
                current_line_width += options.word_spacing * options.size;
                prev_char = Some(c);
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

            // Handle space specially - advance position even if font doesn't have space glyph
            // (warning was already emitted in first pass if fallback is used)
            if c == ' ' {
                let space_width = font
                    .glyph(' ')
                    .map(|g| g.advance_width)
                    .unwrap_or(fallback_space_width);
                x += space_width * scale;
                x += options.word_spacing * options.size;
                prev_char = Some(c);
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

                // Position glyph - store char instead of cloning glyph
                let glyph_y = y + (line_index as f64) * line_height;
                glyphs.push(PositionedGlyph {
                    char: c,
                    position: Point::new(x + align_offset, glyph_y),
                    scale,
                });

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
                if let Some(glyph) = font.glyph(pg.char) {
                    if let Some(glyph_bounds) = glyph.bounds() {
                        // Convert bounds from font space to drawing space
                        // Note: y0/y1 are swapped because Y is flipped
                        let x0 = pg.position.x + glyph_bounds.x0 * pg.scale;
                        let y0 = font_y_to_drawing(glyph_bounds.y1, pg.scale, pg.position.y);
                        let x1 = pg.position.x + glyph_bounds.x1 * pg.scale;
                        let y1 = font_y_to_drawing(glyph_bounds.y0, pg.scale, pg.position.y);

                        min_x = min_x.min(x0);
                        min_y = min_y.min(y0);
                        max_x = max_x.max(x1);
                        max_y = max_y.max(y1);
                    }
                }
            }

            if min_x < max_x && min_y < max_y {
                Some(Rect::new(min_x, min_y, max_x, max_y))
            } else {
                None
            }
        };

        TextLayout {
            font,
            glyphs,
            bounds,
            line_count,
        }
    }

    /// Measure text without rendering
    pub fn measure(&self, text: &str, font: FontRef, options: &TextOptions) -> Option<Rect> {
        self.layout(text, font, options).bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_options_builder() {
        let options = TextOptions::new(24.0)
            .at((100.0, 200.0))
            .align(TextAlign::Center)
            .letter_spacing(0.1);

        assert_eq!(options.size, 24.0);
        assert_eq!(options.position, Point::new(100.0, 200.0));
        assert_eq!(options.align, TextAlign::Center);
        assert_eq!(options.letter_spacing, 0.1);
    }

    #[test]
    fn test_font_y_to_drawing_coordinate_handedness() {
        // Font space: Y-up (positive Y = above baseline)
        // Drawing space: Y-down (positive Y = below baseline)
        let baseline_y = 100.0;
        let scale = 1.0;

        // A point above the baseline in font space (positive Y)
        // should be above the baseline in drawing space (smaller Y value)
        let font_y_above = 50.0; // 50 units above baseline in font space
        let drawing_y = font_y_to_drawing(font_y_above, scale, baseline_y);
        assert!(
            drawing_y < baseline_y,
            "Font Y above baseline ({}) should map to drawing Y above baseline (< {}), got {}",
            font_y_above,
            baseline_y,
            drawing_y
        );
        assert_eq!(drawing_y, 50.0); // baseline (100) - font_y (50) * scale (1) = 50

        // A point below the baseline in font space (negative Y)
        // should be below the baseline in drawing space (larger Y value)
        let font_y_below = -30.0; // 30 units below baseline in font space
        let drawing_y = font_y_to_drawing(font_y_below, scale, baseline_y);
        assert!(
            drawing_y > baseline_y,
            "Font Y below baseline ({}) should map to drawing Y below baseline (> {}), got {}",
            font_y_below,
            baseline_y,
            drawing_y
        );
        assert_eq!(drawing_y, 130.0); // baseline (100) - font_y (-30) * scale (1) = 130
    }

    #[test]
    fn test_font_y_to_drawing_with_scale() {
        let baseline_y = 200.0;
        let scale = 2.0; // Double size

        // 50 units above baseline, scaled 2x
        let drawing_y = font_y_to_drawing(50.0, scale, baseline_y);
        assert_eq!(drawing_y, 100.0); // 200 - 50 * 2 = 100

        // 25 units below baseline, scaled 2x
        let drawing_y = font_y_to_drawing(-25.0, scale, baseline_y);
        assert_eq!(drawing_y, 250.0); // 200 - (-25) * 2 = 250
    }

    #[test]
    fn test_font_y_to_drawing_baseline_at_origin() {
        // When baseline is at Y=0, ascenders should have negative Y in drawing space
        let baseline_y = 0.0;
        let scale = 1.0;

        let ascender_y = font_y_to_drawing(100.0, scale, baseline_y);
        assert_eq!(ascender_y, -100.0);

        let descender_y = font_y_to_drawing(-50.0, scale, baseline_y);
        assert_eq!(descender_y, 50.0);
    }
}
