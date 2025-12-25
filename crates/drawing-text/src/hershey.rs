//! Hershey font format parser and font implementation
//!
//! Hershey fonts are single-stroke vector fonts developed by Dr. Allen Vincent Hershey
//! at the Naval Weapons Laboratory in 1967. They remain public domain.
//!
//! JHF Format (one glyph per line):
//! ```text
//! NNNNN CC L R <coordinate pairs...>
//! ```
//! - NNNNN: 5-digit glyph number (whitespace padded)
//! - CC: Character count (number of coordinate pairs, 3 chars whitespace padded)
//! - L: Left margin (x offset from 'R')
//! - R: Right margin (x offset from 'R')
//! - Coordinate pairs: ASCII characters offset from 'R' (ASCII 82)
//! - ' R' (space + R) = pen up / move without drawing

use std::collections::HashMap;

use drawing_core::Point;

use crate::error::FontError;
use crate::font::Font;
use crate::types::{Contour, ContourSegment, FontMetrics, Glyph};

/// A Hershey font loaded from JHF or similar format
#[derive(Debug, Clone)]
pub struct HersheyFont {
    name: String,
    glyphs: HashMap<char, Glyph>,
    metrics: FontMetrics,
}

/// Mapping of Hershey glyph numbers to ASCII characters for Roman Simplex
/// Based on standard Hershey font conventions
const HERSHEY_ROMAN_SIMPLEX_MAP: &[(u32, char)] = &[
    (699, ' '), // space
    (714, '!'),
    (717, '"'),
    (733, '#'),
    (719, '$'),
    (2271, '%'),
    (734, '&'),
    (731, '\''),
    (721, '('),
    (722, ')'),
    (2219, '*'),
    (725, '+'),
    (711, ','),
    (724, '-'),
    (710, '.'),
    (720, '/'),
    (700, '0'),
    (701, '1'),
    (702, '2'),
    (703, '3'),
    (704, '4'),
    (705, '5'),
    (706, '6'),
    (707, '7'),
    (708, '8'),
    (709, '9'),
    (712, ':'),
    (713, ';'),
    (2241, '<'),
    (726, '='),
    (2242, '>'),
    (715, '?'),
    (2273, '@'),
    (501, 'A'),
    (502, 'B'),
    (503, 'C'),
    (504, 'D'),
    (505, 'E'),
    (506, 'F'),
    (507, 'G'),
    (508, 'H'),
    (509, 'I'),
    (510, 'J'),
    (511, 'K'),
    (512, 'L'),
    (513, 'M'),
    (514, 'N'),
    (515, 'O'),
    (516, 'P'),
    (517, 'Q'),
    (518, 'R'),
    (519, 'S'),
    (520, 'T'),
    (521, 'U'),
    (522, 'V'),
    (523, 'W'),
    (524, 'X'),
    (525, 'Y'),
    (526, 'Z'),
    (2223, '['),
    (804, '\\'),
    (2224, ']'),
    (2262, '^'),
    (999, '_'),
    (730, '`'),
    (601, 'a'),
    (602, 'b'),
    (603, 'c'),
    (604, 'd'),
    (605, 'e'),
    (606, 'f'),
    (607, 'g'),
    (608, 'h'),
    (609, 'i'),
    (610, 'j'),
    (611, 'k'),
    (612, 'l'),
    (613, 'm'),
    (614, 'n'),
    (615, 'o'),
    (616, 'p'),
    (617, 'q'),
    (618, 'r'),
    (619, 's'),
    (620, 't'),
    (621, 'u'),
    (622, 'v'),
    (623, 'w'),
    (624, 'x'),
    (625, 'y'),
    (626, 'z'),
    (2225, '{'),
    (723, '|'),
    (2226, '}'),
    (2246, '~'),
];

