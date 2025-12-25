//! Element - a scene graph node with shape, transform, and style

use kurbo::{Affine, Line, Point, Rect, Vec2};
use serde::{Deserialize, Serialize};

use crate::flatten::{
    flatten_arc, flatten_circle, flatten_ellipse, flatten_path, flatten_regular_polygon,
};
use crate::group::Group;
use crate::path::Path;
use crate::primitives::{Arc, Circle, Ellipse, Polyline, RegularPolygon};
use crate::shape::Shape;
use crate::stroke::Stroke;
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

    // === Transform builders ===

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
        self.style.stroke_width = w;
        self
    }

    pub fn stroke_color(mut self, c: Color) -> Self {
        self.style.stroke_color = c;
        self
    }

    // === Flattening ===

    /// Flatten to strokes, applying transform
    pub fn flatten(&self) -> Vec<Stroke> {
        self.flatten_with_transform(Affine::IDENTITY)
    }

    pub(crate) fn flatten_with_transform(&self, parent_transform: Affine) -> Vec<Stroke> {
        let transform = parent_transform * self.transform;

        match &self.shape {
            Shape::Line(line) => {
                vec![Stroke::line(
                    transform * line.p0,
                    transform * line.p1,
                    self.style,
                )]
            }

            Shape::Polyline(poly) => {
                let points = poly.points.iter().map(|p| transform * *p).collect();
                vec![Stroke {
                    points,
                    style: self.style,
                    closed: poly.closed,
                }]
            }

            Shape::Circle(circle) => {
                let points = flatten_circle(circle, &transform);
                vec![Stroke {
                    points,
                    style: self.style,
                    closed: true,
                }]
            }

            Shape::Ellipse(ellipse) => {
                let points = flatten_ellipse(ellipse, &transform);
                vec![Stroke {
                    points,
                    style: self.style,
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
                    style: self.style,
                    closed: true,
                }]
            }

            Shape::Arc(arc) => {
                let points = flatten_arc(arc, &transform);
                vec![Stroke {
                    points,
                    style: self.style,
                    closed: false,
                }]
            }

            Shape::RegularPolygon(poly) => {
                let points = flatten_regular_polygon(poly, &transform);
                vec![Stroke {
                    points,
                    style: self.style,
                    closed: true,
                }]
            }

            Shape::Path(path) => flatten_path(path, &transform, self.style),

            Shape::Group(group) => group
                .children
                .iter()
                .flat_map(|child| child.flatten_with_transform(transform))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_flatten_line() {
        let elem = Element::line((0.0, 0.0), (10.0, 10.0));
        let strokes = elem.flatten();
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].points.len(), 2);
    }

    #[test]
    fn test_element_flatten_circle() {
        let elem = Element::circle((0.0, 0.0), 10.0);
        let strokes = elem.flatten();
        assert_eq!(strokes.len(), 1);
        assert!(strokes[0].closed);
        assert!(strokes[0].points.len() > 10); // Should have many points
    }
}
