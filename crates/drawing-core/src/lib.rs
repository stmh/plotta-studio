//! Core types for plotta-studio
//!
//! This crate provides the fundamental types for creating and manipulating drawings:
//! - Primitives: Line, Circle, Rect, Path, etc.
//! - Transform: 2D affine transformations
//! - Element: A shape with transform and style
//! - Group: Nested elements that transform together
//! - Drawing: The top-level container

use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

// ============================================================================
// Point
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::ZERO
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }

    pub fn lerp(&self, other: Point, t: f64) -> Point {
        Point::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    pub fn dot(&self, other: Point) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    pub fn from_angle(angle: f64) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    pub fn rotate(&self, angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }
}

impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::Div<f64> for Point {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Point::new(self.x / rhs, self.y / rhs)
    }
}

impl std::ops::Neg for Point {
    type Output = Self;
    fn neg(self) -> Self {
        Point::new(-self.x, -self.y)
    }
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Self::new(x, y)
    }
}

impl From<[f64; 2]> for Point {
    fn from([x, y]: [f64; 2]) -> Self {
        Self::new(x, y)
    }
}

// ============================================================================
// Transform (2D affine)
// ============================================================================

/// 2D affine transformation matrix
///
/// Represents the matrix:
/// ```text
/// | a  b  tx |
/// | c  d  ty |
/// | 0  0  1  |
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub fn translate(x: f64, y: f64) -> Self {
        Self {
            tx: x,
            ty: y,
            ..Self::IDENTITY
        }
    }

    pub fn rotate(angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            a: cos,
            b: -sin,
            c: sin,
            d: cos,
            tx: 0.0,
            ty: 0.0,
        }
    }

    pub fn rotate_deg(degrees: f64) -> Self {
        Self::rotate(degrees.to_radians())
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            tx: 0.0,
            ty: 0.0,
        }
    }

    pub fn scale_uniform(s: f64) -> Self {
        Self::scale(s, s)
    }

    /// Rotate around a specific point
    pub fn rotate_around(angle: f64, center: Point) -> Self {
        Self::translate(center.x, center.y)
            .then(Self::rotate(angle))
            .then(Self::translate(-center.x, -center.y))
    }

    /// Skew/shear transformation
    pub fn skew(sx: f64, sy: f64) -> Self {
        Self {
            a: 1.0,
            b: sx.tan(),
            c: sy.tan(),
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Combine transforms: self then other
    pub fn then(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            tx: self.tx * other.a + self.ty * other.c + other.tx,
            ty: self.tx * other.b + self.ty * other.d + other.ty,
        }
    }

    /// Apply transform to a point
    pub fn apply(&self, p: Point) -> Point {
        Point::new(
            self.a * p.x + self.b * p.y + self.tx,
            self.c * p.x + self.d * p.y + self.ty,
        )
    }

    /// Apply transform to multiple points
    pub fn apply_all(&self, points: &[Point]) -> Vec<Point> {
        points.iter().map(|p| self.apply(*p)).collect()
    }

    /// Get the inverse transform (if it exists)
    pub fn inverse(&self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-10 {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            tx: (self.b * self.ty - self.d * self.tx) * inv_det,
            ty: (self.c * self.tx - self.a * self.ty) * inv_det,
        })
    }
}

impl std::ops::Mul for Transform {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.then(rhs)
    }
}

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
        self.points
            .windows(2)
            .map(|w| w[0].distance(w[1]))
            .sum()
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
// Path Segment (for complex paths)
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PathSegment {
    MoveTo(Point),
    LineTo(Point),
    QuadTo { ctrl: Point, to: Point },
    CubicTo { ctrl1: Point, ctrl2: Point, to: Point },
    Close,
}

// ============================================================================
// Primitives
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Line {
    pub from: Point,
    pub to: Point,
}

impl Line {
    pub fn new(from: impl Into<Point>, to: impl Into<Point>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

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
pub struct Rect {
    pub origin: Point,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            origin: Point::new(x, y),
            width: w,
            height: h,
        }
    }

    pub fn from_origin(origin: impl Into<Point>, width: f64, height: f64) -> Self {
        Self {
            origin: origin.into(),
            width,
            height,
        }
    }

