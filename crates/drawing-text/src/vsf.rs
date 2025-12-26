//! VSF (Vector Stroke Font) format parser and font implementation
//!
//! VSF is a simple JSON-based format designed for single-line/stroke fonts.
//! It supports bezier curves and full font metrics.
//!
//! # Format Specification
//!
//! ```json
//! {
//!   "version": "1.0",
//!   "name": "Font Name",
//!   "metadata": {
//!     "author": "Author Name",
//!     "license": "MIT",
//!     "description": "Font description"
//!   },
//!   "metrics": {
//!     "units_per_em": 1000,
//!     "ascender": 800,
//!     "descender": -200,
//!     "x_height": 500,
//!     "cap_height": 700,
//!     "line_gap": 100
//!   },
//!   "glyphs": {
//!     "A": {
//!       "unicode": 65,
//!       "advance": 600,
//!       "contours": [...]
//!     }
//!   },
//!   "kerning": {
//!     "AV": -50,
//!     "VA": -50
//!   }
//! }
//! ```

use std::collections::HashMap;
use std::path::Path;

use drawing_core::Point;
use serde::{Deserialize, Serialize};

use crate::error::FontError;
use crate::font::Font;
use crate::types::{Contour, ContourSegment, FontMetrics, Glyph};

/// VSF file structure (JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsfFile {
    /// Format version
    pub version: String,
    /// Font name
    pub name: String,
    /// Optional metadata
    #[serde(default)]
    pub metadata: VsfMetadata,
    /// Font metrics
    pub metrics: VsfMetrics,
    /// Glyphs indexed by character
    pub glyphs: HashMap<String, VsfGlyph>,
    /// Optional kerning pairs
    #[serde(default)]
    pub kerning: HashMap<String, f64>,
}

/// VSF metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VsfMetadata {
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// VSF font metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsfMetrics {
    pub units_per_em: f64,
    pub ascender: f64,
    pub descender: f64,
    #[serde(default)]
    pub x_height: Option<f64>,
    #[serde(default)]
    pub cap_height: Option<f64>,
    #[serde(default)]
    pub line_gap: f64,
}

/// VSF glyph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsfGlyph {
    /// Unicode code point
    pub unicode: u32,
    /// Advance width
    pub advance: f64,
    /// Contours making up the glyph
    pub contours: Vec<VsfContour>,
}

/// VSF contour
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsfContour {
    /// Whether the contour is closed
    #[serde(default)]
    pub closed: bool,
    /// Points in the contour
    pub points: Vec<VsfPoint>,
}

/// VSF point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsfPoint {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Point type: "move", "line", "quad", "cubic"
    #[serde(rename = "type")]
    pub point_type: String,
    /// Control points for curves (optional)
    #[serde(default)]
    pub ctrl: Option<Vec<f64>>,
}

/// A VSF font loaded from JSON
#[derive(Debug, Clone)]
pub struct VsfFont {
    name: String,
    glyphs: HashMap<char, Glyph>,
    metrics: FontMetrics,
    kerning: HashMap<(char, char), f64>,
    metadata: VsfMetadata,
}

