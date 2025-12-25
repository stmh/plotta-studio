//! SVG font format parser and font implementation
//!
//! SVG fonts are defined using the `<font>` element in SVG files.
//! While deprecated in browsers, they remain useful for single-line stroke fonts
//! as they use standard SVG path syntax.
//!
//! # Format
//!
//! ```xml
//! <svg xmlns="http://www.w3.org/2000/svg">
//!   <defs>
//!     <font id="MyFont" horiz-adv-x="1000">
//!       <font-face font-family="MyFont" units-per-em="1000" ascent="800" descent="-200"/>
//!       <glyph unicode="A" horiz-adv-x="600" d="M0 0L300 700L600 0M100 250L500 250"/>
//!       <hkern u1="A" u2="V" k="50"/>
//!     </font>
//!   </defs>
//! </svg>
//! ```

use std::collections::HashMap;
use std::path::Path;

use drawing_core::Point;

use crate::error::FontError;
use crate::font::Font;
use crate::types::{Contour, ContourSegment, FontMetrics, Glyph};

/// An SVG font loaded from an SVG file with `<font>` element
#[derive(Debug, Clone)]
pub struct SvgFont {
    name: String,
    glyphs: HashMap<char, Glyph>,
    metrics: FontMetrics,
    kerning: HashMap<(char, char), f64>,
}

