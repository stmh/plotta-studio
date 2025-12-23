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
        // 1. Translate so center is at origin
        // 2. Rotate
        // 3. Translate back
        Self::translate(-center.x, -center.y)
            .then(Self::rotate(angle))
            .then(Self::translate(center.x, center.y))
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
    /// Returns a transform that first applies self, then applies other.
    /// In matrix terms: other * self (since transforms apply right-to-left)
    pub fn then(self, other: Self) -> Self {
        Self {
            a: other.a * self.a + other.b * self.c,
            b: other.a * self.b + other.b * self.d,
            c: other.c * self.a + other.d * self.c,
            d: other.c * self.b + other.d * self.d,
            tx: other.a * self.tx + other.b * self.ty + other.tx,
            ty: other.c * self.tx + other.d * self.ty + other.ty,
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
// Path Segment (for complex paths)
// ============================================================================

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
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    fn point_approx_eq(a: Point, b: Point) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    // ========================================================================
    // Point::distance tests
    // ========================================================================

    #[test]
    fn test_point_distance_basic() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);
        assert!(approx_eq(a.distance(b), 5.0)); // 3-4-5 triangle
    }

    #[test]
    fn test_point_distance_same_point() {
        let a = Point::new(5.0, 10.0);
        assert!(approx_eq(a.distance(a), 0.0));
    }

    #[test]
    fn test_point_distance_negative_coords() {
        let a = Point::new(-3.0, -4.0);
        let b = Point::new(0.0, 0.0);
        assert!(approx_eq(a.distance(b), 5.0));
    }

    #[test]
    fn test_point_distance_symmetric() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(4.0, 6.0);
        assert!(approx_eq(a.distance(b), b.distance(a)));
    }

    // ========================================================================
    // Point::length tests
    // ========================================================================

    #[test]
    fn test_point_length_unit_vectors() {
        assert!(approx_eq(Point::new(1.0, 0.0).length(), 1.0));
        assert!(approx_eq(Point::new(0.0, 1.0).length(), 1.0));
        assert!(approx_eq(Point::new(-1.0, 0.0).length(), 1.0));
    }

    #[test]
    fn test_point_length_zero() {
        assert!(approx_eq(Point::ZERO.length(), 0.0));
    }

    #[test]
    fn test_point_length_pythagorean() {
        assert!(approx_eq(Point::new(3.0, 4.0).length(), 5.0));
    }

    // ========================================================================
    // Point::normalize tests
    // ========================================================================

    #[test]
    fn test_point_normalize_unit_length() {
        let p = Point::new(3.0, 4.0).normalize();
        assert!(approx_eq(p.length(), 1.0));
    }

    #[test]
    fn test_point_normalize_direction_preserved() {
        let p = Point::new(10.0, 0.0).normalize();
        assert!(point_approx_eq(p, Point::new(1.0, 0.0)));

        let p = Point::new(0.0, -5.0).normalize();
        assert!(point_approx_eq(p, Point::new(0.0, -1.0)));
    }

    #[test]
    fn test_point_normalize_zero_vector() {
        let p = Point::ZERO.normalize();
        assert!(point_approx_eq(p, Point::ZERO));
    }

    #[test]
    fn test_point_normalize_already_unit() {
        let p = Point::new(1.0, 0.0).normalize();
        assert!(point_approx_eq(p, Point::new(1.0, 0.0)));
    }

    // ========================================================================
    // Point::lerp tests
    // ========================================================================

    #[test]
    fn test_point_lerp_endpoints() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 20.0);
        assert!(point_approx_eq(a.lerp(b, 0.0), a));
        assert!(point_approx_eq(a.lerp(b, 1.0), b));
    }

    #[test]
    fn test_point_lerp_midpoint() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 20.0);
        assert!(point_approx_eq(a.lerp(b, 0.5), Point::new(5.0, 10.0)));
    }

    #[test]
    fn test_point_lerp_quarter() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(100.0, 100.0);
        assert!(point_approx_eq(a.lerp(b, 0.25), Point::new(25.0, 25.0)));
    }

    #[test]
    fn test_point_lerp_extrapolate() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 10.0);
        // t > 1 extrapolates beyond b
        assert!(point_approx_eq(a.lerp(b, 2.0), Point::new(20.0, 20.0)));
        // t < 0 extrapolates before a
        assert!(point_approx_eq(a.lerp(b, -1.0), Point::new(-10.0, -10.0)));
    }

    // ========================================================================
    // Point::dot tests
    // ========================================================================

    #[test]
    fn test_point_dot_parallel() {
        let a = Point::new(1.0, 0.0);
        let b = Point::new(5.0, 0.0);
        assert!(approx_eq(a.dot(b), 5.0));
    }

    #[test]
    fn test_point_dot_perpendicular() {
        let a = Point::new(1.0, 0.0);
        let b = Point::new(0.0, 1.0);
        assert!(approx_eq(a.dot(b), 0.0));
    }

    #[test]
    fn test_point_dot_opposite() {
        let a = Point::new(1.0, 0.0);
        let b = Point::new(-1.0, 0.0);
        assert!(approx_eq(a.dot(b), -1.0));
    }

    #[test]
    fn test_point_dot_general() {
        let a = Point::new(2.0, 3.0);
        let b = Point::new(4.0, 5.0);
        // 2*4 + 3*5 = 8 + 15 = 23
        assert!(approx_eq(a.dot(b), 23.0));
    }

    #[test]
    fn test_point_dot_commutative() {
        let a = Point::new(2.0, 3.0);
        let b = Point::new(4.0, 5.0);
        assert!(approx_eq(a.dot(b), b.dot(a)));
    }

    // ========================================================================
    // Point::angle tests
    // ========================================================================

    #[test]
    fn test_point_angle_zero() {
        let p = Point::new(1.0, 0.0);
        assert!(approx_eq(p.angle(), 0.0));
    }

    #[test]
    fn test_point_angle_pi_over_2() {
        let p = Point::new(0.0, 1.0);
        assert!(approx_eq(p.angle(), FRAC_PI_2));
    }

    #[test]
    fn test_point_angle_pi() {
        let p = Point::new(-1.0, 0.0);
        assert!(approx_eq(p.angle(), PI));
    }

    #[test]
    fn test_point_angle_negative_pi_over_2() {
        let p = Point::new(0.0, -1.0);
        assert!(approx_eq(p.angle(), -FRAC_PI_2));
    }

    #[test]
    fn test_point_angle_45_degrees() {
        let p = Point::new(1.0, 1.0);
        assert!(approx_eq(p.angle(), FRAC_PI_4));
    }

    // ========================================================================
    // Point::from_angle tests
    // ========================================================================

    #[test]
    fn test_point_from_angle_zero() {
        let p = Point::from_angle(0.0);
        assert!(point_approx_eq(p, Point::new(1.0, 0.0)));
    }

    #[test]
    fn test_point_from_angle_pi_over_2() {
        let p = Point::from_angle(FRAC_PI_2);
        assert!(point_approx_eq(p, Point::new(0.0, 1.0)));
    }

    #[test]
    fn test_point_from_angle_pi() {
        let p = Point::from_angle(PI);
        assert!(point_approx_eq(p, Point::new(-1.0, 0.0)));
    }

    #[test]
    fn test_point_from_angle_2pi() {
        let p = Point::from_angle(TAU);
        assert!(point_approx_eq(p, Point::new(1.0, 0.0)));
    }

    #[test]
    fn test_point_from_angle_unit_length() {
        for angle in [0.0, 0.5, 1.0, 2.0, 3.0, -1.0, -2.0] {
            let p = Point::from_angle(angle);
            assert!(approx_eq(p.length(), 1.0));
        }
    }

    #[test]
    fn test_point_from_angle_roundtrip() {
        for angle in [0.0, FRAC_PI_4, FRAC_PI_2, PI, -FRAC_PI_4] {
            let p = Point::from_angle(angle);
            assert!(approx_eq(p.angle(), angle));
        }
    }

    // ========================================================================
    // Point::rotate tests
    // ========================================================================

    #[test]
    fn test_point_rotate_zero() {
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(p.rotate(0.0), p));
    }

    #[test]
    fn test_point_rotate_90_degrees() {
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(p.rotate(FRAC_PI_2), Point::new(0.0, 1.0)));
    }

    #[test]
    fn test_point_rotate_180_degrees() {
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(p.rotate(PI), Point::new(-1.0, 0.0)));
    }

    #[test]
    fn test_point_rotate_360_degrees() {
        let p = Point::new(3.0, 4.0);
        assert!(point_approx_eq(p.rotate(TAU), p));
    }

    #[test]
    fn test_point_rotate_negative() {
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(p.rotate(-FRAC_PI_2), Point::new(0.0, -1.0)));
    }

    #[test]
    fn test_point_rotate_preserves_length() {
        let p = Point::new(3.0, 4.0);
        let original_length = p.length();
        for angle in [0.5, 1.0, 2.0, PI, -1.0] {
            assert!(approx_eq(p.rotate(angle).length(), original_length));
        }
    }

    #[test]
    fn test_point_rotate_zero_vector() {
        let p = Point::ZERO;
        assert!(point_approx_eq(p.rotate(PI), Point::ZERO));
    }

    // ========================================================================
    // Point operator tests
    // ========================================================================

    #[test]
    fn test_point_ops() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(3.0, 4.0);
        assert_eq!(a + b, Point::new(4.0, 6.0));
        assert_eq!(b - a, Point::new(2.0, 2.0));
        assert_eq!(a * 2.0, Point::new(2.0, 4.0));
    }

    #[test]
    fn test_point_div() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p / 2.0, Point::new(5.0, 10.0));
    }

    #[test]
    fn test_point_neg() {
        let p = Point::new(3.0, -4.0);
        assert_eq!(-p, Point::new(-3.0, 4.0));
    }

    #[test]
    fn test_point_add_assign() {
        let mut p = Point::new(1.0, 2.0);
        p += Point::new(3.0, 4.0);
        assert_eq!(p, Point::new(4.0, 6.0));
    }

    // ========================================================================
    // Point conversion tests
    // ========================================================================

    #[test]
    fn test_point_from_tuple() {
        let p: Point = (3.0, 4.0).into();
        assert_eq!(p, Point::new(3.0, 4.0));
    }

    #[test]
    fn test_point_from_array() {
        let p: Point = [3.0, 4.0].into();
        assert_eq!(p, Point::new(3.0, 4.0));
    }

    // ========================================================================
    // Point edge cases
    // ========================================================================

    #[test]
    fn test_point_zero_constant() {
        assert_eq!(Point::ZERO, Point::new(0.0, 0.0));
    }

    #[test]
    fn test_point_default() {
        assert_eq!(Point::default(), Point::ZERO);
    }

    #[test]
    fn test_point_very_small_values() {
        let p = Point::new(1e-15, 1e-15);
        assert!(p.length() > 0.0);
    }

    #[test]
    fn test_point_very_large_values() {
        let p = Point::new(1e15, 1e15);
        let normalized = p.normalize();
        assert!(approx_eq(normalized.length(), 1.0));
    }

    // ========================================================================
    // Transform tests
    // ========================================================================

    fn transform_approx_eq(a: Transform, b: Transform) -> bool {
        approx_eq(a.a, b.a)
            && approx_eq(a.b, b.b)
            && approx_eq(a.c, b.c)
            && approx_eq(a.d, b.d)
            && approx_eq(a.tx, b.tx)
            && approx_eq(a.ty, b.ty)
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform::IDENTITY;
        let p = Point::new(10.0, 20.0);
        assert_eq!(t.apply(p), p);
    }

    #[test]
    fn test_transform_identity_default() {
        assert_eq!(Transform::default(), Transform::IDENTITY);
    }

    #[test]
    fn test_transform_translate() {
        let t = Transform::translate(5.0, 10.0);
        let p = Point::new(1.0, 2.0);
        assert_eq!(t.apply(p), Point::new(6.0, 12.0));
    }

    #[test]
    fn test_transform_translate_negative() {
        let t = Transform::translate(-5.0, -10.0);
        let p = Point::new(10.0, 20.0);
        assert_eq!(t.apply(p), Point::new(5.0, 10.0));
    }

    #[test]
    fn test_transform_translate_zero() {
        let t = Transform::translate(0.0, 0.0);
        assert!(transform_approx_eq(t, Transform::IDENTITY));
    }

    // === Rotation tests ===

    #[test]
    fn test_transform_rotate_zero() {
        let t = Transform::rotate(0.0);
        assert!(transform_approx_eq(t, Transform::IDENTITY));
    }

    #[test]
    fn test_transform_rotate_90() {
        let t = Transform::rotate(FRAC_PI_2);
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(t.apply(p), Point::new(0.0, 1.0)));
    }

    #[test]
    fn test_transform_rotate_180() {
        let t = Transform::rotate(PI);
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(t.apply(p), Point::new(-1.0, 0.0)));
    }

    #[test]
    fn test_transform_rotate_360() {
        let t = Transform::rotate(TAU);
        let p = Point::new(3.0, 4.0);
        assert!(point_approx_eq(t.apply(p), p));
    }

    #[test]
    fn test_transform_rotate_negative() {
        let t = Transform::rotate(-FRAC_PI_2);
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(t.apply(p), Point::new(0.0, -1.0)));
    }

    #[test]
    fn test_transform_rotate_preserves_length() {
        let t = Transform::rotate(1.234);
        let p = Point::new(3.0, 4.0);
        let rotated = t.apply(p);
        assert!(approx_eq(rotated.length(), p.length()));
    }

    #[test]
    fn test_transform_rotate_deg() {
        let t = Transform::rotate_deg(90.0);
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(t.apply(p), Point::new(0.0, 1.0)));
    }

    #[test]
    fn test_transform_rotate_deg_180() {
        let t = Transform::rotate_deg(180.0);
        let p = Point::new(1.0, 0.0);
        assert!(point_approx_eq(t.apply(p), Point::new(-1.0, 0.0)));
    }

    #[test]
    fn test_transform_rotate_deg_45() {
        let t = Transform::rotate_deg(45.0);
        let p = Point::new(1.0, 0.0);
        let sqrt2_over_2 = std::f64::consts::FRAC_1_SQRT_2;
        assert!(point_approx_eq(
            t.apply(p),
            Point::new(sqrt2_over_2, sqrt2_over_2)
        ));
    }

    // === Scale tests ===

    #[test]
    fn test_transform_scale() {
        let t = Transform::scale(2.0, 3.0);
        let p = Point::new(5.0, 10.0);
        assert_eq!(t.apply(p), Point::new(10.0, 30.0));
    }

    #[test]
    fn test_transform_scale_identity() {
        let t = Transform::scale(1.0, 1.0);
        assert!(transform_approx_eq(t, Transform::IDENTITY));
    }

    #[test]
    fn test_transform_scale_zero() {
        let t = Transform::scale(0.0, 0.0);
        let p = Point::new(100.0, 200.0);
        assert_eq!(t.apply(p), Point::ZERO);
    }

    #[test]
    fn test_transform_scale_negative() {
        let t = Transform::scale(-1.0, -1.0);
        let p = Point::new(5.0, 10.0);
        assert_eq!(t.apply(p), Point::new(-5.0, -10.0));
    }

    #[test]
    fn test_transform_scale_non_uniform() {
        let t = Transform::scale(2.0, 0.5);
        let p = Point::new(10.0, 10.0);
        assert_eq!(t.apply(p), Point::new(20.0, 5.0));
    }

    #[test]
    fn test_transform_scale_uniform() {
        let t = Transform::scale_uniform(3.0);
        let p = Point::new(5.0, 10.0);
        assert_eq!(t.apply(p), Point::new(15.0, 30.0));
    }

    #[test]
    fn test_transform_scale_uniform_preserves_angles() {
        let t = Transform::scale_uniform(2.0);
        let p = Point::new(1.0, 1.0);
        let scaled = t.apply(p);
        assert!(approx_eq(p.angle(), scaled.angle()));
    }

    // === Skew tests ===

    #[test]
    fn test_transform_skew_zero() {
        let t = Transform::skew(0.0, 0.0);
        assert!(transform_approx_eq(t, Transform::IDENTITY));
    }

    #[test]
    fn test_transform_skew_x() {
        let t = Transform::skew(FRAC_PI_4, 0.0); // 45 degree skew in x
        let p = Point::new(0.0, 1.0);
        // With 45 degree skew, y=1 adds tan(45)=1 to x
        assert!(point_approx_eq(t.apply(p), Point::new(1.0, 1.0)));
    }

    #[test]
    fn test_transform_skew_y() {
        let t = Transform::skew(0.0, FRAC_PI_4); // 45 degree skew in y
        let p = Point::new(1.0, 0.0);
        // With 45 degree skew, x=1 adds tan(45)=1 to y
        assert!(point_approx_eq(t.apply(p), Point::new(1.0, 1.0)));
    }

    // === Rotate around tests ===

    #[test]
    fn test_transform_rotate_around_origin() {
        let t = Transform::rotate_around(FRAC_PI_2, Point::ZERO);
        let p = Point::new(1.0, 0.0);
        // Same as regular rotation when center is origin
        assert!(point_approx_eq(t.apply(p), Point::new(0.0, 1.0)));
    }

    #[test]
    fn test_transform_rotate_around_point() {
        let center = Point::new(1.0, 0.0);
        let t = Transform::rotate_around(FRAC_PI_2, center);
        let p = Point::new(2.0, 0.0); // 1 unit to the right of center
                                      // After 90 degree rotation around (1,0), should be at (1,1)
        assert!(point_approx_eq(t.apply(p), Point::new(1.0, 1.0)));
    }

    #[test]
    fn test_transform_rotate_around_center_unchanged() {
        let center = Point::new(5.0, 5.0);
        let t = Transform::rotate_around(PI, center);
        // The center point should remain unchanged
        assert!(point_approx_eq(t.apply(center), center));
    }

    // === Composition tests ===

    #[test]
    fn test_transform_chain() {
        let t = Transform::translate(10.0, 0.0).then(Transform::scale(2.0, 2.0));
        let p = Point::new(0.0, 0.0);
        assert_eq!(t.apply(p), Point::new(20.0, 0.0));
    }

    #[test]
    fn test_transform_chain_order_matters() {
        let p = Point::new(1.0, 0.0);

        // Translate then scale
        let t1 = Transform::translate(1.0, 0.0).then(Transform::scale(2.0, 2.0));
        // Scale then translate
        let t2 = Transform::scale(2.0, 2.0).then(Transform::translate(1.0, 0.0));

        // Results should be different
        let r1 = t1.apply(p);
        let r2 = t2.apply(p);

        // t1: (1,0) -> translate -> (2,0) -> scale -> (4,0)
        assert!(point_approx_eq(r1, Point::new(4.0, 0.0)));
        // t2: (1,0) -> scale -> (2,0) -> translate -> (3,0)
        assert!(point_approx_eq(r2, Point::new(3.0, 0.0)));
    }

    #[test]
    fn test_transform_chain_identity() {
        let t = Transform::translate(5.0, 5.0);
        let chained = t.then(Transform::IDENTITY);
        assert!(transform_approx_eq(t, chained));
    }

    #[test]
    fn test_transform_mul_operator() {
        let t1 = Transform::translate(10.0, 0.0);
        let t2 = Transform::scale(2.0, 2.0);
        let combined = t1 * t2;
        assert!(transform_approx_eq(combined, t1.then(t2)));
    }

    #[test]
    fn test_transform_chain_rotate_translate() {
        let t = Transform::rotate(FRAC_PI_2).then(Transform::translate(1.0, 0.0));
        let p = Point::new(1.0, 0.0);
        // (1,0) -> rotate 90 -> (0,1) -> translate -> (1,1)
        assert!(point_approx_eq(t.apply(p), Point::new(1.0, 1.0)));
    }

    // === Inverse tests ===

    #[test]
    fn test_transform_inverse_identity() {
        let t = Transform::IDENTITY;
        let inv = t.inverse().unwrap();
        assert!(transform_approx_eq(inv, Transform::IDENTITY));
    }

    #[test]
    fn test_transform_inverse_translate() {
        let t = Transform::translate(5.0, 10.0);
        let inv = t.inverse().unwrap();
        let p = Point::new(1.0, 2.0);
        let transformed = t.apply(p);
        let back = inv.apply(transformed);
        assert!(point_approx_eq(back, p));
    }

    #[test]
    fn test_transform_inverse_scale() {
        let t = Transform::scale(2.0, 4.0);
        let inv = t.inverse().unwrap();
        let p = Point::new(10.0, 20.0);
        let back = inv.apply(t.apply(p));
        assert!(point_approx_eq(back, p));
    }

    #[test]
    fn test_transform_inverse_rotate() {
        let t = Transform::rotate(1.234);
        let inv = t.inverse().unwrap();
        let p = Point::new(3.0, 4.0);
        let back = inv.apply(t.apply(p));
        assert!(point_approx_eq(back, p));
    }

    #[test]
    fn test_transform_inverse_composed() {
        let t = Transform::translate(5.0, 10.0)
            .then(Transform::rotate(FRAC_PI_4))
            .then(Transform::scale(2.0, 3.0));
        let inv = t.inverse().unwrap();
        let p = Point::new(7.0, 11.0);
        let back = inv.apply(t.apply(p));
        assert!(point_approx_eq(back, p));
    }

    #[test]
    fn test_transform_inverse_singular_zero_scale() {
        let t = Transform::scale(0.0, 1.0);
        assert!(t.inverse().is_none());
    }

    #[test]
    fn test_transform_inverse_singular_both_zero() {
        let t = Transform::scale(0.0, 0.0);
        assert!(t.inverse().is_none());
    }

    // === Apply all tests ===

    #[test]
    fn test_transform_apply_all() {
        let t = Transform::translate(10.0, 20.0);
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
        ];
        let transformed = t.apply_all(&points);
        assert_eq!(transformed.len(), 3);
        assert_eq!(transformed[0], Point::new(10.0, 20.0));
        assert_eq!(transformed[1], Point::new(11.0, 21.0));
        assert_eq!(transformed[2], Point::new(12.0, 22.0));
    }

    #[test]
    fn test_transform_apply_all_empty() {
        let t = Transform::translate(10.0, 20.0);
        let points: Vec<Point> = vec![];
        let transformed = t.apply_all(&points);
        assert!(transformed.is_empty());
    }

    // ========================================================================
    // Color tests
    // ========================================================================

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::BLACK, Color::rgb(0, 0, 0));
        assert_eq!(Color::WHITE, Color::rgb(255, 255, 255));
        assert_eq!(Color::RED, Color::rgb(255, 0, 0));
        assert_eq!(Color::GREEN, Color::rgb(0, 255, 0));
        assert_eq!(Color::BLUE, Color::rgb(0, 0, 255));
        assert_eq!(Color::TRANSPARENT, Color::rgba(0, 0, 0, 0));
    }

    #[test]
    fn test_color_default() {
        assert_eq!(Color::default(), Color::BLACK);
    }

    #[test]
    fn test_color_rgb() {
        let c = Color::rgb(100, 150, 200);
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 150);
        assert_eq!(c.b, 200);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_rgba() {
        let c = Color::rgba(100, 150, 200, 128);
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 150);
        assert_eq!(c.b, 200);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn test_color_gray() {
        let c = Color::gray(128);
        assert_eq!(c.r, 128);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 128);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_with_alpha() {
        let c = Color::RED.with_alpha(128);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 128);
    }

    // === HSL tests ===

    #[test]
    fn test_color_hsl_red() {
        // Red: h=0, s=1, l=0.5
        let c = Color::hsl(0.0, 1.0, 0.5);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_color_hsl_green() {
        // Green: h=120, s=1, l=0.5
        let c = Color::hsl(120.0, 1.0, 0.5);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_color_hsl_blue() {
        // Blue: h=240, s=1, l=0.5
        let c = Color::hsl(240.0, 1.0, 0.5);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_color_hsl_yellow() {
        // Yellow: h=60, s=1, l=0.5
        let c = Color::hsl(60.0, 1.0, 0.5);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_color_hsl_cyan() {
        // Cyan: h=180, s=1, l=0.5
        let c = Color::hsl(180.0, 1.0, 0.5);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_color_hsl_magenta() {
        // Magenta: h=300, s=1, l=0.5
        let c = Color::hsl(300.0, 1.0, 0.5);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
    }

    #[test]
    fn test_color_hsl_lightness_zero_is_black() {
        // Any hue with l=0 should be black
        for h in [0.0, 60.0, 120.0, 180.0, 240.0, 300.0] {
            let c = Color::hsl(h, 1.0, 0.0);
            assert_eq!(c.r, 0, "h={} should have r=0", h);
            assert_eq!(c.g, 0, "h={} should have g=0", h);
            assert_eq!(c.b, 0, "h={} should have b=0", h);
        }
    }

    #[test]
    fn test_color_hsl_lightness_one_is_white() {
        // Any hue with l=1 should be white
        for h in [0.0, 60.0, 120.0, 180.0, 240.0, 300.0] {
            let c = Color::hsl(h, 1.0, 1.0);
            assert_eq!(c.r, 255, "h={} should have r=255", h);
            assert_eq!(c.g, 255, "h={} should have g=255", h);
            assert_eq!(c.b, 255, "h={} should have b=255", h);
        }
    }

    #[test]
    fn test_color_hsl_saturation_zero_is_gray() {
        // s=0 should give grayscale regardless of hue
        let c1 = Color::hsl(0.0, 0.0, 0.5);
        let c2 = Color::hsl(180.0, 0.0, 0.5);

        // Both should be the same gray
        assert_eq!(c1.r, c1.g);
        assert_eq!(c1.g, c1.b);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_color_hsl_saturation_zero_lightness_levels() {
        // s=0 should give gray at different lightness levels
        let black = Color::hsl(0.0, 0.0, 0.0);
        let mid_gray = Color::hsl(0.0, 0.0, 0.5);
        let white = Color::hsl(0.0, 0.0, 1.0);

        assert_eq!(black, Color::rgb(0, 0, 0));
        assert_eq!(white, Color::rgb(255, 255, 255));
        // Mid gray should have equal r, g, b around 127-128
        assert_eq!(mid_gray.r, mid_gray.g);
        assert_eq!(mid_gray.g, mid_gray.b);
        assert!(mid_gray.r >= 127 && mid_gray.r <= 128);
    }

    #[test]
    fn test_color_hsl_hue_ranges() {
        // Test each hue sector (0-60, 60-120, etc.)
        // Sector 0-60: red to yellow
        let c = Color::hsl(30.0, 1.0, 0.5);
        assert_eq!(c.r, 255);
        assert!(c.g > 0 && c.g < 255);
        assert_eq!(c.b, 0);

        // Sector 60-120: yellow to green
        let c = Color::hsl(90.0, 1.0, 0.5);
        assert!(c.r > 0 && c.r < 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 0);

        // Sector 120-180: green to cyan
        let c = Color::hsl(150.0, 1.0, 0.5);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert!(c.b > 0 && c.b < 255);

        // Sector 180-240: cyan to blue
        let c = Color::hsl(210.0, 1.0, 0.5);
        assert_eq!(c.r, 0);
        assert!(c.g > 0 && c.g < 255);
        assert_eq!(c.b, 255);

        // Sector 240-300: blue to magenta
        let c = Color::hsl(270.0, 1.0, 0.5);
        assert!(c.r > 0 && c.r < 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);

        // Sector 300-360: magenta to red
        let c = Color::hsl(330.0, 1.0, 0.5);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert!(c.b > 0 && c.b < 255);
    }

    #[test]
    fn test_color_hsl_partial_saturation() {
        // Reduced saturation should give muted colors
        let full = Color::hsl(0.0, 1.0, 0.5);
        let half = Color::hsl(0.0, 0.5, 0.5);

        // Full saturation red
        assert_eq!(full.r, 255);
        assert_eq!(full.g, 0);

        // Half saturation should have higher green/blue (more grayish)
        assert!(half.r > half.g);
        assert!(half.g > 0); // Not pure red anymore
    }

    #[test]
    fn test_color_hsl_lightness_range() {
        // l=0.25 should be darker, l=0.75 should be lighter
        let dark = Color::hsl(0.0, 1.0, 0.25);
        let mid = Color::hsl(0.0, 1.0, 0.5);
        let light = Color::hsl(0.0, 1.0, 0.75);

        // Dark red should have lower r than mid red
        assert!(dark.r < mid.r);
        // Light red should have higher g and b than mid (more white)
        assert!(light.g > mid.g);
        assert!(light.b > mid.b);
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
