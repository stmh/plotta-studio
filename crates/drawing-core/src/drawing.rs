//! Drawing - the top-level container

use kurbo::Point;
use serde::{Deserialize, Serialize};

use crate::element::Element;
use crate::stroke::Stroke;
use crate::Color;

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