impl SvgFont {
    /// Create a new empty SVG font
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            glyphs: HashMap::new(),
            metrics: FontMetrics::default(),
            kerning: HashMap::new(),
        }
    }

    /// Load an SVG font from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FontError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parse an SVG font from a string
    pub fn from_str(svg: &str) -> Result<Self, FontError> {
        let options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let doc = roxmltree::Document::parse_with_options(svg, options)
            .map_err(|e| FontError::ParseError(format!("Invalid XML: {}", e)))?;

        // Find the <font> element
        let font_elem = doc
            .descendants()
            .find(|n| n.tag_name().name() == "font")
            .ok_or_else(|| FontError::InvalidFormat("No <font> element found".into()))?;

        // Get font name from id attribute or font-face
        let font_id = font_elem.attribute("id").unwrap_or("Unknown");

        // Get default advance width
        let default_advance: f64 = font_elem
            .attribute("horiz-adv-x")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000.0);

        // Parse font-face for metrics
        let metrics = Self::parse_font_face(&font_elem)?;

        // Parse glyphs
        let mut glyphs = HashMap::new();
        for glyph_elem in font_elem
            .children()
            .filter(|n| n.tag_name().name() == "glyph")
        {
            if let Some((c, glyph)) =
                Self::parse_glyph(&glyph_elem, default_advance, metrics.units_per_em)?
            {
                glyphs.insert(c, glyph);
            }
        }

        // Parse kerning
        let mut kerning = HashMap::new();
        for kern_elem in font_elem
            .children()
            .filter(|n| n.tag_name().name() == "hkern")
        {
            if let Some(((left, right), value)) = Self::parse_hkern(&kern_elem) {
                kerning.insert((left, right), value);
            }
        }

        // Get font name from font-face if available
        let name = font_elem
            .children()
            .find(|n| n.tag_name().name() == "font-face")
            .and_then(|ff| ff.attribute("font-family"))
            .unwrap_or(font_id)
            .to_string();

        Ok(Self {
            name,
            glyphs,
            metrics,
            kerning,
        })
    }

    /// Parse <font-face> element for metrics
    fn parse_font_face(font_elem: &roxmltree::Node) -> Result<FontMetrics, FontError> {
        let font_face = font_elem
            .children()
            .find(|n| n.tag_name().name() == "font-face");

        let mut metrics = FontMetrics::default();

        if let Some(ff) = font_face {
            if let Some(v) = ff.attribute("units-per-em").and_then(|s| s.parse().ok()) {
                metrics.units_per_em = v;
            }
            if let Some(v) = ff.attribute("ascent").and_then(|s| s.parse().ok()) {
                metrics.ascender = v;
            }
            if let Some(v) = ff.attribute("descent").and_then(|s| s.parse().ok()) {
                metrics.descender = v;
            }
            if let Some(v) = ff.attribute("x-height").and_then(|s| s.parse().ok()) {
                metrics.x_height = Some(v);
            }
            if let Some(v) = ff.attribute("cap-height").and_then(|s| s.parse().ok()) {
                metrics.cap_height = Some(v);
            }
        }

        Ok(metrics)
    }

    /// Parse a <glyph> element
    fn parse_glyph(
        glyph_elem: &roxmltree::Node,
        default_advance: f64,
        _units_per_em: f64,
    ) -> Result<Option<(char, Glyph)>, FontError> {
        // Get unicode character
        let unicode_str = match glyph_elem.attribute("unicode") {
            Some(s) => s,
            None => return Ok(None), // Skip glyphs without unicode (like .notdef)
        };

        // Parse unicode - could be a character, entity, or hex code
        let c = Self::parse_unicode(unicode_str)?;

        // Get advance width
        let advance: f64 = glyph_elem
            .attribute("horiz-adv-x")
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_advance);

        // Get path data
        let path_data = glyph_elem.attribute("d").unwrap_or("");

        // Parse path to contours
        let contours = if path_data.is_empty() {
            Vec::new()
        } else {
            Self::parse_svg_path(path_data)?
        };

        let mut glyph = Glyph::new(c, advance);
        for contour in contours {
            glyph = glyph.with_contour(contour);
        }

        Ok(Some((c, glyph)))
    }

    /// Parse unicode attribute value to char
    fn parse_unicode(s: &str) -> Result<char, FontError> {
        // Direct single character
        if s.chars().count() == 1 {
            return Ok(s.chars().next().unwrap());
        }

        // HTML entity like &#65; or &#x41;
        if s.starts_with("&#") && s.ends_with(';') {
            let inner = &s[2..s.len() - 1];
            let code = if inner.starts_with('x') || inner.starts_with('X') {
                u32::from_str_radix(&inner[1..], 16)
            } else {
                inner.parse::<u32>()
            };

            if let Ok(code) = code {
                if let Some(c) = char::from_u32(code) {
                    return Ok(c);
                }
            }
        }

        // If multiple characters, just take the first
        s.chars()
            .next()
            .ok_or_else(|| FontError::ParseError(format!("Invalid unicode: {}", s)))
    }

    /// Parse <hkern> element for kerning pair
    fn parse_hkern(kern_elem: &roxmltree::Node) -> Option<((char, char), f64)> {
        let u1 = kern_elem.attribute("u1")?;
        let u2 = kern_elem.attribute("u2")?;
        let k: f64 = kern_elem.attribute("k")?.parse().ok()?;

        let c1 = u1.chars().next()?;
        let c2 = u2.chars().next()?;

        Some(((c1, c2), k))
    }

    /// Parse SVG path data to contours
    fn parse_svg_path(d: &str) -> Result<Vec<Contour>, FontError> {
        use svgtypes::PathParser;
        use svgtypes::PathSegment as SvgSeg;

        let mut contours = Vec::new();
        let mut current_contour = Contour::new();
        let mut current_pos = Point::ORIGIN;
        let mut start_pos = Point::ORIGIN;

        for segment in PathParser::from(d) {
            let segment = segment
                .map_err(|e| FontError::ParseError(format!("Invalid path segment: {:?}", e)))?;

            match segment {
                SvgSeg::MoveTo { abs, x, y } => {
                    // Save current contour if it has points
                    if !current_contour.segments.is_empty() {
                        contours.push(current_contour);
                        current_contour = Contour::new();
                    }

                    let point = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };

                    // SVG fonts use Y-up, we use Y-down, so negate Y
                    let point = Point::new(point.x, -point.y);

                    current_contour.segments.push(ContourSegment::MoveTo(point));
                    current_pos = Point::new(x, y);
                    start_pos = current_pos;
                }

                SvgSeg::LineTo { abs, x, y } => {
                    let point = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };

                    let point = Point::new(point.x, -point.y);

                    current_contour.segments.push(ContourSegment::LineTo(point));
                    current_pos = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };
                }

                SvgSeg::HorizontalLineTo { abs, x } => {
                    let new_x = if abs { x } else { current_pos.x + x };
                    let point = Point::new(new_x, -current_pos.y);

                    current_contour.segments.push(ContourSegment::LineTo(point));
                    current_pos.x = new_x;
                }

                SvgSeg::VerticalLineTo { abs, y } => {
                    let new_y = if abs { y } else { current_pos.y + y };
                    let point = Point::new(current_pos.x, -new_y);

                    current_contour.segments.push(ContourSegment::LineTo(point));
                    current_pos.y = new_y;
                }

                SvgSeg::Quadratic { abs, x1, y1, x, y } => {
                    let (ctrl, to) = if abs {
                        (Point::new(x1, y1), Point::new(x, y))
                    } else {
                        (
                            Point::new(current_pos.x + x1, current_pos.y + y1),
                            Point::new(current_pos.x + x, current_pos.y + y),
                        )
                    };

                    let ctrl = Point::new(ctrl.x, -ctrl.y);
                    let to = Point::new(to.x, -to.y);

                    current_contour
                        .segments
                        .push(ContourSegment::QuadTo { ctrl, to });
                    current_pos = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };
                }

                SvgSeg::CurveTo {
                    abs,
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => {
                    let (ctrl1, ctrl2, to) = if abs {
                        (Point::new(x1, y1), Point::new(x2, y2), Point::new(x, y))
                    } else {
                        (
                            Point::new(current_pos.x + x1, current_pos.y + y1),
                            Point::new(current_pos.x + x2, current_pos.y + y2),
                            Point::new(current_pos.x + x, current_pos.y + y),
                        )
                    };

                    let ctrl1 = Point::new(ctrl1.x, -ctrl1.y);
                    let ctrl2 = Point::new(ctrl2.x, -ctrl2.y);
                    let to = Point::new(to.x, -to.y);

                    current_contour
                        .segments
                        .push(ContourSegment::CubicTo { ctrl1, ctrl2, to });
                    current_pos = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };
                }

                SvgSeg::ClosePath { .. } => {
                    current_contour.closed = true;
                    current_pos = start_pos;
                }

                // Handle smooth curves by computing reflected control points
                SvgSeg::SmoothQuadratic { abs, x, y } => {
                    // For smooth quadratic, we'd need to track the previous control point
                    // For simplicity, treat as line for now
                    let point = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };
                    let point = Point::new(point.x, -point.y);

                    current_contour.segments.push(ContourSegment::LineTo(point));
                    current_pos = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };
                }

                SvgSeg::SmoothCurveTo { abs, x2, y2, x, y } => {
                    // For smooth cubic, first control point is reflection of previous
                    // For simplicity, use endpoint as first control point
                    let (ctrl2, to) = if abs {
                        (Point::new(x2, y2), Point::new(x, y))
                    } else {
                        (
                            Point::new(current_pos.x + x2, current_pos.y + y2),
                            Point::new(current_pos.x + x, current_pos.y + y),
                        )
                    };

                    let ctrl1 = Point::new(current_pos.x, -current_pos.y);
                    let ctrl2 = Point::new(ctrl2.x, -ctrl2.y);
                    let to = Point::new(to.x, -to.y);

                    current_contour
                        .segments
                        .push(ContourSegment::CubicTo { ctrl1, ctrl2, to });
                    current_pos = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };
                }

                SvgSeg::EllipticalArc { abs, x, y, .. } => {
                    // Arcs are complex - approximate with line for now
                    let point = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };
                    let point = Point::new(point.x, -point.y);

                    current_contour.segments.push(ContourSegment::LineTo(point));
                    current_pos = if abs {
                        Point::new(x, y)
                    } else {
                        Point::new(current_pos.x + x, current_pos.y + y)
                    };
                }
            }
        }

        // Save final contour
        if !current_contour.segments.is_empty() {
            contours.push(current_contour);
        }

        Ok(contours)
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

