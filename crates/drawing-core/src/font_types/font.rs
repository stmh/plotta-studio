//! Font trait definition

use super::glyph::Glyph;
use super::metrics::FontMetrics;

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
