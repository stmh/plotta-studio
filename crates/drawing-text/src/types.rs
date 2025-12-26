//! Re-exports of font types from drawing-core
//!
//! These types are defined in drawing-core to avoid cyclic dependencies.
//! This module re-exports them for backward compatibility.

pub use drawing_core::{
    Contour, ContourSegment, Font, FontMetrics, Glyph, PositionedGlyph, TextAlign, TextLayout,
    TextOptions, TextRenderer,
};
