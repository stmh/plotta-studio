//! Stroke - flattened output for rendering/plotting

use kurbo::Point;
use serde::{Deserialize, Serialize};

use crate::Style;

/// A flattened stroke - the final output for rendering/plotting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub points: Vec<Point>,
    pub style: Style,
    pub closed: bool,
}

impl Stroke {
    pub fn new(points: Vec<Point>, style: Style) -> Self {
        Self {
            points,
            style,
            closed: false,
        }
    }

    pub fn closed(mut self) -> Self {
        self.closed = true;
        self
    }

    pub fn line(from: Point, to: Point, style: Style) -> Self {
        Self::new(vec![from, to], style)
    }

    /// Total length of this stroke
    pub fn length(&self) -> f64 {
        self.points.windows(2).map(|w| w[0].distance(w[1])).sum()
    }

    /// Bounding box (min, max)
    pub fn bounds(&self) -> Option<(Point, Point)> {
        if self.points.is_empty() {
            return None;
        }
        let mut min = self.points[0];
        let mut max = self.points[0];
        for p in &self.points {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Some((min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_stroke_length() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(3.0, 4.0)],
            Style::default(),
        );
        assert!(approx_eq(stroke.length(), 5.0));
    }
}
