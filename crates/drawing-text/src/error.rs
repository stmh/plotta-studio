//! Error types for font operations

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during font operations
#[derive(Debug, Error)]
pub enum FontError {
    #[error("Failed to parse font: {0}")]
    ParseError(String),

    #[error("Invalid font format: {0}")]
    InvalidFormat(String),

    #[error("Glyph not found for character: {0}")]
    GlyphNotFound(char),

    #[error("I/O error reading {0}: {1}")]
    IoError(PathBuf, String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}
