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

/// Hershey font variants
///
/// This enum provides type-safe access to the built-in Hershey font variants.
/// Use with `FontManager::load_hershey()` to load fonts.
///
/// # Example
///
/// ```rust,ignore
/// use drawing_text::{FontManager, Hershey};
///
/// let manager = FontManager::new(registry);
/// manager.load_hershey(Hershey::Simplex)?;
///
/// // Get font using the enum (implements AsRef<str>)
/// let font = registry.get(Hershey::Simplex);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hershey {
    /// Roman Simplex - single-stroke, clean and readable
    Simplex,
    /// Roman Duplex - double-stroke, bolder
    Duplex,
    /// Roman Triplex - triple-stroke, most ornate
    Triplex,
    /// Script Simplex - cursive, single-stroke
    ScriptSimplex,
    /// Script Complex - cursive, double-stroke
    ScriptComplex,
    /// Gothic German Bold - Fraktur-style, bold
    GothicGermanBold,
    /// Gothic German - Fraktur-style
    GothicGerman,
    /// Gothic Italian - Fraktur-style, Italian variant
    GothicItalian,
}

impl Hershey {
    /// Get the font name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Hershey::Simplex => "Hershey Simplex",
            Hershey::Duplex => "Hershey Duplex",
            Hershey::Triplex => "Hershey Triplex",
            Hershey::ScriptSimplex => "Hershey Script Simplex",
            Hershey::ScriptComplex => "Hershey Script Complex",
            Hershey::GothicGermanBold => "Hershey Gothic German Bold",
            Hershey::GothicGerman => "Hershey Gothic German",
            Hershey::GothicItalian => "Hershey Gothic Italian",
        }
    }

    /// Load this font variant
    pub fn load(&self) -> Result<HersheyFont, FontError> {
        match self {
            Hershey::Simplex => load_simplex(),
            Hershey::Duplex => load_duplex(),
            Hershey::Triplex => load_triplex(),
            Hershey::ScriptSimplex => load_script_simplex(),
            Hershey::ScriptComplex => load_script_complex(),
            Hershey::GothicGermanBold => load_gothic_german_bold(),
            Hershey::GothicGerman => load_gothic_german(),
            Hershey::GothicItalian => load_gothic_italian(),
        }
    }

    /// Get all available Hershey font variants
    pub fn all() -> &'static [Hershey] {
        &[
            Hershey::Simplex,
            Hershey::Duplex,
            Hershey::Triplex,
            Hershey::ScriptSimplex,
            Hershey::ScriptComplex,
            Hershey::GothicGermanBold,
            Hershey::GothicGerman,
            Hershey::GothicItalian,
        ]
    }
}

impl AsRef<str> for Hershey {
    fn as_ref(&self) -> &str {
        self.name()
    }
}

impl std::fmt::Display for Hershey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

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

/// Mapping for Hershey Script Simplex font
/// Script fonts use different glyph numbers than Roman fonts
const HERSHEY_SCRIPT_SIMPLEX_MAP: &[(u32, char)] = &[
    (699, ' '), // space (shared)
    (2764, '!'),
    (2778, '"'),
    (733, '#'), // shared
    (2769, '$'),
    (2271, '%'), // shared
    (2768, '&'),
    (2767, '\''),
    (2771, '('),
    (2772, ')'),
    (2773, '*'),
    (725, '+'), // shared
    (2761, ','),
    (724, '-'), // shared
    (710, '.'), // shared
    (2770, '/'),
    (2750, '0'),
    (2751, '1'),
    (2752, '2'),
    (2753, '3'),
    (2754, '4'),
    (2755, '5'),
    (2756, '6'),
    (2757, '7'),
    (2758, '8'),
    (2759, '9'),
    (2762, ':'),
    (2763, ';'),
    (2241, '<'), // shared
    (726, '='),  // shared
    (2242, '>'), // shared
    (2765, '?'),
    (2273, '@'), // shared
    (551, 'A'),
    (552, 'B'),
    (553, 'C'),
    (554, 'D'),
    (555, 'E'),
    (556, 'F'),
    (557, 'G'),
    (558, 'H'),
    (559, 'I'),
    (560, 'J'),
    (561, 'K'),
    (562, 'L'),
    (563, 'M'),
    (564, 'N'),
    (565, 'O'),
    (566, 'P'),
    (567, 'Q'),
    (568, 'R'),
    (569, 'S'),
    (570, 'T'),
    (571, 'U'),
    (572, 'V'),
    (573, 'W'),
    (574, 'X'),
    (575, 'Y'),
    (576, 'Z'),
    (2223, '['), // shared
    (804, '\\'), // shared
    (2224, ']'), // shared
    (2262, '^'), // shared
    (999, '_'),  // shared
    (730, '`'),  // shared
    (651, 'a'),
    (652, 'b'),
    (653, 'c'),
    (654, 'd'),
    (655, 'e'),
    (656, 'f'),
    (657, 'g'),
    (658, 'h'),
    (659, 'i'),
    (660, 'j'),
    (661, 'k'),
    (662, 'l'),
    (663, 'm'),
    (664, 'n'),
    (665, 'o'),
    (666, 'p'),
    (667, 'q'),
    (668, 'r'),
    (669, 's'),
    (670, 't'),
    (671, 'u'),
    (672, 'v'),
    (673, 'w'),
    (674, 'x'),
    (675, 'y'),
    (676, 'z'),
    (2225, '{'), // shared
    (723, '|'),  // shared
    (2226, '}'), // shared
    (2246, '~'), // shared
];

