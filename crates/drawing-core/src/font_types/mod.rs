//! Core font types for text rendering
//!
//! These types are shared between drawing-core and drawing-text to avoid
//! cyclic dependencies.
//!
//! This module is organized into submodules:
//! - `metrics` - Font metrics for layout calculations
//! - `glyph` - Glyph and contour types for font geometry
//! - `font` - Font trait definition
//! - `layout` - Text layout and rendering types

mod font;
mod glyph;
mod layout;
mod metrics;

// Re-export all public types
pub use font::Font;
pub use glyph::{Contour, ContourSegment, Glyph};
pub use layout::{PositionedGlyph, TextAlign, TextLayout, TextOptions, TextRenderer};
pub use metrics::FontMetrics;
