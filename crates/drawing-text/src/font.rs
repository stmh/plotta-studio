//! Font trait and related types
//!
//! The core Font trait is defined in drawing-core.
//! This module provides additional font loading infrastructure.

use crate::error::FontError;

// Re-export the Font trait from drawing-core
pub use drawing_core::Font;

// Re-export font-related types
pub use drawing_core::{FontMetrics, Glyph};

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
