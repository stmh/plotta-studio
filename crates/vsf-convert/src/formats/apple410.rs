//! Apple 410 Color Plotter font parser
//!
//! Parses the Apple 410 plotter font (1983) from JSON format.
//! Original firmware extraction by Adam Mayer (@phooky).

use anyhow::{Context, Result};
use drawing_core::Point;
use drawing_text::types::{Contour, ContourSegment, FontMetrics, Glyph};
use drawing_text::VsfFont;
use serde::Deserialize;
use std::collections::HashMap;

/// Raw glyph data from the JSON file
#[derive(Debug, Deserialize)]
struct Apple410Glyph {
    char: String,
    strokes: Vec<Vec<[f64; 2]>>,
}

/// Apple 410 font geometry constants (from the original ROM extraction)
const APPLE410_L: f64 = 25.0; // left margin
const APPLE410_YBASE: f64 = 825.0; // baseline
const APPLE410_YCAPTOP: f64 = 25.0; // top of caps
const APPLE410_UNITS_PER_EM: f64 = 800.0; // cap height

/// Parse Apple 410 font JSON to VsfFont
pub fn parse(json: &str) -> Result<VsfFont> {
    // Format: { "0x41": { "char": "A", "strokes": [[[x,y], [x,y], ...], ...] }, ... }
    let data: HashMap<String, Apple410Glyph> =
        serde_json::from_str(json).context("Failed to parse Apple 410 font JSON")?;

    let mut font = VsfFont::new("Apple 410");

    // Set metadata
    font.set_metadata(
        Some("Adam Mayer (@phooky)".to_string()),
        Some("MIT".to_string()),
        Some("Vector font from the Apple 410 Color Plotter (1983)".to_string()),
    );

    // Set metrics based on the original font geometry
    font.set_metrics(FontMetrics {
        units_per_em: APPLE410_UNITS_PER_EM,
        ascender: APPLE410_YBASE - APPLE410_YCAPTOP,
        descender: -200.0, // some chars go below baseline
        x_height: Some(500.0),
        cap_height: Some(APPLE410_YBASE - APPLE410_YCAPTOP),
        line_gap: 100.0,
    });

    for (_hex_code, glyph_data) in data {
        // Get the character
        let c = glyph_data.char.chars().next().unwrap_or('?');

        // Parse the glyph
        let glyph = parse_glyph(c, &glyph_data.strokes)?;
        font.add_glyph(c, glyph);
    }

    Ok(font)
}

fn parse_glyph(c: char, strokes: &[Vec<[f64; 2]>]) -> Result<Glyph> {
    // Normalize coordinates to our coordinate system
    // Original: Y increases downward, baseline at 825
    // Our system: Y increases upward, baseline at 0
    let normalize = |x: f64, y: f64| -> Point {
        let nx = x - APPLE410_L;
        let ny = APPLE410_YBASE - y; // flip Y and shift to baseline
        Point::new(nx, ny)
    };

    // The full character cell width is 792 units (from underscore: 817 - 25)
    // This matches the original Apple 410 plotter character spacing
    let advance = 792.0;
    let mut glyph = Glyph::new(c, advance);

    for stroke in strokes {
        if stroke.is_empty() {
            continue;
        }

        let mut contour = Contour::new();
        for (i, [x, y]) in stroke.iter().enumerate() {
            let point = normalize(*x, *y);
            if i == 0 {
                contour.segments.push(ContourSegment::MoveTo(point));
            } else {
                contour.segments.push(ContourSegment::LineTo(point));
            }
        }

        if !contour.segments.is_empty() {
            glyph = glyph.with_contour(contour);
        }
    }

    Ok(glyph)
}

/// Get the embedded Apple 410 font data
pub fn embedded_data() -> &'static str {
    include_str!("../../data/apple410.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_text::Font;

    const SAMPLE: &str = r#"{
        "0x41": {
            "char": "A",
            "strokes": [[[25,825],[289,25],[421,425],[157,425],[421,425],[553,825]]]
        },
        "0x20": {
            "char": " ",
            "strokes": []
        }
    }"#;

    #[test]
    fn test_parse_apple410() {
        let font = parse(SAMPLE).unwrap();
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph(' '));

        let glyph_a = font.glyph('A').unwrap();
        assert_eq!(glyph_a.contours.len(), 1);
    }
}
