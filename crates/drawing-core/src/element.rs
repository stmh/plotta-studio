//! Element - a scene graph node with shape, transform, and style

use crate::font_registry::FontRef;
use crate::font_types::TextOptions;
use kurbo::{Affine, Line, Point, Rect, Vec2};
use serde::{Deserialize, Serialize};

use crate::clip::ClipGroup;
use crate::context::RenderContext;
use crate::flatten::{
    flatten_arc, flatten_circle, flatten_ellipse, flatten_path, flatten_regular_polygon,
};
use crate::group::Group;
use crate::path::Path;
use crate::primitives::{Arc, Circle, Ellipse, Polyline, RegularPolygon};
use crate::shape::Shape;
use crate::stroke::Stroke;
use crate::style::ResolvedStyle;
use crate::text::Text;
use crate::{Color, Style};

/// An element in the scene graph - a shape with transform and style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub shape: Shape,
    pub transform: Affine,
    pub style: Style,
}

impl Element {
    pub fn new(shape: impl Into<Shape>) -> Self {
        Self {
            shape: shape.into(),
            transform: Affine::IDENTITY,
            style: Style::default(),
        }
    }

    // === Convenience constructors ===

    pub fn line(from: impl Into<Point>, to: impl Into<Point>) -> Self {
        Self::new(Line::new(from.into(), to.into()))
    }

    pub fn circle(center: impl Into<Point>, radius: f64) -> Self {
        Self::new(Circle::new(center, radius))
    }

    pub fn ellipse(center: impl Into<Point>, rx: f64, ry: f64) -> Self {
        Self::new(Ellipse::new(center, rx, ry))
    }

    pub fn rect(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self::new(Rect::from_origin_size((x, y), (w, h)))
    }

    pub fn rect_centered(center: impl Into<Point>, w: f64, h: f64) -> Self {
        Self::new(Rect::from_center_size(center.into(), (w, h)))
    }

    pub fn arc(center: impl Into<Point>, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        Self::new(Arc::new(center, radius, start_angle, end_angle))
    }

    pub fn polygon(center: impl Into<Point>, radius: f64, sides: usize) -> Self {
        Self::new(RegularPolygon::new(center, radius, sides))
    }

    pub fn polyline(points: Vec<Point>) -> Self {
        Self::new(Polyline::new(points))
    }

    pub fn polygon_from_points(points: Vec<Point>) -> Self {
        Self::new(Polyline::closed(points))
    }

    pub fn path(path: Path) -> Self {
        Self::new(path)
    }

    pub fn group(group: Group) -> Self {
        Self::new(group)
    }

    /// Create a clip group that clips children to a closed shape
    pub fn clip(clip_shape: Element) -> Self {
        Self::new(ClipGroup::new(clip_shape))
    }

    /// Create an element from an existing ClipGroup
    pub fn clip_group(clip_group: ClipGroup) -> Self {
        Self::new(clip_group)
    }

    /// Create a text element with a font reference
    pub fn text(text: impl Into<String>, font: FontRef) -> Self {
        Self::new(Text::new(text, font))
    }

    /// Create an element from a pre-flattened stroke
    /// The stroke's points become a polyline
    pub fn from_stroke(stroke: Stroke) -> Self {
        let polyline = if stroke.closed {
            Polyline::closed(stroke.points)
        } else {
            Polyline::new(stroke.points)
        };
        Self::new(polyline).with_style(stroke.style.into())
    }

    // === Transform builders ===

    /// Set the transform directly
    pub fn with_transform(mut self, transform: Affine) -> Self {
        self.transform = transform;
        self
    }

    /// Set the style directly
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn translate(mut self, x: f64, y: f64) -> Self {
        self.transform = self.transform.then_translate(Vec2::new(x, y));
        self
    }

    pub fn rotate(mut self, angle: f64) -> Self {
        self.transform = self.transform.then_rotate(angle);
        self
    }

    pub fn rotate_deg(self, degrees: f64) -> Self {
        self.rotate(degrees.to_radians())
    }

    pub fn rotate_around(mut self, angle: f64, center: impl Into<Point>) -> Self {
        self.transform = self.transform.then_rotate_about(angle, center.into());
        self
    }

    pub fn scale(mut self, sx: f64, sy: f64) -> Self {
        self.transform = self.transform.then_scale_non_uniform(sx, sy);
        self
    }

    pub fn scale_uniform(self, s: f64) -> Self {
        self.scale(s, s)
    }

    pub fn skew(mut self, sx: f64, sy: f64) -> Self {
        self.transform *= Affine::skew(sx.tan(), sy.tan());
        self
    }

    // === Style builders ===

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn stroke_width(mut self, w: f64) -> Self {
        self.style.stroke_width = Some(w);
        self
    }

    pub fn stroke_color(mut self, c: Color) -> Self {
        self.style.stroke_color = Some(c);
        self
    }

    /// Set whether stroke width should scale with transforms (default: true)
    ///
    /// When false, the stroke width remains constant regardless of element scaling.
    /// Useful for signatures, icons, or other elements that should maintain
    /// consistent line weight when scaled.
    pub fn scale_stroke(mut self, scale: bool) -> Self {
        self.style.scale_stroke = Some(scale);
        self
    }

    // === Text-specific builders ===

    /// Set text options (only applies to Text shapes)
    pub fn text_options(mut self, options: TextOptions) -> Self {
        if let Shape::Text(ref mut text) = self.shape {
            text.options = options;
        }
        self
    }