/// Mapping for Hershey Script Complex font (offset from Script Simplex by 2000)
const HERSHEY_SCRIPT_COMPLEX_MAP: &[(u32, char)] = &[
    (699, ' '), // space (shared)
    (2764, '!'),
    (2778, '"'),
    (733, '#'),
    (2769, '$'),
    (2271, '%'),
    (2768, '&'),
    (2767, '\''),
    (2771, '('),
    (2772, ')'),
    (2773, '*'),
    (725, '+'),
    (2761, ','),
    (724, '-'),
    (710, '.'),
    (2770, '/'),
    (2750, '0'),
    (2751, '1'),
    (2752, '2'),
    (2753, '3'),
    (2754, '4'),
    (2755, '5'),
    (2756, '6'),
    (2757, '7'),
    (2758, '8'),
    (2759, '9'),
    (2762, ':'),
    (2763, ';'),
    (2241, '<'),
    (726, '='),
    (2242, '>'),
    (2765, '?'),
    (2273, '@'),
    (2551, 'A'),
    (2552, 'B'),
    (2553, 'C'),
    (2554, 'D'),
    (2555, 'E'),
    (2556, 'F'),
    (2557, 'G'),
    (2558, 'H'),
    (2559, 'I'),
    (2560, 'J'),
    (2561, 'K'),
    (2562, 'L'),
    (2563, 'M'),
    (2564, 'N'),
    (2565, 'O'),
    (2566, 'P'),
    (2567, 'Q'),
    (2568, 'R'),
    (2569, 'S'),
    (2570, 'T'),
    (2571, 'U'),
    (2572, 'V'),
    (2573, 'W'),
    (2574, 'X'),
    (2575, 'Y'),
    (2576, 'Z'),
    (2223, '['),
    (804, '\\'),
    (2224, ']'),
    (2262, '^'),
    (999, '_'),
    (730, '`'),
    (2651, 'a'),
    (2652, 'b'),
    (2653, 'c'),
    (2654, 'd'),
    (2655, 'e'),
    (2656, 'f'),
    (2657, 'g'),
    (2658, 'h'),
    (2659, 'i'),
    (2660, 'j'),
    (2661, 'k'),
    (2662, 'l'),
    (2663, 'm'),
    (2664, 'n'),
    (2665, 'o'),
    (2666, 'p'),
    (2667, 'q'),
    (2668, 'r'),
    (2669, 's'),
    (2670, 't'),
    (2671, 'u'),
    (2672, 'v'),
    (2673, 'w'),
    (2674, 'x'),
    (2675, 'y'),
    (2676, 'z'),
    (2225, '{'),
    (723, '|'),
    (2226, '}'),
    (2246, '~'),
];

