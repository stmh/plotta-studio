//! Motion planning for smooth plotter movement
//!
//! This module implements velocity planning and trapezoidal motion profiles
//! to eliminate harsh motor noise when plotting curves. Instead of constant
//! velocity motion (which creates sudden direction changes at segment boundaries),
//! this planner calculates appropriate corner velocities and acceleration profiles.
//!
//! ## Key Concepts
//!
//! - **Junction velocity**: The velocity at which the plotter transitions from one
//!   segment to the next. Calculated based on the angle between segments.
//! - **Trapezoidal profile**: Each segment may have acceleration, cruise, and
//!   deceleration phases to smoothly transition between junction velocities.
//! - **Lookahead**: The planner looks at all segments to compute optimal velocities,
//!   working both forward (limited by max acceleration) and backward (limited by
//!   deceleration into corners).
//!
//! ## SM Command Integration (Recommended)
//!
//! Following the AxiDraw Python driver approach, we use SM (Stepper Move) commands
//! with time-slice interpolation for smooth motion:
//! ```text
//! SM,duration_ms,axis1_steps,axis2_steps
//! ```
//! - Motion is broken into 25ms time slices
//! - Each slice has constant velocity (computed from trapezoid profile)
//! - Software handles acceleration by varying velocity between slices
//!
//! ## LM Command Integration (Legacy)
//!
//! The EBB's LM command supports acceleration but is complex to use correctly:
//! ```text
//! LM,Rate1,Steps1,Accel1,Rate2,Steps2,Accel2[,Clear]
//! ```
//! - Rate: step rate factor (Rate = 85899.35 * steps_per_second)
//! - Accel: change in Rate every 40us (positive for acceleration, negative for decel)
//!
//! ## Module Organization
//!
//! - `config` - Constants and configuration types
//! - `segment` - Motion segments and junction velocity calculations
//! - `profile` - Motion profiles and the motion planner
//! - `sm_command` - SM commands (recommended approach)
//! - `lm_command` - LM commands (legacy approach)

mod config;
mod lm_command;
mod profile;
mod segment;
mod sm_command;

// Re-export configuration
pub use config::{
    MotionConfig, DEFAULT_ACCEL_PEN_DOWN, DEFAULT_ACCEL_PEN_UP, DEFAULT_JUNCTION_DEVIATION,
};

// Re-export segment types and functions
pub use segment::{calculate_junction_velocity, MotionSegment};

// Re-export profile types
pub use profile::{MotionPlanner, MotionProfile};

// Re-export SM command types (recommended)
pub use sm_command::{generate_sm_commands, SmCommand, SmPlannedMove};

// Re-export LM command types (legacy)
pub use lm_command::{acceleration_to_accel_param, velocity_to_rate, LmCommand, PlannedMove};
