//! Core types for font representation

use drawing_core::{Path, Point, Rect, Stroke, Style};
use kurbo::Shape;
use serde::{Deserialize, Serialize};

/// Font metrics for layout calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetrics {
    /// Units per em (typically 1000 or 2048)
    pub units_per_em: f64,
    /// Distance from baseline to top of tallest glyph
    pub ascender: f64,
    /// Distance from baseline to bottom of deepest glyph (negative)
    pub descender: f64,
    /// Height of lowercase 'x'
    pub x_height: Option<f64>,
    /// Height of capital letters
    pub cap_height: Option<f64>,
    /// Additional space between lines
    pub line_gap: f64,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            units_per_em: 1000.0,
            ascender: 800.0,
            descender: -200.0,
            x_height: Some(500.0),
            cap_height: Some(700.0),
            line_gap: 100.0,
        }
    }
}

impl FontMetrics {
    /// Calculate line height (ascender - descender + line_gap)
    pub fn line_height(&self) -> f64 {
        self.ascender - self.descender + self.line_gap
    }
}

/// A segment within a contour
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ContourSegment {
    /// Move to point (pen up)
    MoveTo(Point),
    /// Draw line to point
    LineTo(Point),
    /// Quadratic bezier curve
    QuadTo { ctrl: Point, to: Point },
    /// Cubic bezier curve
    CubicTo {
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
    },
}

/// A contour (open or closed path) within a glyph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contour {
    /// The segments making up this contour
    pub segments: Vec<ContourSegment>,
    /// Whether this contour is closed
    pub closed: bool,
}

impl Contour {
    /// Create a new empty contour
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            closed: false,
        }
    }

    /// Create an open contour from segments
    pub fn open(segments: Vec<ContourSegment>) -> Self {
        Self {
            segments,
            closed: false,
        }
    }

    /// Create a closed contour from segments
    pub fn closed(segments: Vec<ContourSegment>) -> Self {
        Self {
            segments,
            closed: true,
        }
    }

    /// Add a move_to segment
    pub fn move_to(mut self, p: impl Into<Point>) -> Self {
        self.segments.push(ContourSegment::MoveTo(p.into()));
        self
    }

    /// Add a line_to segment
    pub fn line_to(mut self, p: impl Into<Point>) -> Self {
        self.segments.push(ContourSegment::LineTo(p.into()));
        self
    }

    /// Add a quad_to segment
    pub fn quad_to(mut self, ctrl: impl Into<Point>, to: impl Into<Point>) -> Self {
        self.segments.push(ContourSegment::QuadTo {
            ctrl: ctrl.into(),
            to: to.into(),
        });
        self
    }

    /// Add a cubic_to segment
    pub fn cubic_to(
        mut self,
        ctrl1: impl Into<Point>,
        ctrl2: impl Into<Point>,
        to: impl Into<Point>,
    ) -> Self {
        self.segments.push(ContourSegment::CubicTo {
            ctrl1: ctrl1.into(),
            ctrl2: ctrl2.into(),
            to: to.into(),
        });
        self
    }

    /// Mark the contour as closed
    pub fn close(mut self) -> Self {
        self.closed = true;
        self
    }

    /// Convert to a drawing-core Path
    pub fn to_path(&self) -> Path {
        let mut path = Path::new();
        for seg in &self.segments {
            path = match seg {
                ContourSegment::MoveTo(p) => path.move_to(*p),
                ContourSegment::LineTo(p) => path.line_to(*p),
                ContourSegment::QuadTo { ctrl, to } => path.quad_to(*ctrl, *to),
                ContourSegment::CubicTo { ctrl1, ctrl2, to } => path.cubic_to(*ctrl1, *ctrl2, *to),
            };
        }
        if self.closed {
            path = path.close();
        }
        path
    }

    /// Convert to a drawing-core Stroke by flattening curves
    pub fn to_stroke(&self, style: Style, tolerance: f64) -> Stroke {
        let points = self.flatten(tolerance);
        let mut stroke = Stroke::new(points, style);
        if self.closed {
            stroke = stroke.closed();
        }
        stroke
    }

    /// Flatten curves to line segments
    pub fn flatten(&self, tolerance: f64) -> Vec<Point> {
        let bezpath = self.to_path().to_bezpath();
        let mut points = Vec::new();

        #[allow(deprecated)]
        bezpath.flatten(tolerance, |el| match el {
            kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => {
                points.push(p);
            }
            _ => {}
        });

        points
    }

    /// Calculate bounding box
    pub fn bounds(&self) -> Option<Rect> {
        let bezpath = self.to_path().to_bezpath();
        let bbox = bezpath.bounding_box();
        if bbox.width() == 0.0 && bbox.height() == 0.0 && bbox.x0 == 0.0 && bbox.y0 == 0.0 {
            None
        } else {
            Some(bbox)
        }
    }
}

