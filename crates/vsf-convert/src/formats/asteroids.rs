//! Asteroids arcade font parser
//!
//! Parses the Asteroids vector font (1979 Atari) from JSON format.
//! Original by Ed Logg, C version by Trammell Hudson.

use anyhow::{Context, Result};
use drawing_core::Point;
use drawing_text::types::{Contour, ContourSegment, FontMetrics, Glyph};
use drawing_text::VsfFont;
use serde::Deserialize;
use std::collections::HashMap;

/// Raw glyph data from the JSON file
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GlyphElement {
    Point([f64; 2]),
    Command(String),
}

/// Parse Asteroids font JSON to VsfFont
pub fn parse(json: &str) -> Result<VsfFont> {
    // The JSON is a JavaScript object, so we need to handle it
    // Format: { "A": [[x,y], [x,y], "FONT_UP", [x,y], "FONT_LAST"], ... }
    let data: HashMap<String, Vec<GlyphElement>> =
        serde_json::from_str(json).context("Failed to parse Asteroids font JSON")?;

    let mut font = VsfFont::new("Asteroids");

    // Set metadata
    font.set_metadata(
        Some("Ed Logg (Atari), extracted by Trammell Hudson".to_string()),
        Some("Public Domain".to_string()),
        Some("Vector font from the 1979 Atari Asteroids arcade game".to_string()),
    );

    // Set metrics based on the font grid (12 units tall)
    font.set_metrics(FontMetrics {
        units_per_em: 12.0,
        ascender: 12.0,
        descender: 0.0,
        x_height: Some(8.0),
        cap_height: Some(12.0),
        line_gap: 2.0,
    });

    for (char_str, elements) in data {
        // Get the character
        let c = char_str.chars().next().unwrap_or('?');

        // Parse the glyph
        let glyph = parse_glyph(c, &elements)?;
        font.add_glyph(c, glyph);

        // For uppercase letters, also add lowercase version
        if c.is_ascii_uppercase() {
            let lower = c.to_ascii_lowercase();
            let glyph_lower = parse_glyph(lower, &elements)?;
            font.add_glyph(lower, glyph_lower);
        }
    }

    Ok(font)
}

fn parse_glyph(c: char, elements: &[GlyphElement]) -> Result<Glyph> {
    let mut glyph = Glyph::new(c, 12.0); // Fixed width of 12 units
    let mut contours = Vec::new();
    let mut current_contour = Contour::new();
    let mut pen_up = true;

    for element in elements {
        match element {
            GlyphElement::Point([x, y]) => {
                let point = Point::new(*x, *y);
                if pen_up {
                    // Start a new contour if we have points in the current one
                    if !current_contour.segments.is_empty() {
                        contours.push(current_contour);
                        current_contour = Contour::new();
                    }
                    current_contour.segments.push(ContourSegment::MoveTo(point));
                    pen_up = false;
                } else {
                    current_contour.segments.push(ContourSegment::LineTo(point));
                }
            }
            GlyphElement::Command(cmd) => match cmd.as_str() {
                "FONT_UP" => {
                    pen_up = true;
                }
                "FONT_LAST" => {
                    break;
                }
                _ => {}
            },
        }
    }

    // Add the last contour if it has points
    if !current_contour.segments.is_empty() {
        contours.push(current_contour);
    }

    for contour in contours {
        glyph = glyph.with_contour(contour);
    }

    Ok(glyph)
}

/// Get the embedded Asteroids font data
pub fn embedded_data() -> &'static str {
    include_str!("../../data/asteroids.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_text::Font;

    const SAMPLE: &str = r#"{
        "A": [[0,0],[0,8],[4,12],[8,8],[8,0],"FONT_UP",[0,4],[8,4],"FONT_LAST"],
        "B": [[0,0],[0,12],[4,12],[8,10],[4,6],[8,2],[4,0],[0,0],"FONT_LAST"]
    }"#;

    #[test]
    fn test_parse_asteroids() {
        let font = parse(SAMPLE).unwrap();
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('B'));

        let glyph_a = font.glyph('A').unwrap();
        assert_eq!(glyph_a.contours.len(), 2); // Two strokes (main + crossbar)
    }
}