/// Mapping for Hershey Gothic fonts (German, Italian variants)
/// Gothic fonts use 3301-3326 for uppercase and 3401-3426 for lowercase
const HERSHEY_GOTHIC_MAP: &[(u32, char)] = &[
    (3699, ' '), // space
    (3714, '!'),
    (3728, '"'),
    (2275, '#'), // shared
    (3719, '$'),
    (2271, '%'), // shared
    (3718, '&'),
    (3717, '\''),
    (3721, '('),
    (3722, ')'),
    (3723, '*'),
    (3725, '+'),
    (3711, ','),
    (3724, '-'),
    (3710, '.'),
    (3720, '/'),
    (3700, '0'),
    (3701, '1'),
    (3702, '2'),
    (3703, '3'),
    (3704, '4'),
    (3705, '5'),
    (3706, '6'),
    (3707, '7'),
    (3708, '8'),
    (3709, '9'),
    (3712, ':'),
    (3713, ';'),
    (2241, '<'), // shared
    (3726, '='),
    (2242, '>'), // shared
    (3715, '?'),
    (2273, '@'), // shared
    (3301, 'A'),
    (3302, 'B'),
    (3303, 'C'),
    (3304, 'D'),
    (3305, 'E'),
    (3306, 'F'),
    (3307, 'G'),
    (3308, 'H'),
    (3309, 'I'),
    (3310, 'J'),
    (3311, 'K'),
    (3312, 'L'),
    (3313, 'M'),
    (3314, 'N'),
    (3315, 'O'),
    (3316, 'P'),
    (3317, 'Q'),
    (3318, 'R'),
    (3319, 'S'),
    (3320, 'T'),
    (3321, 'U'),
    (3322, 'V'),
    (3323, 'W'),
    (3324, 'X'),
    (3325, 'Y'),
    (3326, 'Z'),
    (2223, '['), // shared
    (804, '\\'), // shared
    (2224, ']'), // shared
    (2262, '^'), // shared
    (999, '_'),  // shared
    (3729, '`'),
    (3401, 'a'),
    (3402, 'b'),
    (3403, 'c'),
    (3404, 'd'),
    (3405, 'e'),
    (3406, 'f'),
    (3407, 'g'),
    (3408, 'h'),
    (3409, 'i'),
    (3410, 'j'),
    (3411, 'k'),
    (3412, 'l'),
    (3413, 'm'),
    (3414, 'n'),
    (3415, 'o'),
    (3416, 'p'),
    (3417, 'q'),
    (3418, 'r'),
    (3419, 's'),
    (3420, 't'),
    (3421, 'u'),
    (3422, 'v'),
    (3423, 'w'),
    (3424, 'x'),
    (3425, 'y'),
    (3426, 'z'),
    (2225, '{'), // shared
    (3716, '|'),
    (2226, '}'), // shared
    (2246, '~'), // shared
];

/// Mapping for Hershey Gothic Italian font
/// Italian Gothic uses 3801-3826 for uppercase and 3901-3926 for lowercase
const HERSHEY_GOTHIC_ITALIAN_MAP: &[(u32, char)] = &[
    (3699, ' '), // space
    (3714, '!'),
    (3728, '"'),
    (2275, '#'), // shared
    (3719, '$'),
    (2271, '%'), // shared
    (3718, '&'),
    (3717, '\''),
    (3721, '('),
    (3722, ')'),
    (3723, '*'),
    (3725, '+'),
    (3711, ','),
    (3724, '-'),
    (3710, '.'),
    (3720, '/'),
    (3700, '0'),
    (3701, '1'),
    (3702, '2'),
    (3703, '3'),
    (3704, '4'),
    (3705, '5'),
    (3706, '6'),
    (3707, '7'),
    (3708, '8'),
    (3709, '9'),
    (3712, ':'),
    (3713, ';'),
    (2241, '<'), // shared
    (3726, '='),
    (2242, '>'), // shared
    (3715, '?'),
    (2273, '@'), // shared
    (3801, 'A'),
    (3802, 'B'),
    (3803, 'C'),
    (3804, 'D'),
    (3805, 'E'),
    (3806, 'F'),
    (3807, 'G'),
    (3808, 'H'),
    (3809, 'I'),
    (3810, 'J'),
    (3811, 'K'),
    (3812, 'L'),
    (3813, 'M'),
    (3814, 'N'),
    (3815, 'O'),
    (3816, 'P'),
    (3817, 'Q'),
    (3818, 'R'),
    (3819, 'S'),
    (3820, 'T'),
    (3821, 'U'),
    (3822, 'V'),
    (3823, 'W'),
    (3824, 'X'),
    (3825, 'Y'),
    (3826, 'Z'),
    (2223, '['), // shared
    (804, '\\'), // shared
    (2224, ']'), // shared
    (2262, '^'), // shared
    (999, '_'),  // shared
    (3729, '`'),
    (3901, 'a'),
    (3902, 'b'),
    (3903, 'c'),
    (3904, 'd'),
    (3905, 'e'),
    (3906, 'f'),
    (3907, 'g'),
    (3908, 'h'),
    (3909, 'i'),
    (3910, 'j'),
    (3911, 'k'),
    (3912, 'l'),
    (3913, 'm'),
    (3914, 'n'),
    (3915, 'o'),
    (3916, 'p'),
    (3917, 'q'),
    (3918, 'r'),
    (3919, 's'),
    (3920, 't'),
    (3921, 'u'),
    (3922, 'v'),
    (3923, 'w'),
    (3924, 'x'),
    (3925, 'y'),
    (3926, 'z'),
    (2225, '{'), // shared
    (3716, '|'),
    (2226, '}'), // shared
    (2246, '~'), // shared
];