impl Default for Contour {
    fn default() -> Self {
        Self::new()
    }
}

/// A single glyph from a font
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    /// The unicode character this glyph represents
    pub unicode: char,
    /// Optional glyph name
    pub name: Option<String>,
    /// Width to advance after drawing this glyph
    pub advance_width: f64,
    /// The contours making up this glyph
    pub contours: Vec<Contour>,
}

impl Glyph {
    /// Create a new glyph
    pub fn new(unicode: char, advance_width: f64) -> Self {
        Self {
            unicode,
            name: None,
            advance_width,
            contours: Vec::new(),
        }
    }

    /// Add a contour to the glyph
    pub fn with_contour(mut self, contour: Contour) -> Self {
        self.contours.push(contour);
        self
    }

    /// Set the glyph name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Convert all contours to paths
    pub fn to_paths(&self) -> Vec<Path> {
        self.contours.iter().map(|c| c.to_path()).collect()
    }

    /// Convert all contours to strokes
    pub fn to_strokes(&self, style: Style, tolerance: f64) -> Vec<Stroke> {
        self.contours
            .iter()
            .map(|c| c.to_stroke(style.clone(), tolerance))
            .collect()
    }

    /// Calculate bounding box
    pub fn bounds(&self) -> Option<Rect> {
        let mut result: Option<Rect> = None;
        for contour in &self.contours {
            if let Some(bbox) = contour.bounds() {
                result = Some(match result {
                    Some(r) => r.union(bbox),
                    None => bbox,
                });
            }
        }
        result
    }
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
}

/// A positioned glyph ready for rendering
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    /// The glyph
    pub glyph: Glyph,
    /// Position (baseline origin)
    pub position: Point,
    /// Scale factor (size / units_per_em)
    pub scale: f64,
}

impl PositionedGlyph {
    /// Convert to strokes at the positioned location
    pub fn to_strokes(&self, style: Style, tolerance: f64) -> Vec<Stroke> {
        self.glyph
            .contours
            .iter()
            .map(|contour| {
                // Scale and translate points
                // Hershey fonts use Y-down coordinates with baseline at Y=0
                // Ascenders go to negative Y, descenders to positive Y
                // We keep Y as-is since our drawing system is also Y-down
                let scaled_points: Vec<Point> = contour
                    .flatten(tolerance)
                    .iter()
                    .map(|p| {
                        Point::new(
                            p.x * self.scale + self.position.x,
                            p.y * self.scale + self.position.y,
                        )
                    })
                    .collect();

                let mut stroke = Stroke::new(scaled_points, style.clone());
                if contour.closed {
                    stroke = stroke.closed();
                }
                stroke
            })
            .collect()
    }
}

/// Result of text layout
#[derive(Debug, Clone)]
pub struct TextLayout {
    /// Positioned glyphs
    pub glyphs: Vec<PositionedGlyph>,
    /// Bounding box of the entire text
    pub bounds: Option<Rect>,
    /// Number of lines
    pub line_count: usize,
}

impl TextLayout {
    /// Convert entire layout to strokes
    pub fn to_strokes(&self, style: Style, tolerance: f64) -> Vec<Stroke> {
        self.glyphs
            .iter()
            .flat_map(|pg| pg.to_strokes(style.clone(), tolerance))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_core::PathSegment;

    #[test]
    fn test_font_metrics_line_height() {
        let metrics = FontMetrics::default();
        assert_eq!(metrics.line_height(), 1100.0); // 800 - (-200) + 100
    }

    #[test]
    fn test_contour_to_path() {
        let contour = Contour::new()
            .move_to((0.0, 0.0))
            .line_to((100.0, 0.0))
            .line_to((100.0, 100.0));

        let path = contour.to_path();
        assert_eq!(path.segments.len(), 3);
        assert!(matches!(path.segments[0], PathSegment::MoveTo(_)));
        assert!(matches!(path.segments[1], PathSegment::LineTo(_)));
    }

    #[test]
    fn test_glyph_creation() {
        let glyph = Glyph::new('A', 600.0).with_name("A").with_contour(
            Contour::new()
                .move_to((0.0, 0.0))
                .line_to((300.0, 700.0))
                .line_to((600.0, 0.0)),
        );

        assert_eq!(glyph.unicode, 'A');
        assert_eq!(glyph.advance_width, 600.0);
        assert_eq!(glyph.contours.len(), 1);
    }

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
