//! Text layout and rendering types

use crate::font_registry::FontRef;
use crate::stroke::Stroke;
use crate::style::ResolvedStyle;
use kurbo::{Point, Rect};
use serde::{Deserialize, Serialize};

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
                            // Scale and translate points
                            // Font glyphs use Y-up coordinates (positive = ascenders, negative = descenders)
                            // Our drawing system uses Y-down, so we negate Y during scaling
                            let scaled_points: Vec<Point> = contour
                                .flatten(tolerance)
                                .iter()
                                .map(|p| {
                                    Point::new(
                                        p.x * pg.scale + pg.position.x,
                                        -p.y * pg.scale + pg.position.y, // Negate Y to flip
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

        // Calculate fallback space width (roughly 1/3 of em)
        let fallback_space_width = metrics.units_per_em / 3.0;

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
                let space_width = font
                    .glyph(' ')
                    .map(|g| g.advance_width)
                    .unwrap_or(fallback_space_width);
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
}
