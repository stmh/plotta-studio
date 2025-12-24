//! Error types for plotter operations

use thiserror::Error;

/// Errors that can occur during plotter operations
#[derive(Error, Debug)]
pub enum PlotterError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Plotter error: {0}")]
    Device(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
