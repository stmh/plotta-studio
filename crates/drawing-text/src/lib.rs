//! Single-line font support for plotta-studio
//!
//! This crate provides vector stroke font rendering for pen plotters.
//! It supports multiple font formats:
//!
//! - **Hershey fonts**: Classic public domain stroke fonts from 1967
//! - **VSF (Vector Stroke Font)**: Modern JSON-based format with bezier support
//! - **UFO 3**: Industry standard font format (future)
//!
//! # Example
//!
//! ```rust,ignore
//! use drawing_text::{HersheyFont, Font, TextRenderer, TextOptions};
//!
//! // Load built-in Hershey Simplex font
//! let font = drawing_text::hershey::load_simplex().unwrap();
//!
//! // Create text renderer
//! let renderer = TextRenderer::new();
//!
//! // Render text to strokes
//! let options = TextOptions::new(24.0).at((100.0, 100.0));
//! let layout = renderer.layout("Hello, World!", &font, &options);
//! let strokes = layout.to_strokes(drawing_core::Style::default(), 0.1);
//! ```

pub mod error;
pub mod font;
pub mod hershey;
pub mod manager;
pub mod svgfont;
pub mod types;
pub mod vsf;

// Re-export main types
pub use error::FontError;
pub use font::{Font, FontFormat, FontLoader, FontSource};
pub use hershey::{Hershey, HersheyFont};
pub use manager::{FontManager, DEFAULT_FONT_NAME};
pub use svgfont::SvgFont;
pub use types::{
    Contour, ContourSegment, FontMetrics, Glyph, PositionedGlyph, TextAlign, TextLayout,
    TextOptions, TextRenderer,
};
pub use vsf::VsfFont;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_text_renderer_creation() {
        let renderer = TextRenderer::new();
        assert_eq!(renderer.tolerance, 0.1);

        let renderer = renderer.with_tolerance(0.5);
        assert_eq!(renderer.tolerance, 0.5);
    }

    #[test]
    fn test_layout_empty_text() {
        let font: drawing_core::FontRef = Arc::new(hershey::load_simplex().unwrap());
        let renderer = TextRenderer::new();
        let options = TextOptions::new(24.0);
        let layout = renderer.layout("", font, &options);

        assert!(layout.glyphs.is_empty());
        assert!(layout.bounds.is_none());
        assert_eq!(layout.line_count, 1);
    }

    #[test]
    fn test_layout_simple_text() {
        let font: drawing_core::FontRef = Arc::new(hershey::load_simplex().unwrap());
        let renderer = TextRenderer::new();
        let options = TextOptions::new(24.0).at((100.0, 100.0));
        let layout = renderer.layout("ABC", font, &options);

        assert_eq!(layout.glyphs.len(), 3);
        assert!(layout.bounds.is_some());
    }

    #[test]
    fn test_layout_multiline() {
        let font: drawing_core::FontRef = Arc::new(hershey::load_simplex().unwrap());
        let renderer = TextRenderer::new();
        let options = TextOptions::new(24.0);
        let layout = renderer.layout("AB\nCD", font, &options);

        assert_eq!(layout.line_count, 2);
    }
}
