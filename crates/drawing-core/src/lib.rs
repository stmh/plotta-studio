//! Core types for plotta-studio
//!
//! This crate provides the fundamental types for creating and manipulating drawings:
//! - Primitives: Line, Circle, Rect, Path, etc. (using kurbo for geometry)
//! - Element: A shape with transform and style
//! - Group: Nested elements that transform together
//! - Drawing: The top-level container

use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

// Re-export kurbo types as our public API
pub use kurbo::{Affine, BezPath, Line, PathEl, Point, Rect, Vec2};

/// Type alias for transform (kurbo::Affine) for API clarity
pub type Transform = Affine;

// ============================================================================
// Color & Style
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn gray(v: u8) -> Self {
        Self::rgb(v, v, v)
    }

    /// Create from HSL (h: 0-360, s: 0-1, l: 0-1)
    pub fn hsl(h: f64, s: f64, l: f64) -> Self {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match h as u32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::rgb(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Style {
    pub stroke_width: f64,
    pub stroke_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            stroke_width: 1.0,
            stroke_color: Color::BLACK,
        }
    }
}

impl Style {
    pub fn new(width: f64, color: Color) -> Self {
        Self {
            stroke_width: width,
            stroke_color: color,
        }
    }

    pub fn width(mut self, w: f64) -> Self {
        self.stroke_width = w;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.stroke_color = c;
        self
    }
}

// ============================================================================
// Stroke (flattened output)
// ============================================================================

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

// ============================================================================
// Primitives (custom shapes with segment control)
// ============================================================================

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

// ============================================================================
// Shape enum
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape {
    Line(Line),
    Polyline(Polyline),
    Circle(Circle),
    Ellipse(Ellipse),
    Rect(Rect),
    Arc(Arc),
    RegularPolygon(RegularPolygon),
    Path(Path),
    Group(Group),
}

impl From<Line> for Shape {
    fn from(v: Line) -> Self {
        Shape::Line(v)
    }
}
impl From<Polyline> for Shape {
    fn from(v: Polyline) -> Self {
        Shape::Polyline(v)
    }
}
impl From<Circle> for Shape {
    fn from(v: Circle) -> Self {
        Shape::Circle(v)
    }
}
impl From<Ellipse> for Shape {
    fn from(v: Ellipse) -> Self {
        Shape::Ellipse(v)
    }
}
impl From<Rect> for Shape {
    fn from(v: Rect) -> Self {
        Shape::Rect(v)
    }
}
impl From<Arc> for Shape {
    fn from(v: Arc) -> Self {
        Shape::Arc(v)
    }
}
impl From<RegularPolygon> for Shape {
    fn from(v: RegularPolygon) -> Self {
        Shape::RegularPolygon(v)
    }
}
impl From<Path> for Shape {
    fn from(v: Path) -> Self {
        Shape::Path(v)
    }
}
impl From<Group> for Shape {
    fn from(v: Group) -> Self {
        Shape::Group(v)
    }
}