impl HersheyFont {
    /// Create a new empty Hershey font
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            glyphs: HashMap::new(),
            metrics: FontMetrics {
                units_per_em: 32.0, // Hershey fonts typically use a 32-unit height
                ascender: 21.0,
                descender: -11.0,
                x_height: Some(14.0),
                cap_height: Some(21.0),
                line_gap: 4.0,
            },
        }
    }

    /// Parse a Hershey font from JHF format data
    ///
    /// JHF format has one glyph per line. The glyph number is used to map to characters.
    pub fn from_jhf(name: &str, data: &str) -> Result<Self, FontError> {
        let mut font = Self::new(name);

        // Build reverse mapping from glyph number to character
        let glyph_to_char: HashMap<u32, char> = HERSHEY_ROMAN_SIMPLEX_MAP.iter().copied().collect();

        for line in data.lines() {
            // Don't trim leading spaces - they're significant in the format
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }

            if let Some((glyph_num, glyph)) = Self::parse_glyph_line(line)? {
                if let Some(&c) = glyph_to_char.get(&glyph_num) {
                    // Create glyph with the mapped character
                    let mut mapped_glyph = glyph;
                    mapped_glyph.unicode = c;
                    font.glyphs.insert(c, mapped_glyph);
                }
            }
        }

        Ok(font)
    }

    /// Parse a single glyph line from Hershey JHF format
    ///
    /// Format: `NNNNN CC L R <coordinate pairs...>`
    /// - First 5 chars: glyph number (whitespace padded)
    /// - Next 3 chars: count of vertices (includes L/R margin as first vertex)
    /// - Next 2 chars: left and right margins
    /// - Rest: coordinate pairs
    fn parse_glyph_line(line: &str) -> Result<Option<(u32, Glyph)>, FontError> {
        if line.len() < 10 {
            return Ok(None);
        }

        // Parse glyph number (first 5 chars)
        let glyph_num_str = &line[0..5];
        let glyph_num: u32 = glyph_num_str
            .trim()
            .parse()
            .map_err(|_| FontError::ParseError(format!("Invalid glyph number in: {}", line)))?;

        // Parse character count (positions 5-8)
        let count_str = &line[5..8];
        let count: usize = count_str
            .trim()
            .parse()
            .map_err(|_| FontError::ParseError(format!("Invalid count in: {}", line)))?;

        let bytes = line.as_bytes();

        // Parse left and right margins (positions 8-9)
        let left_margin = Self::decode_coord(bytes[8]);
        let right_margin = Self::decode_coord(bytes[9]);
        let advance_width = (right_margin - left_margin) as f64;

        // If count is 1, that means only the margins exist (space character)
        if count <= 1 {
            return Ok(Some((glyph_num, Glyph::new('?', advance_width))));
        }

        // Parse coordinate pairs starting at position 10
        let coord_data = &bytes[10..];
        let mut contours = Vec::new();
        let mut current_contour = Contour::new();
        let mut pen_down = false;

        let mut i = 0;
        // count includes the left/right margin pair, so actual data pairs = count - 1
        while i + 1 < coord_data.len() && i / 2 < count - 1 {
            let x_byte = coord_data[i];
            let y_byte = coord_data[i + 1];

            // Check for pen up command (space followed by R)
            if x_byte == b' ' && y_byte == b'R' {
                // Pen up - save current contour if it has points
                if !current_contour.segments.is_empty() {
                    contours.push(current_contour);
                    current_contour = Contour::new();
                }
                pen_down = false;
                i += 2;
                continue;
            }

            let x = Self::decode_coord(x_byte) as f64 - left_margin as f64;
            let y = Self::decode_coord(y_byte) as f64;

            if !pen_down {
                current_contour
                    .segments
                    .push(ContourSegment::MoveTo(Point::new(x, y)));
                pen_down = true;
            } else {
                current_contour
                    .segments
                    .push(ContourSegment::LineTo(Point::new(x, y)));
            }

            i += 2;
        }

        // Save final contour
        if !current_contour.segments.is_empty() {
            contours.push(current_contour);
        }

        let mut glyph = Glyph::new('?', advance_width);
        for contour in contours {
            glyph = glyph.with_contour(contour);
        }

        Ok(Some((glyph_num, glyph)))
    }

    /// Decode a Hershey coordinate byte
    /// Value = char - 'R' (ASCII 82)
    fn decode_coord(byte: u8) -> i32 {
        byte as i32 - 82 // 'R' = 82
    }

    /// Add a glyph to the font
    pub fn add_glyph(&mut self, c: char, glyph: Glyph) {
        self.glyphs.insert(c, glyph);
    }

    /// Set font metrics
    pub fn set_metrics(&mut self, metrics: FontMetrics) {
        self.metrics = metrics;
    }
}

impl Font for HersheyFont {
    fn name(&self) -> &str {
        &self.name
    }

    fn glyph(&self, c: char) -> Option<Glyph> {
        self.glyphs.get(&c).cloned()
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

/// Built-in Hershey Simplex Roman font data (public domain)
pub const HERSHEY_SIMPLEX_ROMAN: &str = include_str!("../../../fonts/hershey/simplex.jhf");

/// Load the built-in Hershey Simplex font
pub fn load_simplex() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf("Hershey Simplex", HERSHEY_SIMPLEX_ROMAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_coord() {
        assert_eq!(HersheyFont::decode_coord(b'R'), 0);
        assert_eq!(HersheyFont::decode_coord(b'S'), 1);
        assert_eq!(HersheyFont::decode_coord(b'Q'), -1);
        assert_eq!(HersheyFont::decode_coord(b'L'), -6);
        assert_eq!(HersheyFont::decode_coord(b'Z'), 8);
    }

    #[test]
    fn test_parse_simple_glyph() {
        // Letter A from Hershey Simplex (glyph 501)
        let line = "  501  9I[RFJ[ RRFZ[ RMTWT";
        let result = HersheyFont::parse_glyph_line(line).unwrap();
        assert!(result.is_some());

        let (glyph_num, glyph) = result.unwrap();
        assert_eq!(glyph_num, 501);
        assert!(!glyph.contours.is_empty());
    }

    #[test]
    fn test_hershey_font_creation() {
        let font = HersheyFont::new("Test Font");
        assert_eq!(font.name(), "Test Font");
        assert!(font.available_chars().is_empty());
    }

    #[test]
    fn test_load_simplex() {
        let font = load_simplex().unwrap();
        assert_eq!(font.name(), "Hershey Simplex");

        // Check that we have some basic characters
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('a'));
        assert!(font.has_glyph('0'));
        assert!(font.has_glyph(' '));

        // Check glyph 'A' has contours
        let glyph_a = font.glyph('A').unwrap();
        assert!(!glyph_a.contours.is_empty());
        assert!(glyph_a.advance_width > 0.0);
    }

    #[test]
    fn test_glyph_to_strokes() {
        use drawing_core::Style;

        let font = load_simplex().unwrap();
        let glyph = font.glyph('A').unwrap();
        let strokes = glyph.to_strokes(Style::default(), 0.1);

        assert!(!strokes.is_empty());
        for stroke in &strokes {
            assert!(!stroke.points.is_empty());
        }
    }
}
