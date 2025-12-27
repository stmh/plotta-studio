//! Core types for plotta-studio
//!
//! This crate provides the fundamental types for creating and manipulating drawings:
//! - Primitives: Line, Circle, Rect, Path, etc. (using kurbo for geometry)
//! - Element: A shape with transform and style
//! - Group: Nested elements that transform together
//! - ClipGroup: Elements clipped to a closed shape
//! - Drawing: The top-level container

mod clip;
mod color;
mod context;
mod drawing;
mod element;
mod flatten;
mod font_registry;
pub mod font_types;
mod group;
mod path;
mod primitives;
mod shape;
mod stroke;
mod style;
mod text;

// Re-export kurbo types as our public API
pub use kurbo::{Affine, BezPath, Line, PathEl, Point, Rect, Vec2};

/// Type alias for transform (kurbo::Affine) for API clarity
pub type Transform = Affine;

// Re-export our types
pub use clip::ClipGroup;
pub use color::Color;
pub use context::RenderContext;
pub use drawing::Drawing;
pub use element::Element;
pub use font_registry::{FontRef, FontRegistry};
pub use group::Group;
pub use path::{Path, PathSegment};
pub use primitives::{Arc, Circle, Ellipse, Polyline, RegularPolygon};
pub use shape::Shape;
pub use stroke::Stroke;
pub use style::{ResolvedStyle, Style};
pub use text::Text;

// Re-export font types for convenience
pub use font_types::{
    Contour, ContourSegment, Font, FontMetrics, Glyph, PositionedGlyph, TextAlign, TextLayout,
    TextOptions, TextRenderer,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    fn point_approx_eq(a: Point, b: Point) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    #[test]
    fn test_point_distance() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);
        assert!(approx_eq(a.distance(b), 5.0));
    }

    #[test]
    fn test_affine_translate() {
        let t = Affine::translate(Vec2::new(10.0, 20.0));
        let p = Point::new(0.0, 0.0);
        let result = t * p;
        assert!(point_approx_eq(result, Point::new(10.0, 20.0)));
    }

    #[test]
    fn test_affine_rotate() {
        let t = Affine::rotate(FRAC_PI_2);
        let p = Point::new(1.0, 0.0);
        let result = t * p;
        assert!(point_approx_eq(result, Point::new(0.0, 1.0)));
    }

    #[test]
    fn test_affine_scale() {
        let t = Affine::scale(2.0);
        let p = Point::new(3.0, 4.0);
        let result = t * p;
        assert!(point_approx_eq(result, Point::new(6.0, 8.0)));
    }

    #[test]
    fn test_affine_composition() {
        let t1 = Affine::translate(Vec2::new(10.0, 0.0));
        let t2 = Affine::scale(2.0);
        let composed = t2 * t1; // scale after translate
        let p = Point::new(0.0, 0.0);
        let result = composed * p;
        assert!(point_approx_eq(result, Point::new(20.0, 0.0)));
    }

    #[test]
    fn test_affine_inverse() {
        let t = Affine::translate(Vec2::new(10.0, 20.0));
        let inv = t.inverse();
        let p = Point::new(10.0, 20.0);
        let result = inv * p;
        assert!(point_approx_eq(result, Point::new(0.0, 0.0)));
    }

    #[test]
    fn test_rect_creation() {
        let rect = Rect::from_origin_size((10.0, 20.0), (100.0, 50.0));
        assert!(approx_eq(rect.x0, 10.0));
        assert!(approx_eq(rect.y0, 20.0));
        assert!(approx_eq(rect.x1, 110.0));
        assert!(approx_eq(rect.y1, 70.0));
        assert!(approx_eq(rect.width(), 100.0));
        assert!(approx_eq(rect.height(), 50.0));
    }

    #[test]
    fn test_rect_centered() {
        let rect = Rect::from_center_size((50.0, 50.0), (100.0, 100.0));
        assert!(approx_eq(rect.x0, 0.0));
        assert!(approx_eq(rect.y0, 0.0));
        assert!(approx_eq(rect.x1, 100.0));
        assert!(approx_eq(rect.y1, 100.0));
    }
}