impl Font for SvgFont {
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

    const SAMPLE_SVG_FONT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg">
  <defs>
    <font id="TestFont" horiz-adv-x="1000">
      <font-face 
        font-family="Test Font"
        units-per-em="1000"
        ascent="800"
        descent="-200"
        x-height="500"
        cap-height="700"
      />
      <glyph unicode="A" horiz-adv-x="600" d="M0 0L300 700L600 0M100 250L500 250"/>
      <glyph unicode="B" horiz-adv-x="550" d="M50 0L50 700L350 700L350 350L50 350"/>
      <glyph unicode=" " horiz-adv-x="300"/>
      <hkern u1="A" u2="V" k="50"/>
    </font>
  </defs>
</svg>"#;

    #[test]
    fn test_parse_svg_font() {
        let font = SvgFont::from_str(SAMPLE_SVG_FONT).unwrap();
        assert_eq!(font.name(), "Test Font");
    }

    #[test]
    fn test_svg_font_metrics() {
        let font = SvgFont::from_str(SAMPLE_SVG_FONT).unwrap();
        let metrics = font.metrics();

        assert_eq!(metrics.units_per_em, 1000.0);
        assert_eq!(metrics.ascender, 800.0);
        assert_eq!(metrics.descender, -200.0);
        assert_eq!(metrics.x_height, Some(500.0));
        assert_eq!(metrics.cap_height, Some(700.0));
    }