    /// Set text size (only applies to Text shapes)
    pub fn text_size(mut self, size: f64) -> Self {
        if let Shape::Text(ref mut text) = self.shape {
            text.options.size = size;
        }
        self
    }

    /// Enable/disable text debug visualization (only applies to Text shapes)
    pub fn text_debug(mut self, debug: bool) -> Self {
        if let Shape::Text(ref mut text) = self.shape {
            text.debug = debug;
        }
        self
    }

    // === ClipGroup-specific builders ===

    /// Add a child element to a ClipGroup (only applies to ClipGroup shapes)
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, element: Element) -> Self {
        if let Shape::ClipGroup(ref mut clip_group) = self.shape {
            clip_group.push(element);
        }
        self
    }

    /// Invert the clipping behavior (keep outside instead of inside)
    /// Only applies to ClipGroup shapes
    pub fn invert(mut self, invert: bool) -> Self {
        if let Shape::ClipGroup(ref mut clip_group) = self.shape {
            clip_group.invert = invert;
        }
        self
    }

    // === Flattening ===

    /// Flatten to strokes, applying transform
    pub fn flatten(&self, ctx: &RenderContext) -> Vec<Stroke> {
        self.flatten_with_inherited(ctx, Affine::IDENTITY, &ResolvedStyle::default())
    }

    pub(crate) fn flatten_with_inherited(
        &self,
        ctx: &RenderContext,
        parent_transform: Affine,
        parent_style: &ResolvedStyle,
    ) -> Vec<Stroke> {
        let transform = parent_transform * self.transform;

        // Resolve style by inheriting from parent
        let resolved_style = self.style.resolve(parent_style);

        // Scale stroke width based on transform's scale factor (if enabled)
        let scaled_style = if resolved_style.scale_stroke {
            // Extract approximate uniform scale from the transform matrix
            let coeffs = transform.as_coeffs();
            let scale_x = (coeffs[0] * coeffs[0] + coeffs[1] * coeffs[1]).sqrt();
            let scale_y = (coeffs[2] * coeffs[2] + coeffs[3] * coeffs[3]).sqrt();
            let scale_factor = (scale_x + scale_y) / 2.0; // Average scale

            ResolvedStyle {
                stroke_width: resolved_style.stroke_width * scale_factor,
                stroke_color: resolved_style.stroke_color,
                scale_stroke: resolved_style.scale_stroke,
            }
        } else {
            resolved_style
        };

        match &self.shape {
            Shape::Line(line) => {
                vec![Stroke::line(
                    transform * line.p0,
                    transform * line.p1,
                    scaled_style,
                )]
            }

            Shape::Polyline(poly) => {
                let points = poly.points.iter().map(|p| transform * *p).collect();
                vec![Stroke {
                    points,
                    style: scaled_style,
                    closed: poly.closed,
                }]
            }

            Shape::Circle(circle) => {
                let points = flatten_circle(circle, &transform, ctx.tolerance);
                vec![Stroke {
                    points,
                    style: scaled_style,
                    closed: true,
                }]
            }

            Shape::Ellipse(ellipse) => {
                let points = flatten_ellipse(ellipse, &transform, ctx.tolerance);
                vec![Stroke {
                    points,
                    style: scaled_style,
                    closed: true,
                }]
            }

            Shape::Rect(rect) => {
                let corners = [
                    Point::new(rect.x0, rect.y0),
                    Point::new(rect.x1, rect.y0),
                    Point::new(rect.x1, rect.y1),
                    Point::new(rect.x0, rect.y1),
                ];
                let points = corners.iter().map(|p| transform * *p).collect();
                vec![Stroke {
                    points,
                    style: scaled_style,
                    closed: true,
                }]
            }

            Shape::Arc(arc) => {
                let points = flatten_arc(arc, &transform, ctx.tolerance);
                vec![Stroke {
                    points,
                    style: scaled_style,
                    closed: false,
                }]
            }

            Shape::RegularPolygon(poly) => {
                let points = flatten_regular_polygon(poly, &transform);
                vec![Stroke {
                    points,
                    style: scaled_style,
                    closed: true,
                }]
            }

            Shape::Path(path) => flatten_path(path, &transform, scaled_style, ctx.tolerance),

            Shape::Group(group) => group
                .children
                .iter()
                .flat_map(|child| child.flatten_with_inherited(ctx, transform, &resolved_style))
                .collect(),

            Shape::ClipGroup(clip_group) => {
                clip_group.flatten_with_inherited(ctx, transform, &resolved_style)
            }

            Shape::Text(text) => text.flatten(ctx, transform, resolved_style),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FontRegistry;
    use std::sync::Arc;

    fn test_ctx() -> RenderContext {
        RenderContext::new(Arc::new(FontRegistry::new()))
    }

    #[test]
    fn test_element_flatten_line() {
        let ctx = test_ctx();
        let elem = Element::line((0.0, 0.0), (10.0, 10.0));
        let strokes = elem.flatten(&ctx);
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].points.len(), 2);
    }

    #[test]
    fn test_element_flatten_circle() {
        let ctx = test_ctx();
        let elem = Element::circle((0.0, 0.0), 10.0);
        let strokes = elem.flatten(&ctx);
        assert_eq!(strokes.len(), 1);
        assert!(strokes[0].closed);
        assert!(strokes[0].points.len() > 10); // Should have many points
    }

    // Note: Tests requiring actual fonts are in drawing-text or integration tests
}
