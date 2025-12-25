//! Font trait and related types

use crate::error::FontError;
use crate::types::{FontMetrics, Glyph};

/// Trait for font implementations
pub trait Font: Send + Sync {
    /// Font family name
    fn name(&self) -> &str;

    /// Get a glyph by unicode character
    fn glyph(&self, c: char) -> Option<Glyph>;

    /// Get kerning adjustment between two characters (in font units)
    fn kerning(&self, _left: char, _right: char) -> f64 {
        0.0
    }

    /// Get font metrics
    fn metrics(&self) -> FontMetrics;

    /// Check if font has a glyph for character
    fn has_glyph(&self, c: char) -> bool {
        self.glyph(c).is_some()
    }

    /// Get all available characters
    fn available_chars(&self) -> Vec<char>;
}

/// Font source for loading
#[derive(Debug, Clone)]
pub enum FontSource {
    /// Load from file path
    File(std::path::PathBuf),
    /// Load from bytes with format hint
    Bytes { data: Vec<u8>, format: FontFormat },
    /// Load from embedded data
    Embedded {
        name: &'static str,
        data: &'static str,
    },
}

/// Supported font formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFormat {
    /// Hershey font format (.jhf or inline)
    Hershey,
    /// UFO 3 format (directory)
    Ufo,
    /// VSF JSON format
    Vsf,
    /// SVG font
    SvgFont,
}

/// Trait for font loaders
pub trait FontLoader: Send + Sync {
    /// Check if this loader can handle the given source
    fn can_load(&self, source: &FontSource) -> bool;

    /// Load a font from the source
    fn load(&self, source: &FontSource) -> Result<Box<dyn Font>, FontError>;

    /// Supported format name (for error messages)
    fn format_name(&self) -> &'static str;
}