    #[test]
    fn test_svg_font_glyph() {
        let font = SvgFont::from_str(SAMPLE_SVG_FONT).unwrap();

        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('B'));
        assert!(font.has_glyph(' '));
        assert!(!font.has_glyph('Z'));

        let glyph_a = font.glyph('A').unwrap();
        assert_eq!(glyph_a.unicode, 'A');
        assert_eq!(glyph_a.advance_width, 600.0);
        assert_eq!(glyph_a.contours.len(), 2); // Two subpaths in A
    }

    #[test]
    fn test_svg_font_kerning() {
        let font = SvgFont::from_str(SAMPLE_SVG_FONT).unwrap();

        assert_eq!(font.kerning('A', 'V'), 50.0);
        assert_eq!(font.kerning('A', 'B'), 0.0);
    }

    #[test]
    fn test_parse_unicode() {
        assert_eq!(SvgFont::parse_unicode("A").unwrap(), 'A');
        assert_eq!(SvgFont::parse_unicode("&#65;").unwrap(), 'A');
        assert_eq!(SvgFont::parse_unicode("&#x41;").unwrap(), 'A');
    }

    #[test]
    fn test_svg_path_parsing() {
        let contours = SvgFont::parse_svg_path("M0 0L100 0L100 100L0 100Z").unwrap();
        assert_eq!(contours.len(), 1);
        assert!(contours[0].closed);
        assert_eq!(contours[0].segments.len(), 4); // M, L, L, L (Z just sets closed flag)
    }

    #[test]
    fn test_svg_path_multiple_subpaths() {
        let contours = SvgFont::parse_svg_path("M0 0L100 100 M200 200L300 300").unwrap();
        assert_eq!(contours.len(), 2);
    }

    #[test]
    fn test_glyph_to_strokes() {
        use drawing_core::Style;

        let font = SvgFont::from_str(SAMPLE_SVG_FONT).unwrap();
        let glyph = font.glyph('A').unwrap();
        let strokes = glyph.to_strokes(Style::default(), 0.1);

        assert!(!strokes.is_empty());
        for stroke in &strokes {
            assert!(!stroke.points.is_empty());
        }
    }
}
