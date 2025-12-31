//! Flattening helpers for converting shapes to strokes
//!
//! Uses kurbo's tolerance-based flattening for adaptive curve subdivision.

use std::f64::consts::TAU;

use kurbo::{Affine, PathEl, Point, Shape};

use crate::path::Path;
use crate::primitives::{Arc, Circle, Ellipse, RegularPolygon};
use crate::simplify::cleanup_points;
use crate::stroke::Stroke;
use crate::style::ResolvedStyle;

/// Flatten a circle to points using tolerance-based subdivision.
pub fn flatten_circle(circle: &Circle, transform: &Affine, tolerance: f64) -> Vec<Point> {
    // Create kurbo Circle and get its BezPath representation
    let kurbo_circle = kurbo::Circle::new(circle.center, circle.radius);
    let bezpath = kurbo_circle.path_elements(tolerance);

    // Flatten with tolerance and apply transform
    let mut points = Vec::new();
    kurbo::flatten(bezpath, tolerance, |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => {
            points.push(*transform * p);
        }
        _ => {}
    });

    cleanup_points(points, tolerance)
}

/// Flatten an ellipse to points using tolerance-based subdivision.
pub fn flatten_ellipse(ellipse: &Ellipse, transform: &Affine, tolerance: f64) -> Vec<Point> {
    // Create kurbo Ellipse and get its BezPath representation
    let kurbo_ellipse = kurbo::Ellipse::new(ellipse.center, (ellipse.rx, ellipse.ry), 0.0);
    let bezpath = kurbo_ellipse.path_elements(tolerance);

    // Flatten with tolerance and apply transform
    let mut points = Vec::new();
    kurbo::flatten(bezpath, tolerance, |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => {
            points.push(*transform * p);
        }
        _ => {}
    });

    cleanup_points(points, tolerance)
}

/// Flatten an arc to points using tolerance-based subdivision.
pub fn flatten_arc(arc: &Arc, transform: &Affine, tolerance: f64) -> Vec<Point> {
    // Create kurbo Arc
    let sweep_angle = arc.end_angle - arc.start_angle;
    let kurbo_arc = kurbo::Arc::new(
        arc.center,
        (arc.radius, arc.radius), // radii (x, y)
        arc.start_angle,
        sweep_angle,
        0.0, // x_rotation
    );

    let bezpath = kurbo_arc.path_elements(tolerance);

    // Flatten with tolerance and apply transform
    let mut points = Vec::new();
    kurbo::flatten(bezpath, tolerance, |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => {
            points.push(*transform * p);
        }
        _ => {}
    });

    cleanup_points(points, tolerance)
}

/// Flatten a regular polygon to points.
///
/// Regular polygons don't need tolerance-based flattening since they
/// are defined by their exact vertex positions.
pub fn flatten_regular_polygon(poly: &RegularPolygon, transform: &Affine) -> Vec<Point> {
    (0..=poly.sides)
        .map(|i| {
            let t = i as f64 / poly.sides as f64;
            let angle = t * TAU + poly.rotation;
            let p = Point::new(
                poly.center.x + angle.cos() * poly.radius,
                poly.center.y + angle.sin() * poly.radius,
            );
            *transform * p
        })
        .collect()
}

/// Flatten a path to strokes using tolerance-based subdivision.
pub fn flatten_path(
    path: &Path,
    transform: &Affine,
    style: ResolvedStyle,
    tolerance: f64,
) -> Vec<Stroke> {
    let bezpath = path.to_bezpath();
    let mut strokes = Vec::new();
    let mut current_points: Vec<Point> = Vec::new();
    let mut start_point = Point::ZERO;
    let mut is_closed = false;

    // Use kurbo::flatten for adaptive subdivision
    kurbo::flatten(bezpath, tolerance, |el| match el {
        PathEl::MoveTo(p) => {
            if current_points.len() > 1 {
                let cleaned = cleanup_points(std::mem::take(&mut current_points), tolerance);
                if cleaned.len() > 1 {
                    let mut stroke = Stroke::new(cleaned, style);
                    stroke.closed = is_closed;
                    strokes.push(stroke);
                }
            } else {
                current_points.clear();
            }
            is_closed = false;
            start_point = p;
            current_points.push(*transform * p);
        }
        PathEl::LineTo(p) => {
            current_points.push(*transform * p);
        }
        PathEl::ClosePath => {
            current_points.push(*transform * start_point);
            is_closed = true;
        }
        // QuadTo and CurveTo are flattened to LineTo by kurbo::flatten
        _ => {}
    });

    if current_points.len() > 1 {
        let cleaned = cleanup_points(current_points, tolerance);
        if cleaned.len() > 1 {
            let mut stroke = Stroke::new(cleaned, style);
            stroke.closed = is_closed;
            strokes.push(stroke);
        }
    }

    strokes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_circle_produces_closed_path() {
        let circle = Circle {
            center: Point::new(0.0, 0.0),
            radius: 10.0,
        };
        let points = flatten_circle(&circle, &Affine::IDENTITY, 0.05);
        // Should have multiple points forming a closed circle
        assert!(points.len() > 4);
        // First and last point should be close (closed path)
        let first = points.first().unwrap();
        let last = points.last().unwrap();
        assert!(first.distance(*last) < 0.1);
    }

    #[test]
    fn test_flatten_circle_scales_with_tolerance() {
        let circle = Circle {
            center: Point::new(0.0, 0.0),
            radius: 100.0,
        };
        let fine = flatten_circle(&circle, &Affine::IDENTITY, 0.01);
        let coarse = flatten_circle(&circle, &Affine::IDENTITY, 1.0);
        // Finer tolerance should produce more points
        assert!(fine.len() > coarse.len());
    }

    #[test]
    fn test_flatten_ellipse_produces_closed_path() {
        let ellipse = Ellipse {
            center: Point::new(0.0, 0.0),
            rx: 20.0,
            ry: 10.0,
        };
        let points = flatten_ellipse(&ellipse, &Affine::IDENTITY, 0.05);
        assert!(points.len() > 4);
        // First and last point should be close (closed path)
        let first = points.first().unwrap();
        let last = points.last().unwrap();
        assert!(first.distance(*last) < 0.1);
    }

    #[test]
    fn test_flatten_arc() {
        let arc = Arc {
            center: Point::new(0.0, 0.0),
            radius: 10.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::PI,
        };
        let points = flatten_arc(&arc, &Affine::IDENTITY, 0.05);
        // Should have multiple points
        assert!(points.len() > 2);
        // Start point should be at (radius, 0)
        let first = points.first().unwrap();
        assert!((first.x - 10.0).abs() < 0.1);
        assert!(first.y.abs() < 0.1);
    }

    #[test]
    fn test_flatten_regular_polygon() {
        let hexagon = RegularPolygon {
            center: Point::new(0.0, 0.0),
            radius: 10.0,
            sides: 6,
            rotation: 0.0,
        };
        let points = flatten_regular_polygon(&hexagon, &Affine::IDENTITY);
        // Hexagon has 6 sides, so 7 points (first = last for closed path)
        assert_eq!(points.len(), 7);
    }

    #[test]
    fn test_flatten_path_with_bezier() {
        let path =
            Path::new()
                .move_to((0.0, 0.0))
                .cubic_to((10.0, 0.0), (20.0, 10.0), (20.0, 20.0));
        let strokes = flatten_path(&path, &Affine::IDENTITY, ResolvedStyle::default(), 0.05);
        assert_eq!(strokes.len(), 1);
        // Bezier should be flattened to multiple line segments
        assert!(strokes[0].points.len() > 2);
    }
}
