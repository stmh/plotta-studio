//! Glyph and contour types for font geometry

use crate::stroke::Stroke;
use crate::style::ResolvedStyle;
use crate::Path;
use kurbo::{Point, Rect, Shape};
use serde::{Deserialize, Serialize};

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
    pub fn to_stroke(&self, style: ResolvedStyle, tolerance: f64) -> Stroke {
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

        kurbo::flatten(&bezpath, tolerance, |el| match el {
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
    pub fn to_strokes(&self, style: ResolvedStyle, tolerance: f64) -> Vec<Stroke> {
        self.contours
            .iter()
            .map(|c| c.to_stroke(style, tolerance))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PathSegment;

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
}