impl HersheyFont {
    /// Create a new empty Hershey font
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            glyphs: HashMap::new(),
            metrics: FontMetrics {
                // Hershey fonts have origin roughly centered, not at baseline
                // Raw coords: y=-12 at top of caps, y=+9 at baseline, y=+16 at descenders
                // After negation: y=+12 at top, y=-9 at baseline, y=-16 at descenders
                // We shift so baseline is at y=0: add 9 to all Y coords
                // Result: ascender at y=21, baseline at y=0, descender at y=-7
                units_per_em: 32.0,
                ascender: 21.0,       // Top of capitals (12 + 9 = 21 above baseline)
                descender: -7.0,      // Bottom of descenders (16 - 9 = 7 below baseline)
                x_height: Some(14.0), // lowercase height (~5 + 9 = 14)
                cap_height: Some(21.0),
                line_gap: 4.0,
            },
        }
    }

    /// Parse a Hershey font from JHF format data
    ///
    /// JHF format has one glyph per line. The glyph number is used to map to characters.
    pub fn from_jhf(name: &str, data: &str) -> Result<Self, FontError> {
        Self::from_jhf_with_mapping(name, data, HERSHEY_ROMAN_SIMPLEX_MAP)
    }

    /// Parse a Hershey font from JHF format using an offset from the base mapping
    ///
    /// Many Hershey fonts use the same character layout but with glyph numbers
    /// offset by a fixed amount (e.g., Duplex uses base + 2000).
    pub fn from_jhf_with_offset(name: &str, data: &str, offset: u32) -> Result<Self, FontError> {
        let mut font = Self::new(name);

        // Build offset mapping from base Roman Simplex mapping
        let glyph_to_char: HashMap<u32, char> = HERSHEY_ROMAN_SIMPLEX_MAP
            .iter()
            .map(|&(num, c)| (num + offset, c))
            .collect();

        for line in data.lines() {
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }

            if let Some((glyph_num, glyph)) = Self::parse_glyph_line(line)? {
                if let Some(&c) = glyph_to_char.get(&glyph_num) {
                    let mut mapped_glyph = glyph;
                    mapped_glyph.unicode = c;
                    font.glyphs.insert(c, mapped_glyph);
                }
            }
        }

        Ok(font)
    }

    /// Parse a Hershey font from JHF format using a custom character mapping
    pub fn from_jhf_with_mapping(
        name: &str,
        data: &str,
        mapping: &[(u32, char)],
    ) -> Result<Self, FontError> {
        let mut font = Self::new(name);

        // Build reverse mapping from glyph number to character
        let glyph_to_char: HashMap<u32, char> = mapping.iter().copied().collect();

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

        // If count is 0 or 1, that means only the margins exist (space character)
        // We check this before computing count - 1 to avoid underflow
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
        // Safe to subtract because we verified count > 1 above
        let max_pairs = count - 1;
        while i + 1 < coord_data.len() && i / 2 < max_pairs {
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
            // Convert from Hershey's Y-down to standard font Y-up coordinates
            // In Hershey: negative Y = up (ascenders), positive Y = down (descenders)
            // Raw y=9 is the baseline (bottom of capitals), which should become y=0
            // So we negate and then add 9: y = -raw_y + 9
            let raw_y = Self::decode_coord(y_byte) as f64;
            let y = -raw_y + 9.0; // Shift so baseline is at y=0

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

/// Built-in Hershey font data (public domain)
pub const HERSHEY_SIMPLEX_ROMAN: &str = include_str!("../../../fonts/hershey/simplex.jhf");
pub const HERSHEY_DUPLEX_ROMAN: &str = include_str!("../../../fonts/hershey/rowmand.jhf");
pub const HERSHEY_TRIPLEX_ROMAN: &str = include_str!("../../../fonts/hershey/rowmant.jhf");
pub const HERSHEY_SCRIPT_SIMPLEX: &str = include_str!("../../../fonts/hershey/scripts.jhf");
pub const HERSHEY_SCRIPT_COMPLEX: &str = include_str!("../../../fonts/hershey/scriptc.jhf");
pub const HERSHEY_GOTHIC_GERMAN_BOLD: &str = include_str!("../../../fonts/hershey/gothgbt.jhf");
pub const HERSHEY_GOTHIC_GERMAN: &str = include_str!("../../../fonts/hershey/gothgrt.jhf");
pub const HERSHEY_GOTHIC_ITALIAN: &str = include_str!("../../../fonts/hershey/gothitt.jhf");

/// Load the built-in Hershey Simplex font
pub fn load_simplex() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf("Hershey Simplex", HERSHEY_SIMPLEX_ROMAN)
}

/// Load the Hershey Roman Duplex font (double-stroke)
pub fn load_duplex() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf_with_offset("Hershey Duplex", HERSHEY_DUPLEX_ROMAN, 2000)
}

