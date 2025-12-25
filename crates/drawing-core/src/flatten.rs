//! Flattening helpers for converting shapes to strokes

use std::f64::consts::TAU;

use kurbo::{Affine, PathEl, Point};

use crate::path::Path;
use crate::primitives::{Arc, Circle, Ellipse, RegularPolygon};
use crate::stroke::Stroke;
use crate::Style;

pub fn flatten_circle(circle: &Circle, transform: &Affine) -> Vec<Point> {
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

pub fn flatten_ellipse(ellipse: &Ellipse, transform: &Affine) -> Vec<Point> {
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

pub fn flatten_arc(arc: &Arc, transform: &Affine) -> Vec<Point> {
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

pub fn flatten_path(path: &Path, transform: &Affine, style: Style) -> Vec<Stroke> {
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
