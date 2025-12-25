//! Style type for stroke appearance

use serde::{Deserialize, Serialize};

use crate::Color;

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
