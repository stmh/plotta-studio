//! Primitive shapes with segment control

use kurbo::Point;
use serde::{Deserialize, Serialize};

/// Polyline (series of connected points)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Polyline {
    pub points: Vec<Point>,
    pub closed: bool,
}

impl Polyline {
    pub fn new(points: Vec<Point>) -> Self {
        Self {
            points,
            closed: false,
        }
    }

    pub fn closed(points: Vec<Point>) -> Self {
        Self {
            points,
            closed: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
    pub segments: usize,
}

impl Circle {
    pub fn new(center: impl Into<Point>, radius: f64) -> Self {
        Self {
            center: center.into(),
            radius,
            segments: 64,
        }
    }

    pub fn with_segments(mut self, segments: usize) -> Self {
        self.segments = segments;
        self
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            center: Point::ZERO,
            radius: 50.0,
            segments: 64,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Ellipse {
    pub center: Point,
    pub rx: f64,
    pub ry: f64,
    pub segments: usize,
}

impl Ellipse {
    pub fn new(center: impl Into<Point>, rx: f64, ry: f64) -> Self {
        Self {
            center: center.into(),
            rx,
            ry,
            segments: 64,
        }
    }

    pub fn with_segments(mut self, segments: usize) -> Self {
        self.segments = segments;
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Arc {
    pub center: Point,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub segments: usize,
}

impl Arc {
    pub fn new(center: impl Into<Point>, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        Self {
            center: center.into(),
            radius,
            start_angle,
            end_angle,
            segments: 32,
        }
    }

    pub fn with_segments(mut self, segments: usize) -> Self {
        self.segments = segments;
        self
    }
}

/// Regular polygon (triangle, pentagon, hexagon, etc.)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RegularPolygon {
    pub center: Point,
    pub radius: f64,
    pub sides: usize,
    pub rotation: f64,
}

impl RegularPolygon {
    pub fn new(center: impl Into<Point>, radius: f64, sides: usize) -> Self {
        Self {
            center: center.into(),
            radius,
            sides,
            rotation: 0.0,
        }
    }

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// Triangle
    pub fn triangle(center: impl Into<Point>, radius: f64) -> Self {
        Self::new(center, radius, 3)
    }

    /// Hexagon
    pub fn hexagon(center: impl Into<Point>, radius: f64) -> Self {
        Self::new(center, radius, 6)
    }
}