// ============================================================================
// Element (scene graph node)
// ============================================================================

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

    fn flatten_with_transform(&self, parent_transform: Affine) -> Vec<Stroke> {
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

// ============================================================================
// Group
// ============================================================================

/// A group of elements that transform together
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Group {
    pub children: Vec<Element>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn add(mut self, element: Element) -> Self {
        self.children.push(element);
        self
    }

    pub fn push(&mut self, element: Element) {
        self.children.push(element);
    }

    pub fn extend(&mut self, elements: impl IntoIterator<Item = Element>) {
        self.children.extend(elements);
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

// ============================================================================
// Flattening helpers
// ============================================================================

fn flatten_circle(circle: &Circle, transform: &Affine) -> Vec<Point> {
    (0..=circle.segments)
        .map(|i| {
            let t = i as f64 / circle.segments as f64;
            let angle = t * TAU;
            let p = Point::new(
                circle.center.x + angle.cos() * circle.radius,
                circle.center.y + angle.sin() * circle.radius,
            );
            *transform * p
        })
        .collect()
}

fn flatten_ellipse(ellipse: &Ellipse, transform: &Affine) -> Vec<Point> {
    (0..=ellipse.segments)
        .map(|i| {
            let t = i as f64 / ellipse.segments as f64;
            let angle = t * TAU;
            let p = Point::new(
                ellipse.center.x + angle.cos() * ellipse.rx,
                ellipse.center.y + angle.sin() * ellipse.ry,
            );
            *transform * p
        })
        .collect()
}

fn flatten_arc(arc: &Arc, transform: &Affine) -> Vec<Point> {
    let angle_span = arc.end_angle - arc.start_angle;
    (0..=arc.segments)
        .map(|i| {
            let t = i as f64 / arc.segments as f64;
            let angle = arc.start_angle + t * angle_span;
            let p = Point::new(
                arc.center.x + angle.cos() * arc.radius,
                arc.center.y + angle.sin() * arc.radius,
            );
            *transform * p
        })
        .collect()
}

fn flatten_regular_polygon(poly: &RegularPolygon, transform: &Affine) -> Vec<Point> {
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

fn flatten_path(path: &Path, transform: &Affine, style: Style) -> Vec<Stroke> {
    use kurbo::ParamCurve;

    let bezpath = path.to_bezpath();
    let mut strokes = Vec::new();
    let mut current_points: Vec<Point> = Vec::new();
    let mut last_point = Point::ZERO;
    let mut start_point = Point::ZERO;

    for el in bezpath.elements() {
        match el {
            PathEl::MoveTo(p) => {
                if current_points.len() > 1 {
                    strokes.push(Stroke::new(std::mem::take(&mut current_points), style));
                } else {
                    current_points.clear();
                }
                start_point = *p;
                last_point = *p;
                current_points.push(*transform * *p);
            }
            PathEl::LineTo(p) => {
                last_point = *p;
                current_points.push(*transform * *p);
            }
            PathEl::QuadTo(ctrl, to) => {
                let quad = kurbo::QuadBez::new(last_point, *ctrl, *to);
                let steps = 16;
                for i in 1..=steps {
                    let t = i as f64 / steps as f64;
                    let p = quad.eval(t);
                    current_points.push(*transform * p);
                }
                last_point = *to;
            }
            PathEl::CurveTo(ctrl1, ctrl2, to) => {
                let cubic = kurbo::CubicBez::new(last_point, *ctrl1, *ctrl2, *to);
                let steps = 24;
                for i in 1..=steps {
                    let t = i as f64 / steps as f64;
                    let p = cubic.eval(t);
                    current_points.push(*transform * p);
                }
                last_point = *to;
            }
            PathEl::ClosePath => {
                current_points.push(*transform * start_point);
            }
        }
    }

    if current_points.len() > 1 {
        strokes.push(Stroke::new(current_points, style));
    }

    strokes
}

// ============================================================================
// Drawing (top-level container)
// ============================================================================

/// The top-level drawing container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawing {
    pub width: f64,
    pub height: f64,
    pub elements: Vec<Element>,
    pub background: Color,
}

impl Drawing {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
            background: Color::WHITE,
        }
    }

    /// A4 landscape in mm (297 x 210)
    pub fn a4_landscape() -> Self {
        Self::new(297.0, 210.0)
    }

    /// A4 portrait in mm (210 x 297)
    pub fn a4_portrait() -> Self {
        Self::new(210.0, 297.0)
    }

    /// A3 landscape in mm (420 x 297)
    pub fn a3_landscape() -> Self {
        Self::new(420.0, 297.0)
    }

    /// A3 portrait in mm (297 x 420)
    pub fn a3_portrait() -> Self {
        Self::new(297.0, 420.0)
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn add(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn extend(&mut self, elements: impl IntoIterator<Item = Element>) {
        self.elements.extend(elements);
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn center(&self) -> Point {
        Point::new(self.width / 2.0, self.height / 2.0)
    }

    /// Flatten all elements to strokes for rendering/export
    pub fn flatten(&self) -> Vec<Stroke> {
        self.elements.iter().flat_map(|e| e.flatten()).collect()
    }

    /// Total number of strokes when flattened
    pub fn stroke_count(&self) -> usize {
        self.elements.iter().map(|e| e.flatten().len()).sum()
    }

    /// Save to JSON
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    /// Load from JSON
    pub fn load(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

// ============================================================================
// Tests
// ============================================================================

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
    fn test_color_hsl() {
        // Red at full saturation and mid lightness
        let c = Color::hsl(0.0, 1.0, 0.5);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);

        // Green
        let c = Color::hsl(120.0, 1.0, 0.5);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 0);

        // Blue
        let c = Color::hsl(240.0, 1.0, 0.5);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_stroke_length() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(3.0, 4.0)],
            Style::default(),
        );
        assert!(approx_eq(stroke.length(), 5.0));
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
