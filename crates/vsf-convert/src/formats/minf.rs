//! minf ultra-minimal font parser
//!
//! Parses the minf font (2024 Golan Levin) from base64 encoded data.
//! Each letter is 4 points with 2-bit coordinates, totaling 72 bytes.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use drawing_core::Point;
use drawing_text::types::{Contour, ContourSegment, FontMetrics, Glyph};
use drawing_text::VsfFont;

/// Parse minf font from base64 encoded string to VsfFont
pub fn parse(base64_data: &str) -> Result<VsfFont> {
    let bytes = STANDARD
        .decode(base64_data.trim())
        .context("Failed to decode minf base64 data")?;

    if bytes.len() != 52 {
        anyhow::bail!("Invalid minf data: expected 52 bytes, got {}", bytes.len());
    }

    let mut font = VsfFont::new("minf");

    // Set metadata
    font.set_metadata(
        Some("Golan Levin".to_string()),
        Some("CC0".to_string()),
        Some("Ultra-minimal 72-byte procedural font (2024)".to_string()),
    );

    // Set metrics based on the 4x4 grid (Y is doubled in rendering)
    font.set_metrics(FontMetrics {
        units_per_em: 8.0, // 4 units wide, 8 units tall (Y doubled)
        ascender: 6.0,
        descender: 0.0,
        x_height: Some(6.0),
        cap_height: Some(6.0),
        line_gap: 2.0,
    });

    // Decode each letter (A-Z, stored as lowercase in display)
    for i in 0..26 {
        let high_byte = bytes[i * 2];
        let low_byte = bytes[i * 2 + 1];
        let value = ((high_byte as u16) << 8) | (low_byte as u16);

        // Extract 4 points, each with 2-bit x and y
        let points = [
            ((value >> 14) & 0b11, (value >> 12) & 0b11),
            ((value >> 10) & 0b11, (value >> 8) & 0b11),
            ((value >> 6) & 0b11, (value >> 4) & 0b11),
            ((value >> 2) & 0b11, value & 0b11),
        ];

        let c = (b'a' + i as u8) as char;
        let glyph = create_glyph(c, &points);
        font.add_glyph(c, glyph);

        // Also add uppercase version
        let c_upper = (b'A' + i as u8) as char;
        let glyph_upper = create_glyph(c_upper, &points);
        font.add_glyph(c_upper, glyph_upper);
    }

    Ok(font)
}

fn create_glyph(c: char, points: &[(u16, u16); 4]) -> Glyph {
    let glyph = Glyph::new(c, 4.0); // 4 units advance
    let mut contour = Contour::new();

    for (i, (x, y)) in points.iter().enumerate() {
        // Y is doubled to match the original rendering
        // Flip Y: original has Y=0 at top, font coords have Y=0 at baseline with Y up
        // Max Y in source is 3 (2 bits), doubled to 6, so flip: 6 - y*2
        let point = Point::new(*x as f64, 6.0 - (*y as f64) * 2.0);
        if i == 0 {
            contour.segments.push(ContourSegment::MoveTo(point));
        } else {
            contour.segments.push(ContourSegment::LineTo(point));
        }
    }

    glyph.with_contour(contour)
}

/// The embedded minf font data (base64 encoded)
pub const EMBEDDED_DATA: &str =
    "+T4D0dE+zy1tG4Mdw/oDnxm/CLLTDwR/Nd8x/R1xMNL8HhNd0vOLHRvfF50X/R/TBcMdPw==";

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_text::Font;

    #[test]
    fn test_parse_minf() {
        let font = parse(EMBEDDED_DATA).unwrap();

        // Check we have all letters
        for c in 'a'..='z' {
            assert!(font.has_glyph(c), "Missing glyph: {}", c);
        }
        for c in 'A'..='Z' {
            assert!(font.has_glyph(c), "Missing glyph: {}", c);
        }

        // Check a specific glyph
        let glyph_a = font.glyph('a').unwrap();
        assert_eq!(glyph_a.contours.len(), 1);
        assert_eq!(glyph_a.contours[0].segments.len(), 4);
    }

    #[test]
    fn test_minf_data_length() {
        let bytes = STANDARD.decode(EMBEDDED_DATA).unwrap();
        assert_eq!(bytes.len(), 52);
    }
}
