//! Shape enum combining all primitive types

use kurbo::{Line, Rect};
use serde::{Deserialize, Serialize};

use crate::group::Group;
use crate::path::Path;
use crate::primitives::{Arc, Circle, Ellipse, Polyline, RegularPolygon};
use crate::text::Text;

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
    Text(Text),
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

impl From<Text> for Shape {
    fn from(v: Text) -> Self {
        Shape::Text(v)
    }
}
