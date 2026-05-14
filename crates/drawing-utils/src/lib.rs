//! Reusable utilities for plotta-studio drawings
//!
//! This crate provides common drawing utilities:
//! - Hatching patterns for filling shapes
//! - Frame and title decorations
//! - [`Signature`] trait for signing artwork (with a built-in
//!   [`PlaceholderSignature`] for demos)

mod frame;
mod hatch;
mod signature;

pub use frame::{draw_frame, draw_frame_with_title, FrameOptions};
pub use hatch::{generate_hatch_lines, generate_hatch_lines_rect, HatchOptions};
pub use signature::{PlaceholderSignature, Signature};