impl VsfFont {
    /// Create a new empty VSF font
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            glyphs: HashMap::new(),
            metrics: FontMetrics::default(),
            kerning: HashMap::new(),
            metadata: VsfMetadata::default(),
        }
    }

    /// Set font metadata (author, license, description)
    pub fn set_metadata(
        &mut self,
        author: Option<String>,
        license: Option<String>,
        description: Option<String>,
    ) {
        self.metadata = VsfMetadata {
            author,
            license,
            description,
        };
    }

    /// Load a VSF font from JSON data
    pub fn from_json(data: &str) -> Result<Self, FontError> {
        let vsf: VsfFile = serde_json::from_str(data)?;
        Self::from_vsf_file(vsf)
    }

    /// Load a VSF font from a file path
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FontError> {
        let path = path.as_ref();
        let data = std::fs::read_to_string(path)
            .map_err(|e| FontError::IoError(path.to_path_buf(), e.to_string()))?;
        Self::from_json(&data)
    }

    /// Convert VsfFile to VsfFont
    fn from_vsf_file(vsf: VsfFile) -> Result<Self, FontError> {
        let metrics = FontMetrics {
            units_per_em: vsf.metrics.units_per_em,
            ascender: vsf.metrics.ascender,
            descender: vsf.metrics.descender,
            x_height: vsf.metrics.x_height,
            cap_height: vsf.metrics.cap_height,
            line_gap: vsf.metrics.line_gap,
        };

        let mut glyphs = HashMap::new();
        for (key, vsf_glyph) in vsf.glyphs {
            // Get character from key (should be single char) or from unicode
            let c = key
                .chars()
                .next()
                .unwrap_or_else(|| char::from_u32(vsf_glyph.unicode).unwrap_or('?'));

            let glyph = Self::convert_glyph(c, &vsf_glyph)?;
            glyphs.insert(c, glyph);
        }

        let mut kerning = HashMap::new();
        for (pair, value) in vsf.kerning {
            let mut chars = pair.chars();
            if let (Some(left), Some(right)) = (chars.next(), chars.next()) {
                kerning.insert((left, right), value);
            }
        }

        Ok(Self {
            name: vsf.name,
            glyphs,
            metrics,
            kerning,
            metadata: vsf.metadata,
        })
    }

    /// Convert a VSF glyph to our Glyph type
    fn convert_glyph(c: char, vsf_glyph: &VsfGlyph) -> Result<Glyph, FontError> {
        let mut glyph = Glyph::new(c, vsf_glyph.advance);

        for vsf_contour in &vsf_glyph.contours {
            let contour = Self::convert_contour(vsf_contour)?;
            glyph = glyph.with_contour(contour);
        }

        Ok(glyph)
    }

    /// Convert a VSF contour to our Contour type
    fn convert_contour(vsf_contour: &VsfContour) -> Result<Contour, FontError> {
        let mut contour = Contour::new();

        for point in &vsf_contour.points {
            let segment = match point.point_type.as_str() {
                "move" => ContourSegment::MoveTo(Point::new(point.x, point.y)),
                "line" => ContourSegment::LineTo(Point::new(point.x, point.y)),
                "quad" => {
                    let ctrl = point.ctrl.as_ref().ok_or_else(|| {
                        FontError::InvalidFormat("Quad point missing control point".into())
                    })?;
                    if ctrl.len() < 2 {
                        return Err(FontError::InvalidFormat(
                            "Quad point needs 2 control values".into(),
                        ));
                    }
                    ContourSegment::QuadTo {
                        ctrl: Point::new(ctrl[0], ctrl[1]),
                        to: Point::new(point.x, point.y),
                    }
                }
                "cubic" => {
                    let ctrl = point.ctrl.as_ref().ok_or_else(|| {
                        FontError::InvalidFormat("Cubic point missing control points".into())
                    })?;
                    if ctrl.len() < 4 {
                        return Err(FontError::InvalidFormat(
                            "Cubic point needs 4 control values".into(),
                        ));
                    }
                    ContourSegment::CubicTo {
                        ctrl1: Point::new(ctrl[0], ctrl[1]),
                        ctrl2: Point::new(ctrl[2], ctrl[3]),
                        to: Point::new(point.x, point.y),
                    }
                }
                _ => {
                    return Err(FontError::InvalidFormat(format!(
                        "Unknown point type: {}",
                        point.point_type
                    )));
                }
            };
            contour.segments.push(segment);
        }

        contour.closed = vsf_contour.closed;
        Ok(contour)
    }

    /// Export the font to VSF JSON format
    pub fn to_json(&self) -> Result<String, FontError> {
        let vsf = self.to_vsf_file();
        Ok(serde_json::to_string_pretty(&vsf)?)
    }

    /// Convert to VsfFile structure
    fn to_vsf_file(&self) -> VsfFile {
        let mut glyphs = HashMap::new();
        for (&c, glyph) in &self.glyphs {
            let vsf_glyph = Self::glyph_to_vsf(glyph);
            glyphs.insert(c.to_string(), vsf_glyph);
        }

        let mut kerning = HashMap::new();
        for (&(left, right), &value) in &self.kerning {
            let key = format!("{}{}", left, right);
            kerning.insert(key, value);
        }

        VsfFile {
            version: "1.0".to_string(),
            name: self.name.clone(),
            metadata: self.metadata.clone(),
            metrics: VsfMetrics {
                units_per_em: self.metrics.units_per_em,
                ascender: self.metrics.ascender,
                descender: self.metrics.descender,
                x_height: self.metrics.x_height,
                cap_height: self.metrics.cap_height,
                line_gap: self.metrics.line_gap,
            },
            glyphs,
            kerning,
        }
    }

    /// Convert a Glyph to VSF format
    fn glyph_to_vsf(glyph: &Glyph) -> VsfGlyph {
        let contours = glyph
            .contours
            .iter()
            .map(|c| {
                let points = c
                    .segments
                    .iter()
                    .map(|seg| match seg {
                        ContourSegment::MoveTo(p) => VsfPoint {
                            x: p.x,
                            y: p.y,
                            point_type: "move".to_string(),
                            ctrl: None,
                        },
                        ContourSegment::LineTo(p) => VsfPoint {
                            x: p.x,
                            y: p.y,
                            point_type: "line".to_string(),
                            ctrl: None,
                        },
                        ContourSegment::QuadTo { ctrl, to } => VsfPoint {
                            x: to.x,
                            y: to.y,
                            point_type: "quad".to_string(),
                            ctrl: Some(vec![ctrl.x, ctrl.y]),
                        },
                        ContourSegment::CubicTo { ctrl1, ctrl2, to } => VsfPoint {
                            x: to.x,
                            y: to.y,
                            point_type: "cubic".to_string(),
                            ctrl: Some(vec![ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y]),
                        },
                    })
                    .collect();

                VsfContour {
                    closed: c.closed,
                    points,
                }
            })
            .collect();

        VsfGlyph {
            unicode: glyph.unicode as u32,
            advance: glyph.advance_width,
            contours,
        }
    }

    /// Add a glyph to the font
    pub fn add_glyph(&mut self, c: char, glyph: Glyph) {
        self.glyphs.insert(c, glyph);
    }

    /// Set kerning for a pair
    pub fn set_kerning(&mut self, left: char, right: char, value: f64) {
        self.kerning.insert((left, right), value);
    }

    /// Set font metrics
    pub fn set_metrics(&mut self, metrics: FontMetrics) {
        self.metrics = metrics;
    }
}

