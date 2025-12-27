//! Style type for stroke appearance

use serde::{Deserialize, Serialize};

use crate::Color;

/// Style for stroke appearance with optional fields for inheritance
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct Style {
    /// Stroke width (None = inherit from parent)
    pub stroke_width: Option<f64>,
    /// Stroke color (None = inherit from parent)
    pub stroke_color: Option<Color>,
    /// Whether stroke width should scale with transforms (None = inherit, default true)
    pub scale_stroke: Option<bool>,
}

/// Default style values used when no parent style is available
pub const DEFAULT_STROKE_WIDTH: f64 = 1.0;
pub const DEFAULT_STROKE_COLOR: Color = Color::BLACK;

impl Style {
    pub fn new(width: f64, color: Color) -> Self {
        Self {
            stroke_width: Some(width),
            stroke_color: Some(color),
            scale_stroke: None,
        }
    }

    pub fn width(mut self, w: f64) -> Self {
        self.stroke_width = Some(w);
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.stroke_color = Some(c);
        self
    }

    /// Alias for width() for consistency with Element API
    pub fn with_stroke_width(self, w: f64) -> Self {
        self.width(w)
    }

    /// Alias for color() for consistency with Element API
    pub fn with_stroke_color(self, c: Color) -> Self {
        self.color(c)
    }

    /// Set whether stroke width should scale with transforms
    pub fn with_scale_stroke(mut self, scale: bool) -> Self {
        self.scale_stroke = Some(scale);
        self
    }

    /// Resolve this style by inheriting unset values from parent style
    pub fn resolve(&self, parent: &ResolvedStyle) -> ResolvedStyle {
        ResolvedStyle {
            stroke_width: self.stroke_width.unwrap_or(parent.stroke_width),
            stroke_color: self.stroke_color.unwrap_or(parent.stroke_color),
            scale_stroke: self.scale_stroke.unwrap_or(parent.scale_stroke),
        }
    }

    /// Resolve this style using defaults (no parent)
    pub fn resolve_with_defaults(&self) -> ResolvedStyle {
        ResolvedStyle {
            stroke_width: self.stroke_width.unwrap_or(DEFAULT_STROKE_WIDTH),
            stroke_color: self.stroke_color.unwrap_or(DEFAULT_STROKE_COLOR),
            scale_stroke: self.scale_stroke.unwrap_or(true),
        }
    }
}

/// A fully resolved style with concrete values (no inheritance needed)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResolvedStyle {
    pub stroke_width: f64,
    pub stroke_color: Color,
    /// Whether stroke width should scale with transforms (default: true)
    #[serde(default = "default_scale_stroke")]
    pub scale_stroke: bool,
}

fn default_scale_stroke() -> bool {
    true
}

impl Default for ResolvedStyle {
    fn default() -> Self {
        Self {
            stroke_width: DEFAULT_STROKE_WIDTH,
            stroke_color: DEFAULT_STROKE_COLOR,
            scale_stroke: true,
        }
    }
}

impl ResolvedStyle {
    pub fn new(stroke_width: f64, stroke_color: Color) -> Self {
        Self {
            stroke_width,
            stroke_color,
            scale_stroke: true,
        }
    }

    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_stroke_color(mut self, color: Color) -> Self {
        self.stroke_color = color;
        self
    }

    pub fn with_scale_stroke(mut self, scale: bool) -> Self {
        self.scale_stroke = scale;
        self
    }
}

impl From<ResolvedStyle> for Style {
    fn from(resolved: ResolvedStyle) -> Self {
        Style {
            stroke_width: Some(resolved.stroke_width),
            stroke_color: Some(resolved.stroke_color),
            scale_stroke: Some(resolved.scale_stroke),
        }
    }
}