    pub fn centered(center: impl Into<Point>, w: f64, h: f64) -> Self {
        let center = center.into();
        Self {
            origin: Point::new(center.x - w / 2.0, center.y - h / 2.0),
            width: w,
            height: h,
        }
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.width / 2.0,
            self.origin.y + self.height / 2.0,
        )
    }

    pub fn corners(&self) -> [Point; 4] {
        [
            self.origin,
            Point::new(self.origin.x + self.width, self.origin.y),
            Point::new(self.origin.x + self.width, self.origin.y + self.height),
            Point::new(self.origin.x, self.origin.y + self.height),
        ]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
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
    pub transform: Transform,
    pub style: Style,
}

impl Element {
    pub fn new(shape: impl Into<Shape>) -> Self {
        Self {
            shape: shape.into(),
            transform: Transform::IDENTITY,
            style: Style::default(),
        }
    }

    // === Convenience constructors ===

    pub fn line(from: impl Into<Point>, to: impl Into<Point>) -> Self {
        Self::new(Line::new(from, to))
    }

    pub fn circle(center: impl Into<Point>, radius: f64) -> Self {
        Self::new(Circle::new(center, radius))
    }

    pub fn ellipse(center: impl Into<Point>, rx: f64, ry: f64) -> Self {
        Self::new(Ellipse::new(center, rx, ry))
    }

    pub fn rect(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self::new(Rect::new(x, y, w, h))
    }

    pub fn rect_centered(center: impl Into<Point>, w: f64, h: f64) -> Self {
        Self::new(Rect::centered(center, w, h))
    }

