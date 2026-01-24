//! Font trait and related types
//!
//! The core Font trait is defined in drawing-core.
//! This module provides format enumeration for font loading.

// Re-export the Font trait from drawing-core
pub use drawing_core::Font;

// Re-export font-related types
pub use drawing_core::{FontMetrics, Glyph};

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
