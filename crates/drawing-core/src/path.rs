//! Path using kurbo's BezPath wrapped for serde

use kurbo::{BezPath, Point};
use serde::{Deserialize, Serialize};

/// Path using kurbo's BezPath wrapped for serde
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

/// Path segment enum (matches kurbo::PathEl but serializable)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PathSegment {
    MoveTo(Point),
    LineTo(Point),
    QuadTo {
        ctrl: Point,
        to: Point,
    },
    CubicTo {
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
    },
    Close,
}

impl Path {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn move_to(mut self, p: impl Into<Point>) -> Self {
        self.segments.push(PathSegment::MoveTo(p.into()));
        self
    }

    pub fn line_to(mut self, p: impl Into<Point>) -> Self {
        self.segments.push(PathSegment::LineTo(p.into()));
        self
    }

    pub fn quad_to(mut self, ctrl: impl Into<Point>, to: impl Into<Point>) -> Self {
        self.segments.push(PathSegment::QuadTo {
            ctrl: ctrl.into(),
            to: to.into(),
        });
        self
    }

    pub fn cubic_to(
        mut self,
        ctrl1: impl Into<Point>,
        ctrl2: impl Into<Point>,
        to: impl Into<Point>,
    ) -> Self {
        self.segments.push(PathSegment::CubicTo {
            ctrl1: ctrl1.into(),
            ctrl2: ctrl2.into(),
            to: to.into(),
        });
        self
    }

    pub fn close(mut self) -> Self {
        self.segments.push(PathSegment::Close);
        self
    }

    /// Convert to kurbo BezPath
    pub fn to_bezpath(&self) -> BezPath {
        let mut path = BezPath::new();
        for seg in &self.segments {
            match seg {
                PathSegment::MoveTo(p) => path.move_to(*p),
                PathSegment::LineTo(p) => path.line_to(*p),
                PathSegment::QuadTo { ctrl, to } => path.quad_to(*ctrl, *to),
                PathSegment::CubicTo { ctrl1, ctrl2, to } => path.curve_to(*ctrl1, *ctrl2, *to),
                PathSegment::Close => path.close_path(),
            }
        }
        path
    }

    /// Check if path is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_bezpath() {
        let path = Path::new()
            .move_to((0.0, 0.0))
            .line_to((10.0, 0.0))
            .line_to((10.0, 10.0))
            .close();

        let bezpath = path.to_bezpath();
        assert_eq!(bezpath.elements().len(), 4);
    }
}
