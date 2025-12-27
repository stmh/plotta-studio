//! Reusable utilities for plotta-studio drawings
//!
//! This crate provides common drawing utilities:
//! - Hatching patterns for filling shapes
//! - Frame and title decorations

mod frame;
mod hatch;

pub use frame::{draw_frame, draw_frame_with_title, FrameOptions};
pub use hatch::{generate_hatch_lines, generate_hatch_lines_rect, HatchOptions};
