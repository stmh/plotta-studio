//! Font metrics for layout calculations

use serde::{Deserialize, Serialize};

/// Font metrics for layout calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetrics {
    /// Units per em (typically 1000 or 2048)
    pub units_per_em: f64,
    /// Distance from baseline to top of tallest glyph
    pub ascender: f64,
    /// Distance from baseline to bottom of deepest glyph (negative)
    pub descender: f64,
    /// Height of lowercase 'x'
    pub x_height: Option<f64>,
    /// Height of capital letters
    pub cap_height: Option<f64>,
    /// Additional space between lines
    pub line_gap: f64,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            units_per_em: 1000.0,
            ascender: 800.0,
            descender: -200.0,
            x_height: Some(500.0),
            cap_height: Some(700.0),
            line_gap: 100.0,
        }
    }
}

impl FontMetrics {
    /// Calculate line height (ascender - descender + line_gap)
    pub fn line_height(&self) -> f64 {
        self.ascender - self.descender + self.line_gap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_metrics_line_height() {
        let metrics = FontMetrics::default();
        assert_eq!(metrics.line_height(), 1100.0); // 800 - (-200) + 100
    }
}