    pub fn arc(
        center: impl Into<Point>,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Self {
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
        self.transform = self.transform.then(Transform::translate(x, y));
        self
    }

    pub fn rotate(mut self, angle: f64) -> Self {
        self.transform = self.transform.then(Transform::rotate(angle));
        self
    }

    pub fn rotate_deg(self, degrees: f64) -> Self {
        self.rotate(degrees.to_radians())
    }

    pub fn rotate_around(mut self, angle: f64, center: impl Into<Point>) -> Self {
        self.transform = self
            .transform
            .then(Transform::rotate_around(angle, center.into()));
        self
    }

    pub fn scale(mut self, sx: f64, sy: f64) -> Self {
        self.transform = self.transform.then(Transform::scale(sx, sy));
        self
    }

    pub fn scale_uniform(self, s: f64) -> Self {
        self.scale(s, s)
    }

    pub fn skew(mut self, sx: f64, sy: f64) -> Self {
        self.transform = self.transform.then(Transform::skew(sx, sy));
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
        self.flatten_with_transform(Transform::IDENTITY)
    }

    fn flatten_with_transform(&self, parent_transform: Transform) -> Vec<Stroke> {
        let transform = parent_transform.then(self.transform);

        match &self.shape {
            Shape::Line(line) => {
                vec![Stroke::line(
                    transform.apply(line.from),
                    transform.apply(line.to),
                    self.style,
                )]
            }

            Shape::Polyline(poly) => {
                let points = transform.apply_all(&poly.points);
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
                let points = transform.apply_all(&rect.corners());
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

fn flatten_circle(circle: &Circle, transform: &Transform) -> Vec<Point> {
    (0..=circle.segments)
        .map(|i| {
            let t = i as f64 / circle.segments as f64;
            let angle = t * TAU;
            let p = Point::new(
                circle.center.x + angle.cos() * circle.radius,
                circle.center.y + angle.sin() * circle.radius,
            );
            transform.apply(p)
        })
        .collect()
}

fn flatten_ellipse(ellipse: &Ellipse, transform: &Transform) -> Vec<Point> {
    (0..=ellipse.segments)
        .map(|i| {
            let t = i as f64 / ellipse.segments as f64;
            let angle = t * TAU;
            let p = Point::new(
                ellipse.center.x + angle.cos() * ellipse.rx,
                ellipse.center.y + angle.sin() * ellipse.ry,
            );
            transform.apply(p)
        })
        .collect()
}

fn flatten_arc(arc: &Arc, transform: &Transform) -> Vec<Point> {
    let angle_span = arc.end_angle - arc.start_angle;
    (0..=arc.segments)
        .map(|i| {
            let t = i as f64 / arc.segments as f64;
            let angle = arc.start_angle + t * angle_span;
            let p = Point::new(
                arc.center.x + angle.cos() * arc.radius,
                arc.center.y + angle.sin() * arc.radius,
            );
            transform.apply(p)
        })
        .collect()
}

fn flatten_regular_polygon(poly: &RegularPolygon, transform: &Transform) -> Vec<Point> {
    (0..=poly.sides)
        .map(|i| {
            let t = i as f64 / poly.sides as f64;
            let angle = t * TAU + poly.rotation;
            let p = Point::new(
                poly.center.x + angle.cos() * poly.radius,
                poly.center.y + angle.sin() * poly.radius,
            );
            transform.apply(p)
        })
        .collect()
}

fn flatten_path(path: &Path, transform: &Transform, style: Style) -> Vec<Stroke> {
    let mut strokes = Vec::new();
    let mut current_points: Vec<Point> = Vec::new();
    let mut start_point = Point::ZERO;

    for seg in &path.segments {
        match seg {
            PathSegment::MoveTo(p) => {
                if current_points.len() > 1 {
                    strokes.push(Stroke::new(std::mem::take(&mut current_points), style));
                } else {
                    current_points.clear();
                }
                start_point = *p;
                current_points.push(transform.apply(*p));
            }
            PathSegment::LineTo(p) => {
                current_points.push(transform.apply(*p));
            }
            PathSegment::QuadTo { ctrl, to } => {
                let last = current_points.last().copied().unwrap_or(Point::ZERO);
                let ctrl_t = transform.apply(*ctrl);
                let to_t = transform.apply(*to);
                let steps = 16;
                for i in 1..=steps {
                    let t = i as f64 / steps as f64;
                    let p = quad_bezier(last, ctrl_t, to_t, t);
                    current_points.push(p);
                }
            }
            PathSegment::CubicTo { ctrl1, ctrl2, to } => {
                let last = current_points.last().copied().unwrap_or(Point::ZERO);
                let ctrl1_t = transform.apply(*ctrl1);
                let ctrl2_t = transform.apply(*ctrl2);
                let to_t = transform.apply(*to);
                let steps = 24;
                for i in 1..=steps {
                    let t = i as f64 / steps as f64;
                    let p = cubic_bezier(last, ctrl1_t, ctrl2_t, to_t, t);
                    current_points.push(p);
                }
            }
            PathSegment::Close => {
                current_points.push(transform.apply(start_point));
            }
        }
    }

    if current_points.len() > 1 {
        strokes.push(Stroke::new(current_points, style));
    }

    strokes
}

fn quad_bezier(p0: Point, p1: Point, p2: Point, t: f64) -> Point {
    let mt = 1.0 - t;
    Point::new(
        mt * mt * p0.x + 2.0 * mt * t * p1.x + t * t * p2.x,
        mt * mt * p0.y + 2.0 * mt * t * p1.y + t * t * p2.y,
    )
}

fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;
    Point::new(
        mt2 * mt * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t2 * t * p3.x,
        mt2 * mt * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t2 * t * p3.y,
    )
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

    #[test]
    fn test_point_ops() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(3.0, 4.0);
        assert_eq!(a + b, Point::new(4.0, 6.0));
        assert_eq!(b - a, Point::new(2.0, 2.0));
        assert_eq!(a * 2.0, Point::new(2.0, 4.0));
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform::IDENTITY;
        let p = Point::new(10.0, 20.0);
        assert_eq!(t.apply(p), p);
    }

    #[test]
    fn test_transform_translate() {
        let t = Transform::translate(5.0, 10.0);
        let p = Point::new(1.0, 2.0);
        assert_eq!(t.apply(p), Point::new(6.0, 12.0));
    }

    #[test]
    fn test_transform_chain() {
        let t = Transform::translate(10.0, 0.0).then(Transform::scale(2.0, 2.0));
        let p = Point::new(0.0, 0.0);
        assert_eq!(t.apply(p), Point::new(20.0, 0.0));
    }

    #[test]
    fn test_circle_flatten() {
        let circle = Circle::new(Point::ZERO, 10.0).with_segments(4);
        let element = Element::new(circle);
        let strokes = element.flatten();
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].points.len(), 5); // 4 segments + 1 to close
    }

    #[test]
    fn test_group_transform() {
        let mut group = Group::new();
        group.push(Element::circle(Point::ZERO, 10.0));
        group.push(Element::rect_centered(Point::ZERO, 20.0, 20.0));

        let element = Element::group(group).translate(100.0, 100.0);
        let strokes = element.flatten();

        // Both shapes should be translated
        for stroke in &strokes {
            for point in &stroke.points {
                assert!(point.x >= 80.0 && point.x <= 120.0);
                assert!(point.y >= 80.0 && point.y <= 120.0);
            }
        }
    }
}