impl Font for VsfFont {
    fn name(&self) -> &str {
        &self.name
    }

    fn glyph(&self, c: char) -> Option<Glyph> {
        self.glyphs.get(&c).cloned()
    }

    fn kerning(&self, left: char, right: char) -> f64 {
        self.kerning.get(&(left, right)).copied().unwrap_or(0.0)
    }

    fn metrics(&self) -> FontMetrics {
        self.metrics.clone()
    }

    fn available_chars(&self) -> Vec<char> {
        let mut chars: Vec<char> = self.glyphs.keys().copied().collect();
        chars.sort();
        chars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_VSF: &str = r#"{
        "version": "1.0",
        "name": "Test Font",
        "metrics": {
            "units_per_em": 1000,
            "ascender": 800,
            "descender": -200,
            "x_height": 500,
            "cap_height": 700,
            "line_gap": 100
        },
        "glyphs": {
            "A": {
                "unicode": 65,
                "advance": 600,
                "contours": [
                    {
                        "closed": false,
                        "points": [
                            {"x": 0, "y": 0, "type": "move"},
                            {"x": 300, "y": 700, "type": "line"},
                            {"x": 600, "y": 0, "type": "line"}
                        ]
                    },
                    {
                        "closed": false,
                        "points": [
                            {"x": 150, "y": 250, "type": "move"},
                            {"x": 450, "y": 250, "type": "line"}
                        ]
                    }
                ]
            }
        },
        "kerning": {
            "AV": -50
        }
    }"#;

    #[test]
    fn test_parse_vsf() {
        let font = VsfFont::from_json(SAMPLE_VSF).unwrap();
        assert_eq!(font.name(), "Test Font");
        assert!(font.has_glyph('A'));
    }

    #[test]
    fn test_vsf_glyph() {
        let font = VsfFont::from_json(SAMPLE_VSF).unwrap();
        let glyph = font.glyph('A').unwrap();

        assert_eq!(glyph.unicode, 'A');
        assert_eq!(glyph.advance_width, 600.0);
        assert_eq!(glyph.contours.len(), 2);
    }

    #[test]
    fn test_vsf_metrics() {
        let font = VsfFont::from_json(SAMPLE_VSF).unwrap();
        let metrics = font.metrics();

        assert_eq!(metrics.units_per_em, 1000.0);
        assert_eq!(metrics.ascender, 800.0);
        assert_eq!(metrics.descender, -200.0);
    }

    #[test]
    fn test_vsf_kerning() {
        let font = VsfFont::from_json(SAMPLE_VSF).unwrap();
        assert_eq!(font.kerning('A', 'V'), -50.0);
        assert_eq!(font.kerning('A', 'B'), 0.0);
    }

    #[test]
    fn test_vsf_roundtrip() {
        let font = VsfFont::from_json(SAMPLE_VSF).unwrap();
        let json = font.to_json().unwrap();
        let font2 = VsfFont::from_json(&json).unwrap();

        assert_eq!(font.name(), font2.name());
        assert_eq!(font.available_chars(), font2.available_chars());
    }

    #[test]
    fn test_vsf_with_curves() {
        let vsf_with_curves = r#"{
            "version": "1.0",
            "name": "Curve Test",
            "metrics": {
                "units_per_em": 1000,
                "ascender": 800,
                "descender": -200
            },
            "glyphs": {
                "O": {
                    "unicode": 79,
                    "advance": 700,
                    "contours": [
                        {
                            "closed": true,
                            "points": [
                                {"x": 350, "y": 0, "type": "move"},
                                {"x": 700, "y": 350, "type": "quad", "ctrl": [700, 0]},
                                {"x": 350, "y": 700, "type": "quad", "ctrl": [700, 700]},
                                {"x": 0, "y": 350, "type": "quad", "ctrl": [0, 700]},
                                {"x": 350, "y": 0, "type": "quad", "ctrl": [0, 0]}
                            ]
                        }
                    ]
                }
            }
        }"#;

        let font = VsfFont::from_json(vsf_with_curves).unwrap();
        let glyph = font.glyph('O').unwrap();

        assert_eq!(glyph.contours.len(), 1);
        assert!(glyph.contours[0].closed);
        assert_eq!(glyph.contours[0].segments.len(), 5);
    }
}
