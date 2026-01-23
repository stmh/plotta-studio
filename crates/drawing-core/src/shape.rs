//! Shape enum combining all primitive types

use kurbo::{Line, Rect};
use serde::{Deserialize, Serialize};

use crate::clip::ClipGroup;
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
    ClipGroup(ClipGroup),
    Text(Text),
}

/// Macro to implement From<T> for Shape for multiple types
macro_rules! impl_from_for_shape {
    ($($type:ty => $variant:ident),+ $(,)?) => {
        $(
            impl From<$type> for Shape {
                fn from(v: $type) -> Self {
                    Shape::$variant(v)
                }
            }
        )+
    };
}

impl_from_for_shape! {
    Line => Line,
    Polyline => Polyline,
    Circle => Circle,
    Ellipse => Ellipse,
    Rect => Rect,
    Arc => Arc,
    RegularPolygon => RegularPolygon,
    Path => Path,
    Group => Group,
    ClipGroup => ClipGroup,
    Text => Text,
}
