//! Text shape for rendering text in the scenegraph
//!
//! The `Text` shape renders text using fonts registered in `RenderContext`.
//! It supports debug visualization showing baselines, bounding boxes, and
//! advance width markers.

use kurbo::Affine;
use serde::{Deserialize, Serialize};

use crate::context::RenderContext;
use crate::font_types::{Font, TextAlign, TextLayout, TextOptions, TextRenderer};
use crate::stroke::Stroke;
use crate::{Color, Point, Rect, Style};

/// Text shape for rendering single-line font text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    /// The text content to render
    pub text: String,
    /// Name of the font (looked up from RenderContext)
    pub font_name: String,
    /// Text layout options
    pub options: TextOptions,
    /// Whether to render debug visualization
    pub debug: bool,
}

impl Text {
    /// Create a new text shape
    pub fn new(text: impl Into<String>, font_name: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_name: font_name.into(),
            options: TextOptions::default(),
            debug: false,
        }
    }

    /// Set text options
    pub fn with_options(mut self, options: TextOptions) -> Self {
        self.options = options;
        self
    }

    /// Set font size
    pub fn size(mut self, size: f64) -> Self {
        self.options.size = size;
        self
    }

    /// Set text alignment
    pub fn align(mut self, align: TextAlign) -> Self {
        self.options.align = align;
        self
    }

    /// Set position
    pub fn at(mut self, position: impl Into<Point>) -> Self {
        self.options.position = position.into();
        self
    }

    /// Enable debug visualization
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Flatten text to strokes
    pub(crate) fn flatten(
        &self,
        ctx: &RenderContext,
        transform: Affine,
        style: Style,
    ) -> Vec<Stroke> {
        let font = match ctx.font(&self.font_name) {
            Some(f) => f,
            None => {
                log::warn!("Font '{}' not found in RenderContext", self.font_name);
                return Vec::new();
            }
        };

        let renderer = TextRenderer::new();
        let layout = renderer.layout(&self.text, font, &self.options);

        let mut strokes = Vec::new();

        // Debug geometry first (rendered underneath text)
        if self.debug {
            strokes.extend(self.render_debug(&layout, font, transform, style));
        }

        // Text strokes (rendered on top)
        let text_strokes = layout.to_strokes(style, 0.5);
        for stroke in text_strokes {
            // Apply transform to stroke points
            let transformed_points: Vec<Point> =
                stroke.points.iter().map(|p| transform * *p).collect();
            strokes.push(Stroke {
                points: transformed_points,
                style: stroke.style,
                closed: stroke.closed,
            });
        }

        strokes
    }

    /// Render debug visualization (baselines, bounding boxes, etc.)
    fn render_debug(
        &self,
        layout: &TextLayout,
        font: &dyn Font,
        transform: Affine,
        style: Style,
    ) -> Vec<Stroke> {
        let metrics = font.metrics();
        let mut strokes = Vec::new();

        // Colors for debug elements
        let baseline_color = Color::rgb(255, 0, 0); // Red for baseline
        let ascender_color = Color::rgb(0, 150, 0); // Green for ascender
        let descender_color = Color::rgb(0, 0, 255); // Blue for descender
        let bbox_color = Color::rgb(255, 128, 0); // Orange for glyph bounding boxes
        let advance_color = Color::rgb(128, 0, 255); // Purple for advance width markers

        let debug_stroke_width = style.stroke_width * 0.3;

        // Track lines we've already drawn baselines for (to avoid duplicates)
        let mut drawn_baselines: std::collections::HashSet<i32> = std::collections::HashSet::new();

        for positioned_glyph in &layout.glyphs {
            let pos = positioned_glyph.position;
            let scale = positioned_glyph.scale;
            let glyph = &positioned_glyph.glyph;

            // Calculate metric lines (scaled to drawing units)
            let baseline_y = pos.y;
            let ascender_y = pos.y - metrics.ascender * scale;
            let descender_y = pos.y - metrics.descender * scale;

            // Use baseline_y as key to check if we've drawn this line
            let line_key = (baseline_y * 100.0) as i32;

            // Draw baseline, ascender, descender lines (once per text line)
            if !drawn_baselines.contains(&line_key) {
                drawn_baselines.insert(line_key);

                // Find extent of this line
                let line_start_x = if let Some(bounds) = &layout.bounds {
                    bounds.x0 - 5.0
                } else {
                    pos.x - 50.0
                };
                let line_end_x = if let Some(bounds) = &layout.bounds {
                    bounds.x1 + 5.0
                } else {
                    pos.x + 200.0
                };

                // Baseline (red)
                strokes.push(
                    self.make_line(
                        (line_start_x, baseline_y),
                        (line_end_x, baseline_y),
                        transform,
                        Style::default()
                            .with_stroke_width(debug_stroke_width)
                            .with_stroke_color(baseline_color),
                    ),
                );

                // Ascender line (green)
                strokes.push(
                    self.make_line(
                        (line_start_x, ascender_y),
                        (line_end_x, ascender_y),
                        transform,
                        Style::default()
                            .with_stroke_width(debug_stroke_width)
                            .with_stroke_color(ascender_color),
                    ),
                );

                // Descender line (blue)
                strokes.push(
                    self.make_line(
                        (line_start_x, descender_y),
                        (line_end_x, descender_y),
                        transform,
                        Style::default()
                            .with_stroke_width(debug_stroke_width)
                            .with_stroke_color(descender_color),
                    ),
                );
            }

            // Draw glyph bounding box (orange)
            if let Some(glyph_bounds) = glyph.bounds() {
                let x = glyph_bounds.x0 * scale + pos.x;
                let y = -glyph_bounds.y1 * scale + pos.y;
                let w = glyph_bounds.width() * scale;
                let h = glyph_bounds.height() * scale;

                strokes.push(
                    self.make_rect(
                        Rect::from_origin_size((x, y), (w, h)),
                        transform,
                        Style::default()
                            .with_stroke_width(debug_stroke_width)
                            .with_stroke_color(bbox_color),
                    ),
                );
            }

            // Draw advance width marker (purple vertical line)
            let advance_x = pos.x + glyph.advance_width * scale;
            strokes.push(
                self.make_line(
                    (advance_x, ascender_y),
                    (advance_x, descender_y),
                    transform,
                    Style::default()
                        .with_stroke_width(debug_stroke_width * 0.5)
                        .with_stroke_color(advance_color),
                ),
            );
        }

        strokes
    }

    /// Helper to create a line stroke
    fn make_line(
        &self,
        from: impl Into<Point>,
        to: impl Into<Point>,
        transform: Affine,
        style: Style,
    ) -> Stroke {
        let p1 = transform * from.into();
        let p2 = transform * to.into();
        Stroke {
            points: vec![p1, p2],
            style,
            closed: false,
        }
    }

    /// Helper to create a rectangle stroke
    fn make_rect(&self, rect: Rect, transform: Affine, style: Style) -> Stroke {
        let corners = [
            Point::new(rect.x0, rect.y0),
            Point::new(rect.x1, rect.y0),
            Point::new(rect.x1, rect.y1),
            Point::new(rect.x0, rect.y1),
        ];
        let points = corners.iter().map(|p| transform * *p).collect();
        Stroke {
            points,
            style,
            closed: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_creation() {
        let text = Text::new("Hello", "Hershey Simplex");
        assert_eq!(text.text, "Hello");
        assert_eq!(text.font_name, "Hershey Simplex");
        assert!(!text.debug);
    }

    #[test]
    fn test_text_builder() {
        let text = Text::new("Hello", "Hershey Simplex")
            .size(24.0)
            .align(TextAlign::Center)
            .at((100.0, 200.0))
            .with_debug(true);

        assert_eq!(text.options.size, 24.0);
        assert_eq!(text.options.align, TextAlign::Center);
        assert_eq!(text.options.position, Point::new(100.0, 200.0));
        assert!(text.debug);
    }

    #[test]
    fn test_text_flatten_missing_font() {
        let text = Text::new("Hello", "NonExistent");
        let ctx = RenderContext::new();
        let strokes = text.flatten(&ctx, Affine::IDENTITY, Style::default());
        assert!(strokes.is_empty());
    }

    // Note: Tests requiring actual fonts are in drawing-text or integration tests
}