/// Load the Hershey Roman Triplex font (triple-stroke, more ornate)
pub fn load_triplex() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf_with_offset("Hershey Triplex", HERSHEY_TRIPLEX_ROMAN, 2500)
}

/// Load the Hershey Script Simplex font (cursive, single-stroke)
pub fn load_script_simplex() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf_with_mapping(
        "Hershey Script Simplex",
        HERSHEY_SCRIPT_SIMPLEX,
        HERSHEY_SCRIPT_SIMPLEX_MAP,
    )
}

/// Load the Hershey Script Complex font (cursive, double-stroke)
pub fn load_script_complex() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf_with_mapping(
        "Hershey Script Complex",
        HERSHEY_SCRIPT_COMPLEX,
        HERSHEY_SCRIPT_COMPLEX_MAP,
    )
}

/// Load the Hershey Gothic German Bold font (Fraktur-style, bold)
pub fn load_gothic_german_bold() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf_with_offset(
        "Hershey Gothic German Bold",
        HERSHEY_GOTHIC_GERMAN_BOLD,
        3000,
    )
}

/// Load the Hershey Gothic German font (Fraktur-style)
pub fn load_gothic_german() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf_with_mapping(
        "Hershey Gothic German",
        HERSHEY_GOTHIC_GERMAN,
        HERSHEY_GOTHIC_MAP,
    )
}

/// Load the Hershey Gothic Italian font (Fraktur-style, Italian variant)
pub fn load_gothic_italian() -> Result<HersheyFont, FontError> {
    HersheyFont::from_jhf_with_mapping(
        "Hershey Gothic Italian",
        HERSHEY_GOTHIC_ITALIAN,
        HERSHEY_GOTHIC_ITALIAN_MAP,
    )
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

    #[test]
    fn test_load_duplex() {
        let font = load_duplex().unwrap();
        assert_eq!(font.name(), "Hershey Duplex");
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('a'));
        assert!(font.has_glyph('0'));
    }

    #[test]
    fn test_load_triplex() {
        let font = load_triplex().unwrap();
        assert_eq!(font.name(), "Hershey Triplex");
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('a'));
        assert!(font.has_glyph('0'));
    }

    #[test]
    fn test_load_script_simplex() {
        let font = load_script_simplex().unwrap();
        assert_eq!(font.name(), "Hershey Script Simplex");
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('a'));
        assert!(font.has_glyph('0'));
    }

    #[test]
    fn test_load_script_complex() {
        let font = load_script_complex().unwrap();
        assert_eq!(font.name(), "Hershey Script Complex");
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('a'));
        assert!(font.has_glyph('0'));
    }

    #[test]
    fn test_load_gothic_german_bold() {
        let font = load_gothic_german_bold().unwrap();
        assert_eq!(font.name(), "Hershey Gothic German Bold");
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('a'));
        assert!(font.has_glyph('0'));
    }

    #[test]
    fn test_load_gothic_german() {
        let font = load_gothic_german().unwrap();
        assert_eq!(font.name(), "Hershey Gothic German");
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('a'));
        assert!(font.has_glyph('0'));
    }

    #[test]
    fn test_gothic_german_k_glyph() {
        let font = load_gothic_german().unwrap();
        let glyph = font.glyph('k').expect("Should have 'k' glyph");

        // The 'k' glyph should have multiple contours (Gothic fonts are ornate)
        // Gothic German 'k' has 13 contours with various segments
        assert!(
            !glyph.contours.is_empty(),
            "'k' should have at least one contour"
        );

        // Each contour should have segments
        for (i, contour) in glyph.contours.iter().enumerate() {
            assert!(
                !contour.segments.is_empty(),
                "Contour {} should have segments",
                i
            );
        }

        // Check that advance width is reasonable
        assert!(
            glyph.advance_width > 0.0,
            "'k' should have positive advance width"
        );
    }

    #[test]
    fn test_load_gothic_italian() {
        let font = load_gothic_italian().unwrap();
        assert_eq!(font.name(), "Hershey Gothic Italian");
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('a'));
        assert!(font.has_glyph('0'));
    }
}
