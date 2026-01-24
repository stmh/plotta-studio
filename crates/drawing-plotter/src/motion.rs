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

use drawing_core::Point;

/// LM command Rate calculation constant
/// Rate = 2^31 / 25000 * frequency_hz = 85899.3459 * frequency_hz
/// (25000 Hz = 40us ISR interval)
pub const RATE_FACTOR: f64 = 85899.3459;

/// ISR interval in seconds (40 microseconds)
pub const ISR_INTERVAL_SECS: f64 = 40.0e-6;

/// Default junction deviation for corner velocity calculation (mm)
/// This controls how much the path can deviate at corners.
/// Smaller values = slower corners, larger = faster but less accurate.
///
/// Derived from Python driver's cornering parameter:
/// delta = cornering / 5000 (in inches), where cornering = 10.0
/// So delta = 10.0 / 5000 = 0.002 inches = 0.0508 mm
pub const DEFAULT_JUNCTION_DEVIATION: f64 = 0.05;

/// Default acceleration for pen-down moves (mm/s²)
/// From Python driver: accel_rate = 40.0 inches/s² = 1016 mm/s²
pub const DEFAULT_ACCEL_PEN_DOWN: f64 = 1016.0;

/// Default acceleration for pen-up moves (mm/s²)
/// From Python driver: accel_rate_pu = 60.0 inches/s² = 1524 mm/s²
pub const DEFAULT_ACCEL_PEN_UP: f64 = 1524.0;

/// Time slice duration for SM command interpolation (ms)
/// Matches the Python AxiDraw driver's time_slice = 0.025 seconds
pub const TIME_SLICE_MS: u32 = 25;

/// Minimum step rate to avoid motor resonance (steps per ms)
/// If step rate falls below this, zero out the steps for that axis.
/// From Python driver: rate < 0.002 steps/ms triggers this guard.
pub const MIN_STEP_RATE: f64 = 0.002;

/// Maximum step rate (steps per ms) - 25kHz = 25 steps/ms
/// SM command duration will be increased if rate exceeds this.
pub const MAX_STEP_RATE: f64 = 25.0;

/// Command buffer lead time (ms)
/// Sleep for (duration - BUFFER_LEAD_MS) to ensure next command arrives
/// before current move completes. Matches Python driver's 30ms buffer.
pub const BUFFER_LEAD_MS: u32 = 30;

/// Configuration for motion planning
#[derive(Debug, Clone)]
pub struct MotionConfig {
    /// Maximum velocity for pen-down moves (mm/s)
    pub max_velocity: f64,
    /// Maximum acceleration (mm/s^2)
    pub max_acceleration: f64,
    /// Junction deviation for corner velocity calculation (mm)
    /// Controls the trade-off between speed and accuracy at corners.
    pub junction_deviation: f64,
    /// Steps per mm (AxiDraw: 80 steps/mm)
    pub steps_per_mm: f64,
}

impl Default for MotionConfig {
    fn default() -> Self {
        Self {
            max_velocity: 25.0,                       // mm/s (conservative default)
            max_acceleration: DEFAULT_ACCEL_PEN_DOWN, // mm/s^2 (matches Python driver)
            junction_deviation: DEFAULT_JUNCTION_DEVIATION,
            steps_per_mm: 80.0, // AxiDraw default
        }
    }
}

impl MotionConfig {
    /// Create a motion config with custom velocity and acceleration
    pub fn new(max_velocity: f64, max_acceleration: f64) -> Self {
        Self {
            max_velocity,
            max_acceleration,
            ..Default::default()
        }
    }

    /// Set junction deviation
    pub fn with_junction_deviation(mut self, deviation: f64) -> Self {
        self.junction_deviation = deviation;
        self
    }
}

/// A motion segment representing movement between two points
#[derive(Debug, Clone)]
pub struct MotionSegment {
    /// Start point
    pub start: Point,
    /// End point
    pub end: Point,
    /// Segment length in mm
    pub length: f64,
    /// Unit direction vector
    pub direction: Point,
    /// Entry velocity (mm/s) - velocity at start of segment
    pub entry_velocity: f64,
    /// Exit velocity (mm/s) - velocity at end of segment
    pub exit_velocity: f64,
    /// Maximum velocity for this segment (mm/s)
    pub max_velocity: f64,
}

impl MotionSegment {
    /// Create a new motion segment from two points
    pub fn new(start: Point, end: Point, max_velocity: f64) -> Self {
        let delta = end - start;
        let length = delta.length();
        let direction = if length > 1e-9 {
            Point::new(delta.x / length, delta.y / length)
        } else {
            Point::new(1.0, 0.0)
        };

        Self {
            start,
            end,
            length,
            direction,
            entry_velocity: 0.0,
            exit_velocity: 0.0,
            max_velocity,
        }
    }

    /// Calculate the angle between this segment and the next (in radians)
    /// Returns 0 for collinear segments, PI for 180-degree turns
    pub fn angle_to(&self, next: &MotionSegment) -> f64 {
        // Dot product of unit direction vectors gives cos(angle)
        let dot = self.direction.x * next.direction.x + self.direction.y * next.direction.y;
        // Clamp to handle floating point errors
        let dot_clamped = dot.clamp(-1.0, 1.0);
        // acos gives the angle between vectors (0 = same direction, PI = opposite)
        dot_clamped.acos()
    }
}

/// Calculate the maximum junction velocity between two segments
///
/// Based on Grbl's junction velocity calculation:
/// - For straight segments (angle ~= 0), junction velocity can be max
/// - For sharp corners (angle ~= PI), junction velocity approaches 0
/// - The junction_deviation parameter controls the trade-off
///
/// Formula: v_junction = sqrt(2 * acceleration * deviation_distance)
/// where deviation_distance depends on the angle
pub fn calculate_junction_velocity(
    current: &MotionSegment,
    next: &MotionSegment,
    config: &MotionConfig,
) -> f64 {
    let angle = current.angle_to(next);

    // For very small angles (nearly straight), allow max velocity
    if angle < 0.01 {
        return config.max_velocity;
    }

    // For angles approaching 180 degrees (reversal), velocity should be very low
    if angle > std::f64::consts::PI - 0.01 {
        return 0.0;
    }

    // Calculate the junction velocity using the GRBL cornering algorithm
    // Reference: https://onehossshay.wordpress.com/2011/09/24/improving_grbl_cornering_algorithm/
    //
    // The Python AxiDraw driver uses:
    //   cosine_factor = -dot(v1, v2) = -cos(angle) (since vectors point in direction of travel)
    //   root_factor = sqrt((1 - cosine_factor) / 2) = sqrt((1 + cos(angle)) / 2) = cos(angle/2)
    //   rfactor = delta * root_factor / (1 - root_factor)
    //   vjunction = sqrt(accel * rfactor)
    //
    // Note: Our angle_to() returns the deflection angle (0 = straight, PI = reversal),
    // so we use cos(angle/2) directly.
    let half_angle = angle / 2.0;
    let cos_half = half_angle.cos();

    // Avoid division by zero when cos_half approaches 1 (very small angles)
    let denominator = 1.0 - cos_half;
    if denominator < 0.0001 {
        return config.max_velocity;
    }

    // Junction deviation formula from GRBL/Python driver
    // rfactor = delta * cos(angle/2) / (1 - cos(angle/2))
    let rfactor = config.junction_deviation * cos_half / denominator;

    // Maximum junction velocity: v = sqrt(acceleration * rfactor)
    // Note: Python uses sqrt(accel * rfactor), not sqrt(2 * accel * rfactor)
    let v_junction = (config.max_acceleration * rfactor).sqrt();

    // Clamp to max velocity and the minimum of both segments' max velocities
    v_junction
        .min(config.max_velocity)
        .min(current.max_velocity)
        .min(next.max_velocity)
}

/// Calculate the maximum velocity achievable over a distance given start velocity and acceleration
///
/// Uses kinematic equation: v^2 = v0^2 + 2*a*s
/// Returns the velocity at the end of accelerating over distance `s`
pub fn velocity_after_acceleration(start_velocity: f64, acceleration: f64, distance: f64) -> f64 {
    let v_squared = start_velocity * start_velocity + 2.0 * acceleration * distance;
    if v_squared > 0.0 {
        v_squared.sqrt()
    } else {
        0.0
    }
}

/// Calculate the distance needed to change velocity
///
/// Uses kinematic equation: s = (v^2 - v0^2) / (2*a)
/// Returns the absolute distance (always positive)
pub fn distance_to_velocity(start_velocity: f64, end_velocity: f64, acceleration: f64) -> f64 {
    if acceleration.abs() < 1e-9 {
        return f64::INFINITY;
    }
    ((end_velocity * end_velocity - start_velocity * start_velocity) / (2.0 * acceleration)).abs()
}

/// Calculate the time to change velocity
///
/// Uses kinematic equation: t = (v - v0) / a
pub fn time_to_velocity(start_velocity: f64, end_velocity: f64, acceleration: f64) -> f64 {
    if acceleration.abs() < 1e-9 {
        if (end_velocity - start_velocity).abs() < 1e-9 {
            return 0.0;
        }
        return f64::INFINITY;
    }
    (end_velocity - start_velocity) / acceleration
}

/// A planned motion profile for a segment
#[derive(Debug, Clone)]
pub struct MotionProfile {
    /// Entry velocity (mm/s)
    pub entry_velocity: f64,
    /// Exit velocity (mm/s)
    pub exit_velocity: f64,
    /// Cruise velocity - peak velocity in the middle (mm/s)
    pub cruise_velocity: f64,
    /// Distance of acceleration phase (mm)
    pub accel_distance: f64,
    /// Distance of cruise phase (mm)
    pub cruise_distance: f64,
    /// Distance of deceleration phase (mm)
    pub decel_distance: f64,
    /// Total segment length (mm)
    pub total_distance: f64,
    /// Acceleration rate (mm/s^2), always positive
    pub acceleration: f64,
}

impl MotionProfile {
    /// Calculate the motion profile for a segment
    ///
    /// Given entry and exit velocities and segment length, computes
    /// the acceleration, cruise, and deceleration phases.
    pub fn calculate(
        entry_velocity: f64,
        exit_velocity: f64,
        max_velocity: f64,
        distance: f64,
        max_acceleration: f64,
    ) -> Self {
        // Distance needed to accelerate from entry to max velocity
        let accel_dist = distance_to_velocity(entry_velocity, max_velocity, max_acceleration);

        // Distance needed to decelerate from max velocity to exit
        let decel_dist = distance_to_velocity(max_velocity, exit_velocity, max_acceleration);

        // Check if we can reach cruise velocity
        if accel_dist + decel_dist <= distance {
            // Full trapezoidal profile: accel -> cruise -> decel
            Self {
                entry_velocity,
                exit_velocity,
                cruise_velocity: max_velocity,
                accel_distance: accel_dist,
                cruise_distance: distance - accel_dist - decel_dist,
                decel_distance: decel_dist,
                total_distance: distance,
                acceleration: max_acceleration,
            }
        } else {
            // Triangular profile: can't reach cruise velocity
            // Find the peak velocity we can reach
            // Using: v_peak^2 = v_entry^2 + 2*a*d_accel
            // and:   v_peak^2 = v_exit^2 + 2*a*d_decel
            // where: d_accel + d_decel = distance
            //
            // Solving: v_peak = sqrt((v_entry^2 + v_exit^2 + 2*a*d) / 2)
            let v_entry_sq = entry_velocity * entry_velocity;
            let v_exit_sq = exit_velocity * exit_velocity;
            let peak_velocity_sq =
                (v_entry_sq + v_exit_sq + 2.0 * max_acceleration * distance) / 2.0;

            let peak_velocity = if peak_velocity_sq > 0.0 {
                peak_velocity_sq.sqrt()
            } else {
                entry_velocity.max(exit_velocity)
            };

            let accel_d = distance_to_velocity(entry_velocity, peak_velocity, max_acceleration);
            let decel_d = distance - accel_d;

            Self {
                entry_velocity,
                exit_velocity,
                cruise_velocity: peak_velocity,
                accel_distance: accel_d.max(0.0),
                cruise_distance: 0.0,
                decel_distance: decel_d.max(0.0),
                total_distance: distance,
                acceleration: max_acceleration,
            }
        }
    }

    /// Check if this is a triangular profile (no cruise phase)
    pub fn is_triangular(&self) -> bool {
        self.cruise_distance < 1e-9
    }

    /// Calculate total time for this motion profile
    pub fn total_time(&self) -> f64 {
        let accel_time =
            time_to_velocity(self.entry_velocity, self.cruise_velocity, self.acceleration);
        let decel_time =
            time_to_velocity(self.cruise_velocity, self.exit_velocity, self.acceleration);

        let cruise_time = if self.cruise_velocity > 1e-9 {
            self.cruise_distance / self.cruise_velocity
        } else {
            0.0
        };

        accel_time.abs() + cruise_time + decel_time.abs()
    }
}

/// Motion planner for computing optimal velocities across a path
pub struct MotionPlanner {
    config: MotionConfig,
}

impl MotionPlanner {
    /// Create a new motion planner with the given configuration
    pub fn new(config: MotionConfig) -> Self {
        Self { config }
    }

    /// Create a motion planner with default configuration
    pub fn with_defaults() -> Self {
        Self::new(MotionConfig::default())
    }

    /// Get the configuration
    pub fn config(&self) -> &MotionConfig {
        &self.config
    }

    /// Plan velocities for a sequence of points
    ///
    /// This performs the full planning algorithm:
    /// 1. Create segments from points
    /// 2. Calculate junction velocities
    /// 3. Forward pass: limit entry velocities by acceleration from previous segment
    /// 4. Backward pass: limit exit velocities by deceleration into next segment
    ///
    /// Returns motion segments with entry/exit velocities set
    pub fn plan(&self, points: &[Point]) -> Vec<MotionSegment> {
        if points.len() < 2 {
            return Vec::new();
        }

        // Create segments
        let mut segments: Vec<MotionSegment> = points
            .windows(2)
            .map(|w| MotionSegment::new(w[0], w[1], self.config.max_velocity))
            .filter(|s| s.length > 1e-9) // Filter out zero-length segments
            .collect();

        if segments.is_empty() {
            return Vec::new();
        }

        // Calculate junction velocities (max velocity at transitions)
        let mut junction_velocities = Vec::with_capacity(segments.len() + 1);

        // First junction: start from zero (or could be configurable)
        junction_velocities.push(0.0);

        // Middle junctions: based on angle between segments
        for i in 0..segments.len() - 1 {
            let v_junction =
                calculate_junction_velocity(&segments[i], &segments[i + 1], &self.config);
            junction_velocities.push(v_junction);
        }

        // Last junction: end at zero (or could be configurable)
        junction_velocities.push(0.0);

        // Forward pass: compute max entry velocity based on accelerating from previous junction
        for i in 0..segments.len() {
            let max_entry = junction_velocities[i];
            let max_exit = junction_velocities[i + 1];

            // Entry velocity is limited by junction velocity
            segments[i].entry_velocity = max_entry;

            // Exit velocity is limited by what we can accelerate to
            let achievable_exit = velocity_after_acceleration(
                max_entry,
                self.config.max_acceleration,
                segments[i].length,
            );
            segments[i].exit_velocity = achievable_exit.min(max_exit);
        }

        // Backward pass: ensure we can decelerate to each junction
        for i in (0..segments.len()).rev() {
            let current_exit = segments[i].exit_velocity;

            // Check if we need to limit entry velocity to achieve this exit
            let required_entry = velocity_after_acceleration(
                current_exit,
                self.config.max_acceleration,
                segments[i].length,
            );

            if required_entry < segments[i].entry_velocity {
                segments[i].entry_velocity = required_entry;
            }

            // Propagate to previous segment's exit if needed
            if i > 0 && segments[i].entry_velocity < segments[i - 1].exit_velocity {
                segments[i - 1].exit_velocity = segments[i].entry_velocity;
            }
        }

        segments
    }

    /// Generate motion profiles for planned segments
    pub fn generate_profiles(&self, segments: &[MotionSegment]) -> Vec<MotionProfile> {
        segments
            .iter()
            .map(|seg| {
                MotionProfile::calculate(
                    seg.entry_velocity,
                    seg.exit_velocity,
                    seg.max_velocity.min(self.config.max_velocity),
                    seg.length,
                    self.config.max_acceleration,
                )
            })
            .collect()
    }
}

/// Convert velocity (mm/s) to LM Rate parameter
pub fn velocity_to_rate(velocity_mm_s: f64, steps_per_mm: f64) -> u32 {
    let steps_per_sec = velocity_mm_s * steps_per_mm;
    (RATE_FACTOR * steps_per_sec).round() as u32
}

/// Convert acceleration (mm/s^2) to LM Accel parameter
///
/// The Accel parameter is the change in Rate per ISR interval (40us)
pub fn acceleration_to_accel_param(accel_mm_s2: f64, steps_per_mm: f64) -> i32 {
    // Acceleration in steps/s^2
    let accel_steps_s2 = accel_mm_s2 * steps_per_mm;

    // Rate change per second = RATE_FACTOR * accel_steps_s2 / steps_per_sec
    // But we need rate change per ISR interval (40us)
    // Accel = RATE_FACTOR * accel_steps_s2 * ISR_INTERVAL
    let accel_param = RATE_FACTOR * accel_steps_s2 * ISR_INTERVAL_SECS;
    accel_param.round() as i32
}

/// An LM command for the EBB
///
/// LM command format: LM,Rate1,Steps1,Accel1,Rate2,Steps2,Accel2[,Clear]
#[derive(Debug, Clone)]
pub struct LmCommand {
    /// Rate for axis 1 (always positive)
    pub rate1: u32,
    /// Steps for axis 1 (sign indicates direction)
    pub steps1: i32,
    /// Acceleration for axis 1 (can be negative for deceleration)
    pub accel1: i32,
    /// Rate for axis 2 (always positive)
    pub rate2: u32,
    /// Steps for axis 2 (sign indicates direction)
    pub steps2: i32,
    /// Acceleration for axis 2 (can be negative for deceleration)
    pub accel2: i32,
    /// Duration to wait after sending command (ms)
    pub duration_ms: u32,
    /// Clear accumulator before move (0=none, 1=axis1, 2=axis2, 3=both)
    ///
    /// When starting from rate=0 with acceleration, the accumulator should
    /// be cleared to avoid artifacts from previous moves.
    pub clear: u8,
}

impl LmCommand {
    /// Create a new LM command
    pub fn new(
        rate1: u32,
        steps1: i32,
        accel1: i32,
        rate2: u32,
        steps2: i32,
        accel2: i32,
        duration_ms: u32,
    ) -> Self {
        Self {
            rate1,
            steps1,
            accel1,
            rate2,
            steps2,
            accel2,
            duration_ms,
            clear: 0,
        }
    }

    /// Create an LM command with accumulator clear
    pub fn with_clear(mut self, clear: u8) -> Self {
        self.clear = clear;
        self
    }

    /// Check if this command is valid for the EBB
    ///
    /// From EBB docs: "motion must be possible on at least one axis"
    /// - If Steps is nonzero, then Rate or Accel must be nonzero
    /// - Rate must not exceed max step rate (25kHz = Rate of ~2.1 billion)
    pub fn is_valid(&self) -> bool {
        // If axis 1 has steps, it needs rate or accel
        if self.steps1 != 0 && self.rate1 == 0 && self.accel1 == 0 {
            return false;
        }
        // If axis 2 has steps, it needs rate or accel
        if self.steps2 != 0 && self.rate2 == 0 && self.accel2 == 0 {
            return false;
        }
        // At least one axis must have motion
        if self.steps1 == 0 && self.steps2 == 0 {
            return false;
        }
        true
    }

    /// Check if this command represents no motion (can be skipped)
    pub fn is_empty(&self) -> bool {
        self.steps1 == 0 && self.steps2 == 0
    }

    /// Format as EBB command string
    pub fn to_command_string(&self) -> String {
        if self.clear > 0 {
            format!(
                "LM,{},{},{},{},{},{},{}",
                self.rate1,
                self.steps1,
                self.accel1,
                self.rate2,
                self.steps2,
                self.accel2,
                self.clear
            )
        } else {
            format!(
                "LM,{},{},{},{},{},{}",
                self.rate1, self.steps1, self.accel1, self.rate2, self.steps2, self.accel2
            )
        }
    }

    /// Create a constant-velocity LM command (no acceleration)
    pub fn constant_velocity(
        steps_x: i32,
        steps_y: i32,
        velocity_mm_s: f64,
        distance_mm: f64,
        _steps_per_mm: f64,
    ) -> Self {
        // Apply CoreXY transform
        let steps_axis1 = steps_x + steps_y;
        let steps_axis2 = steps_x - steps_y;

        // Calculate duration
        let duration_secs = if velocity_mm_s > 1e-9 {
            distance_mm / velocity_mm_s
        } else {
            0.0
        };
        let duration_ms = (duration_secs * 1000.0).round() as u32;

        // Calculate step frequencies
        let freq_axis1 = if duration_secs > 1e-9 {
            (steps_axis1.abs() as f64) / duration_secs
        } else {
            0.0
        };
        let freq_axis2 = if duration_secs > 1e-9 {
            (steps_axis2.abs() as f64) / duration_secs
        } else {
            0.0
        };

        // Calculate Rate values (clamp to valid range)
        let rate1 = (RATE_FACTOR * freq_axis1)
            .round()
            .clamp(0.0, u32::MAX as f64) as u32;
        let rate2 = (RATE_FACTOR * freq_axis2)
            .round()
            .clamp(0.0, u32::MAX as f64) as u32;

        Self {
            rate1,
            steps1: steps_axis1,
            accel1: 0,
            rate2,
            steps2: steps_axis2,
            accel2: 0,
            duration_ms: duration_ms.max(1),
            clear: 0,
        }
    }
}

// ============================================================================
// SM Command - Time-slice based motion (recommended approach)
// ============================================================================

/// An SM (Stepper Move) command for the EBB
///
/// SM command format: SM,duration_ms,axis1_steps,axis2_steps
///
/// This is the recommended approach for motion control, matching the
/// Python AxiDraw driver. Instead of using LM with acceleration parameters,
/// we break motion into many small constant-velocity SM commands.
///
/// The EBB uses CoreXY kinematics:
/// - axis1 = X + Y (in steps)
/// - axis2 = X - Y (in steps)
#[derive(Debug, Clone)]
pub struct SmCommand {
    /// Duration of this move segment (ms)
    pub duration_ms: u32,
    /// Steps for axis 1 (X+Y in CoreXY, sign indicates direction)
    pub steps_axis1: i32,
    /// Steps for axis 2 (X-Y in CoreXY, sign indicates direction)
    pub steps_axis2: i32,
}

impl SmCommand {
    /// Create a new SM command
    pub fn new(duration_ms: u32, steps_axis1: i32, steps_axis2: i32) -> Self {
        Self {
            duration_ms,
            steps_axis1,
            steps_axis2,
        }
    }

    /// Create an SM command from X/Y steps (applies CoreXY transform)
    pub fn from_xy_steps(duration_ms: u32, steps_x: i32, steps_y: i32) -> Self {
        Self {
            duration_ms,
            steps_axis1: steps_x + steps_y,
            steps_axis2: steps_x - steps_y,
        }
    }

    /// Format as EBB command string
    pub fn to_command_string(&self) -> String {
        format!(
            "SM,{},{},{}",
            self.duration_ms, self.steps_axis1, self.steps_axis2
        )
    }

    /// Check if this command represents no motion (can be skipped)
    pub fn is_empty(&self) -> bool {
        self.steps_axis1 == 0 && self.steps_axis2 == 0
    }

    /// Get the actual X/Y steps this command will execute
    ///
    /// Reverses the CoreXY transform to get back X/Y steps from axis steps.
    /// Use after `validate_and_adjust()` to get the actual steps that will be sent.
    pub fn xy_steps(&self) -> (i32, i32) {
        // Reverse CoreXY: x = (axis1 + axis2) / 2, y = (axis1 - axis2) / 2
        let steps_x = (self.steps_axis1 + self.steps_axis2) / 2;
        let steps_y = (self.steps_axis1 - self.steps_axis2) / 2;
        (steps_x, steps_y)
    }

    /// Calculate time to sleep before sending next command
    ///
    /// Returns the duration minus the buffer lead time, ensuring the next
    /// command arrives before this move completes. Returns 0 for very short moves.
    pub fn sleep_time_ms(&self) -> u32 {
        if self.duration_ms > BUFFER_LEAD_MS + 20 {
            self.duration_ms - BUFFER_LEAD_MS
        } else {
            // For short moves, don't sleep (command will queue in EBB buffer)
            0
        }
    }

    /// Validate and adjust this command for motor safety
    ///
    /// - Increases duration if step rate exceeds MAX_STEP_RATE
    /// - Zeros out steps if step rate is below MIN_STEP_RATE (prevents resonance)
    ///
    /// Returns true if the command is valid for execution, false if it should be skipped.
    pub fn validate_and_adjust(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        // Calculate step rates (steps per ms)
        let mut rate1 = self.steps_axis1.abs() as f64 / self.duration_ms as f64;
        let mut rate2 = self.steps_axis2.abs() as f64 / self.duration_ms as f64;

        // Check for overspeed and increase duration if needed
        while rate1 >= MAX_STEP_RATE || rate2 >= MAX_STEP_RATE {
            self.duration_ms += 1;
            rate1 = self.steps_axis1.abs() as f64 / self.duration_ms as f64;
            rate2 = self.steps_axis2.abs() as f64 / self.duration_ms as f64;
        }

        // Check for underspeed (motor resonance) and zero out slow axes
        if rate1 > 0.0 && rate1 < MIN_STEP_RATE {
            self.steps_axis1 = 0;
        }
        if rate2 > 0.0 && rate2 < MIN_STEP_RATE {
            self.steps_axis2 = 0;
        }

        // Return false if both axes are now zero
        !self.is_empty()
    }
}

/// A sequence of SM commands representing a motion-planned move
#[derive(Debug, Clone)]
pub struct SmPlannedMove {
    /// The SM commands to execute in sequence
    pub commands: Vec<SmCommand>,
    /// Total duration of the move (ms)
    pub total_duration_ms: u32,
    /// Start point (mm)
    pub start: Point,
    /// End point (mm) - the requested endpoint (may differ slightly from actual due to rounding)
    pub end: Point,
    /// Actual X steps sent (sum of all commands after validation/adjustment)
    /// Use this to calculate actual position to avoid cumulative drift
    pub actual_steps_x: i32,
    /// Actual Y steps sent (sum of all commands after validation/adjustment)
    /// Use this to calculate actual position to avoid cumulative drift
    pub actual_steps_y: i32,
}

impl SmPlannedMove {
    /// Create an empty planned move
    pub fn empty(start: Point, end: Point) -> Self {
        Self {
            commands: Vec::new(),
            total_duration_ms: 0,
            start,
            end,
            actual_steps_x: 0,
            actual_steps_y: 0,
        }
    }

    /// Check if this planned move has any commands
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Calculate the actual endpoint based on start position and actual steps sent
    ///
    /// This should be used instead of `end` to avoid cumulative position drift
    /// from rounding errors.
    pub fn actual_end(&self, steps_per_mm: f64) -> Point {
        Point::new(
            self.start.x + (self.actual_steps_x as f64 / steps_per_mm),
            self.start.y + (self.actual_steps_y as f64 / steps_per_mm),
        )
    }
}

/// Generate SM commands for a motion profile using time-slice interpolation
///
/// This is the core function that converts a velocity profile into a sequence
/// of constant-velocity SM commands, matching the Python AxiDraw driver approach.
///
/// The motion is broken into TIME_SLICE_MS intervals. At each interval boundary,
/// we calculate the instantaneous velocity from the trapezoid profile and generate
/// an SM command for that slice.
pub fn generate_sm_commands(
    profile: &MotionProfile,
    start: Point,
    end: Point,
    steps_per_mm: f64,
) -> SmPlannedMove {
    let delta = end - start;
    let distance = delta.length();

    // Skip tiny moves
    if distance < 0.001 || profile.total_distance < 0.001 {
        return SmPlannedMove::empty(start, end);
    }

    // Unit direction vector
    let dir_x = delta.x / distance;
    let dir_y = delta.y / distance;

    // Total time for the profile
    let total_time_secs = profile.total_time();
    let total_time_ms = (total_time_secs * 1000.0).round() as u32;

    // For very short moves, use a single SM command
    if total_time_ms <= TIME_SLICE_MS * 2 {
        return generate_single_sm_command(profile, start, end, steps_per_mm);
    }

    // Calculate number of time slices
    let num_slices = (total_time_ms / TIME_SLICE_MS).max(1);
    let slice_duration_ms = TIME_SLICE_MS;

    let mut commands = Vec::with_capacity(num_slices as usize);
    let mut total_duration_ms = 0u32;

    // Track cumulative distance and steps for velocity profile interpolation
    let mut cumulative_distance = 0.0;
    let mut cumulative_steps_x = 0i32;
    let mut cumulative_steps_y = 0i32;

    // Track actual steps sent (after validation/adjustment) to avoid position drift
    let mut actual_total_steps_x = 0i32;
    let mut actual_total_steps_y = 0i32;

    for slice_idx in 0..num_slices {
        let slice_start_time = slice_idx as f64 * TIME_SLICE_MS as f64 / 1000.0;
        let slice_end_time =
            ((slice_idx + 1) as f64 * TIME_SLICE_MS as f64 / 1000.0).min(total_time_secs);

        // Handle last slice - may be shorter
        let actual_slice_duration_ms = if slice_idx == num_slices - 1 {
            let remaining_ms = total_time_ms.saturating_sub(slice_idx * TIME_SLICE_MS);
            remaining_ms.max(1)
        } else {
            slice_duration_ms
        };

        // Calculate velocity at the midpoint of this slice for best accuracy
        let slice_mid_time = (slice_start_time + slice_end_time) / 2.0;
        let velocity = velocity_at_time(profile, slice_mid_time);

        // Calculate distance traveled in this slice
        let slice_distance = velocity * (actual_slice_duration_ms as f64 / 1000.0);
        let new_cumulative = cumulative_distance + slice_distance;

        // Calculate target position at end of this slice
        let target_x = dir_x * new_cumulative;
        let target_y = dir_y * new_cumulative;

        // Calculate target steps (total from start)
        let target_steps_x = (target_x * steps_per_mm).round() as i32;
        let target_steps_y = (target_y * steps_per_mm).round() as i32;

        // Steps for this slice = target - cumulative
        let slice_steps_x = target_steps_x - cumulative_steps_x;
        let slice_steps_y = target_steps_y - cumulative_steps_y;

        // Create SM command with CoreXY transform
        let mut cmd =
            SmCommand::from_xy_steps(actual_slice_duration_ms, slice_steps_x, slice_steps_y);

        // Validate and adjust for motor safety
        if cmd.validate_and_adjust() {
            // Track actual steps from the validated command (may differ from intended)
            let (actual_x, actual_y) = cmd.xy_steps();
            actual_total_steps_x += actual_x;
            actual_total_steps_y += actual_y;

            total_duration_ms += cmd.duration_ms;
            commands.push(cmd);
        }

        // Update cumulative tracking for velocity profile interpolation
        // (uses intended steps to maintain smooth velocity curve)
        cumulative_distance = new_cumulative;
        cumulative_steps_x = target_steps_x;
        cumulative_steps_y = target_steps_y;
    }

    SmPlannedMove {
        commands,
        total_duration_ms,
        start,
        end,
        actual_steps_x: actual_total_steps_x,
        actual_steps_y: actual_total_steps_y,
    }
}

/// Generate a single SM command for very short moves
fn generate_single_sm_command(
    profile: &MotionProfile,
    start: Point,
    end: Point,
    steps_per_mm: f64,
) -> SmPlannedMove {
    let delta = end - start;
    let distance = delta.length();

    if distance < 0.001 {
        return SmPlannedMove::empty(start, end);
    }

    // Use average velocity for short moves
    let total_time_secs = profile.total_time();
    let avg_velocity = if total_time_secs > 1e-9 {
        distance / total_time_secs
    } else {
        profile.cruise_velocity
    };

    let duration_ms = if avg_velocity > 1e-9 {
        ((distance / avg_velocity) * 1000.0).round() as u32
    } else {
        TIME_SLICE_MS
    };
    let duration_ms = duration_ms.max(1);

    let steps_x = (delta.x * steps_per_mm).round() as i32;
    let steps_y = (delta.y * steps_per_mm).round() as i32;

    let mut cmd = SmCommand::from_xy_steps(duration_ms, steps_x, steps_y);

    if cmd.validate_and_adjust() {
        // Get actual steps after validation (may differ from intended due to adjustments)
        let (actual_x, actual_y) = cmd.xy_steps();

        SmPlannedMove {
            commands: vec![cmd],
            total_duration_ms: duration_ms,
            start,
            end,
            actual_steps_x: actual_x,
            actual_steps_y: actual_y,
        }
    } else {
        SmPlannedMove::empty(start, end)
    }
}

/// Calculate velocity at a specific time within a motion profile
///
/// The profile has three phases:
/// 1. Acceleration: v increases linearly from entry_velocity to cruise_velocity
/// 2. Cruise: v stays constant at cruise_velocity  
/// 3. Deceleration: v decreases linearly from cruise_velocity to exit_velocity
fn velocity_at_time(profile: &MotionProfile, time_secs: f64) -> f64 {
    if time_secs <= 0.0 {
        return profile.entry_velocity;
    }

    // Calculate phase times
    let accel_time = if profile.acceleration > 1e-9 && profile.accel_distance > 1e-9 {
        (profile.cruise_velocity - profile.entry_velocity) / profile.acceleration
    } else {
        0.0
    };

    let cruise_time = if profile.cruise_velocity > 1e-9 && profile.cruise_distance > 1e-9 {
        profile.cruise_distance / profile.cruise_velocity
    } else {
        0.0
    };

    let decel_time = if profile.acceleration > 1e-9 && profile.decel_distance > 1e-9 {
        (profile.cruise_velocity - profile.exit_velocity) / profile.acceleration
    } else {
        0.0
    };

    let total_time = accel_time + cruise_time + decel_time;

    if time_secs >= total_time {
        return profile.exit_velocity;
    }

    // Determine which phase we're in
    if time_secs < accel_time {
        // Acceleration phase: v = v0 + a*t
        profile.entry_velocity + profile.acceleration * time_secs
    } else if time_secs < accel_time + cruise_time {
        // Cruise phase: constant velocity
        profile.cruise_velocity
    } else {
        // Deceleration phase: v = v_cruise - a*(t - t_decel_start)
        let decel_elapsed = time_secs - accel_time - cruise_time;
        (profile.cruise_velocity - profile.acceleration * decel_elapsed).max(profile.exit_velocity)
    }
}

// ============================================================================
// LM Command - Legacy approach (kept for compatibility)
// ============================================================================

/// A sequence of LM commands representing a motion-planned move (legacy)
#[derive(Debug, Clone)]
pub struct PlannedMove {
    /// The LM commands to execute in sequence
    pub commands: Vec<LmCommand>,
    /// Total duration of the move (ms)
    pub total_duration_ms: u32,
    /// Start point
    pub start: Point,
    /// End point
    pub end: Point,
}

impl PlannedMove {
    /// Create a simple constant-velocity move (for backward compatibility)
    pub fn constant_velocity(
        start: Point,
        end: Point,
        velocity_mm_s: f64,
        steps_per_mm: f64,
    ) -> Self {
        let delta = end - start;
        let distance = delta.length();

        if distance < 0.01 {
            return Self {
                commands: Vec::new(),
                total_duration_ms: 0,
                start,
                end,
            };
        }

        let steps_x = (delta.x * steps_per_mm).round() as i32;
        let steps_y = (delta.y * steps_per_mm).round() as i32;

        let cmd =
            LmCommand::constant_velocity(steps_x, steps_y, velocity_mm_s, distance, steps_per_mm);
        let duration = cmd.duration_ms;

        Self {
            commands: vec![cmd],
            total_duration_ms: duration,
            start,
            end,
        }
    }

    /// Create a motion-planned move with acceleration profile
    ///
    /// For a trapezoidal profile, this generates up to 3 commands:
    /// 1. Acceleration phase (if any)
    /// 2. Cruise phase (if any)
    /// 3. Deceleration phase (if any)
    pub fn with_profile(
        start: Point,
        end: Point,
        profile: &MotionProfile,
        steps_per_mm: f64,
    ) -> Self {
        let delta = end - start;
        let distance = delta.length();

        if distance < 0.01 || profile.total_distance < 0.01 {
            return Self {
                commands: Vec::new(),
                total_duration_ms: 0,
                start,
                end,
            };
        }

        // Unit direction vector
        let dir_x = delta.x / distance;
        let dir_y = delta.y / distance;

        let mut commands = Vec::new();
        let mut total_duration_ms = 0u32;

        // Helper to create command for a phase
        let create_phase_command =
            |phase_distance: f64, start_vel: f64, end_vel: f64, accel: f64| -> Option<LmCommand> {
                if phase_distance < 0.001 {
                    return None;
                }

                // Calculate steps for this phase
                let steps_x = (dir_x * phase_distance * steps_per_mm).round() as i32;
                let steps_y = (dir_y * phase_distance * steps_per_mm).round() as i32;

                // Apply CoreXY transform
                let steps_axis1 = steps_x + steps_y;
                let steps_axis2 = steps_x - steps_y;

                // Calculate time for this phase: t = (v_end - v_start) / a for accel/decel
                // or t = distance / velocity for cruise
                let phase_time = if accel.abs() > 1e-9 {
                    ((end_vel - start_vel) / accel).abs()
                } else if start_vel > 1e-9 {
                    phase_distance / start_vel
                } else {
                    0.0
                };

                if phase_time < 1e-9 {
                    return None;
                }

                let duration_ms = (phase_time * 1000.0).round() as u32;

                // Calculate initial rate (at start of phase)
                let freq_axis1 = if phase_time > 1e-9 {
                    (steps_axis1.abs() as f64) / phase_time
                } else {
                    0.0
                };
                let freq_axis2 = if phase_time > 1e-9 {
                    (steps_axis2.abs() as f64) / phase_time
                } else {
                    0.0
                };

                // For acceleration phases, we start at a lower rate
                // For cruise, rate is constant
                // For deceleration, we start at higher rate
                let (rate1, rate2, accel1, accel2) = if accel.abs() > 1e-9 {
                    // Calculate initial and final rates
                    let initial_speed = start_vel;
                    let final_speed = end_vel;

                    // Rate is proportional to step frequency
                    // Initial frequency for each axis
                    let initial_freq1 = (steps_axis1.abs() as f64) * initial_speed / phase_distance;
                    let initial_freq2 = (steps_axis2.abs() as f64) * initial_speed / phase_distance;
                    let final_freq1 = (steps_axis1.abs() as f64) * final_speed / phase_distance;
                    let final_freq2 = (steps_axis2.abs() as f64) * final_speed / phase_distance;

                    // Clamp rates to valid range (max step rate ~25kHz gives Rate ~2^31)
                    let rate1 = (RATE_FACTOR * initial_freq1)
                        .round()
                        .clamp(0.0, u32::MAX as f64) as u32;
                    let rate2 = (RATE_FACTOR * initial_freq2)
                        .round()
                        .clamp(0.0, u32::MAX as f64) as u32;

                    // Accel is change in rate per 40us
                    // Total rate change = final_rate - initial_rate
                    // Number of ISR intervals = phase_time / 40e-6
                    let num_intervals = phase_time / ISR_INTERVAL_SECS;
                    let rate_change1 = RATE_FACTOR * (final_freq1 - initial_freq1);
                    let rate_change2 = RATE_FACTOR * (final_freq2 - initial_freq2);

                    // Clamp accel to valid i32 range
                    let accel1 = if num_intervals > 1.0 {
                        (rate_change1 / num_intervals)
                            .round()
                            .clamp(i32::MIN as f64, i32::MAX as f64) as i32
                    } else {
                        0
                    };
                    let accel2 = if num_intervals > 1.0 {
                        (rate_change2 / num_intervals)
                            .round()
                            .clamp(i32::MIN as f64, i32::MAX as f64) as i32
                    } else {
                        0
                    };

                    (rate1, rate2, accel1, accel2)
                } else {
                    // Constant velocity - no acceleration
                    let rate1 = (RATE_FACTOR * freq_axis1)
                        .round()
                        .clamp(0.0, u32::MAX as f64) as u32;
                    let rate2 = (RATE_FACTOR * freq_axis2)
                        .round()
                        .clamp(0.0, u32::MAX as f64) as u32;
                    (rate1, rate2, 0, 0)
                };

                Some(LmCommand {
                    rate1,
                    steps1: steps_axis1,
                    accel1,
                    rate2,
                    steps2: steps_axis2,
                    accel2,
                    duration_ms: duration_ms.max(1),
                    clear: 0, // Set later for first command in stroke
                })
            };

        // Generate acceleration phase
        if let Some(cmd) = create_phase_command(
            profile.accel_distance,
            profile.entry_velocity,
            profile.cruise_velocity,
            profile.acceleration,
        ) {
            total_duration_ms += cmd.duration_ms;
            commands.push(cmd);
        }

        // Generate cruise phase
        if let Some(cmd) = create_phase_command(
            profile.cruise_distance,
            profile.cruise_velocity,
            profile.cruise_velocity,
            0.0,
        ) {
            total_duration_ms += cmd.duration_ms;
            commands.push(cmd);
        }

        // Generate deceleration phase
        if let Some(cmd) = create_phase_command(
            profile.decel_distance,
            profile.cruise_velocity,
            profile.exit_velocity,
            -profile.acceleration,
        ) {
            total_duration_ms += cmd.duration_ms;
            commands.push(cmd);
        }

        // When starting from velocity 0, clear accumulators on the first command
        // to avoid artifacts from previous moves. This is especially important
        // when rate=0 and accel>0, which can cause timing issues without a clear.
        if profile.entry_velocity < 1e-9 && !commands.is_empty() {
            commands[0].clear = 3; // Clear both axis accumulators
        }

        Self {
            commands,
            total_duration_ms,
            start,
            end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_motion_config_default() {
        let config = MotionConfig::default();
        assert_eq!(config.max_velocity, 25.0);
        assert_eq!(config.max_acceleration, DEFAULT_ACCEL_PEN_DOWN); // 1016.0 mm/s²
        assert_eq!(config.steps_per_mm, 80.0);
    }

    #[test]
    fn test_motion_segment_creation() {
        let seg = MotionSegment::new(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 25.0);

        assert_eq!(seg.length, 10.0);
        assert!((seg.direction.x - 1.0).abs() < 1e-9);
        assert!(seg.direction.y.abs() < 1e-9);
    }

    #[test]
    fn test_motion_segment_diagonal() {
        let seg = MotionSegment::new(Point::new(0.0, 0.0), Point::new(3.0, 4.0), 25.0);

        assert!((seg.length - 5.0).abs() < 1e-9);
        assert!((seg.direction.x - 0.6).abs() < 1e-9);
        assert!((seg.direction.y - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_segment_angle_straight() {
        let seg1 = MotionSegment::new(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 25.0);
        let seg2 = MotionSegment::new(Point::new(10.0, 0.0), Point::new(20.0, 0.0), 25.0);

        let angle = seg1.angle_to(&seg2);
        assert!(angle.abs() < 0.01, "Straight segments should have ~0 angle");
    }

    #[test]
    fn test_segment_angle_90_degrees() {
        let seg1 = MotionSegment::new(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 25.0);
        let seg2 = MotionSegment::new(Point::new(10.0, 0.0), Point::new(10.0, 10.0), 25.0);

        let angle = seg1.angle_to(&seg2);
        assert!(
            (angle - PI / 2.0).abs() < 0.01,
            "90-degree turn should have PI/2 angle"
        );
    }

    #[test]
    fn test_segment_angle_180_degrees() {
        let seg1 = MotionSegment::new(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 25.0);
        let seg2 = MotionSegment::new(Point::new(10.0, 0.0), Point::new(0.0, 0.0), 25.0);

        let angle = seg1.angle_to(&seg2);
        assert!(
            (angle - PI).abs() < 0.01,
            "180-degree turn should have PI angle"
        );
    }

    #[test]
    fn test_junction_velocity_straight() {
        let config = MotionConfig::default();
        let seg1 = MotionSegment::new(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 25.0);
        let seg2 = MotionSegment::new(Point::new(10.0, 0.0), Point::new(20.0, 0.0), 25.0);

        let v_junction = calculate_junction_velocity(&seg1, &seg2, &config);
        assert!(
            (v_junction - config.max_velocity).abs() < 0.1,
            "Straight path should allow max velocity"
        );
    }

    #[test]
    fn test_junction_velocity_sharp_corner() {
        let config = MotionConfig::default();
        let seg1 = MotionSegment::new(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 25.0);
        let seg2 = MotionSegment::new(Point::new(10.0, 0.0), Point::new(0.0, 0.0), 25.0);

        let v_junction = calculate_junction_velocity(&seg1, &seg2, &config);
        assert!(
            v_junction < 1.0,
            "180-degree turn should have very low velocity"
        );
    }

    #[test]
    fn test_velocity_after_acceleration() {
        // Starting at 0, accelerating at 500 mm/s^2 over 10mm
        // v^2 = 0 + 2 * 500 * 10 = 10000
        // v = 100 mm/s
        let v = velocity_after_acceleration(0.0, 500.0, 10.0);
        assert!((v - 100.0).abs() < 0.01);

        // Starting at 10 mm/s, accelerating at 500 mm/s^2 over 5mm
        // v^2 = 100 + 2 * 500 * 5 = 5100
        // v = 71.4 mm/s
        let v2 = velocity_after_acceleration(10.0, 500.0, 5.0);
        assert!((v2 - 71.414).abs() < 0.01);
    }

    #[test]
    fn test_distance_to_velocity() {
        // From 0 to 100 mm/s at 500 mm/s^2
        // s = (100^2 - 0) / (2 * 500) = 10000 / 1000 = 10mm
        let d = distance_to_velocity(0.0, 100.0, 500.0);
        assert!((d - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_motion_profile_trapezoidal() {
        // Long segment that can reach cruise velocity
        let profile = MotionProfile::calculate(0.0, 0.0, 100.0, 50.0, 500.0);

        assert!(!profile.is_triangular());
        assert!(profile.cruise_distance > 0.0);
        assert_eq!(profile.entry_velocity, 0.0);
        assert_eq!(profile.exit_velocity, 0.0);
        assert_eq!(profile.cruise_velocity, 100.0);
    }

    #[test]
    fn test_motion_profile_triangular() {
        // Short segment that can't reach cruise velocity
        // To reach 100mm/s from 0 at 500mm/s^2 takes 10mm
        // So a 5mm segment with 0 entry/exit can only reach 50mm/s peak
        let profile = MotionProfile::calculate(0.0, 0.0, 100.0, 5.0, 500.0);

        assert!(
            profile.is_triangular(),
            "Expected triangular profile, got cruise_dist={}",
            profile.cruise_distance
        );
        assert!(
            profile.cruise_velocity < 100.0,
            "Expected peak < 100, got {}",
            profile.cruise_velocity
        );
        assert!(
            (profile.cruise_velocity - 50.0).abs() < 1.0,
            "Expected peak ~50mm/s, got {}",
            profile.cruise_velocity
        );
        assert!(profile.cruise_distance.abs() < 1e-6);
    }

    #[test]
    fn test_motion_planner_simple_path() {
        let planner = MotionPlanner::with_defaults();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(20.0, 0.0),
        ];

        let segments = planner.plan(&points);
        assert_eq!(segments.len(), 2);

        // Start and end velocities should be 0
        assert!(segments[0].entry_velocity.abs() < 0.01);
        assert!(segments[1].exit_velocity.abs() < 0.01);
    }

    #[test]
    fn test_motion_planner_corner() {
        let planner = MotionPlanner::with_defaults();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        ];

        let segments = planner.plan(&points);
        assert_eq!(segments.len(), 2);

        // Junction velocity at corner should be less than max
        assert!(segments[0].exit_velocity < planner.config().max_velocity);
        assert!(segments[1].entry_velocity < planner.config().max_velocity);

        // Junction velocities should match
        assert!((segments[0].exit_velocity - segments[1].entry_velocity).abs() < 0.01);
    }

    #[test]
    fn test_velocity_to_rate() {
        let config = MotionConfig::default();

        // At 25 mm/s with 80 steps/mm = 2000 steps/sec
        // Rate = 85899.35 * 2000 = 171,798,700
        let rate = velocity_to_rate(25.0, config.steps_per_mm);
        assert!((rate as f64 - 171_798_691.8).abs() < 100.0);
    }

    #[test]
    fn test_acceleration_to_accel_param() {
        let config = MotionConfig::default();

        // At 500 mm/s^2 with 80 steps/mm = 40000 steps/s^2
        // Accel = 85899.35 * 40000 * 40e-6 = 137438.95
        let accel = acceleration_to_accel_param(500.0, config.steps_per_mm);
        assert!(
            (accel as f64 - 137438.95).abs() < 10.0,
            "Expected ~137439, got {}",
            accel
        );
    }

    #[test]
    fn test_lm_command_to_string() {
        let cmd = LmCommand::new(1000, 100, 50, 2000, -100, -50, 500);
        assert_eq!(cmd.to_command_string(), "LM,1000,100,50,2000,-100,-50");
    }

    #[test]
    fn test_lm_command_constant_velocity() {
        // Move 10mm in X direction at 25mm/s
        let cmd = LmCommand::constant_velocity(800, 0, 25.0, 10.0, 80.0);

        // Duration should be 10mm / 25mm/s = 0.4s = 400ms
        assert_eq!(cmd.duration_ms, 400);

        // CoreXY: axis1 = x+y = 800, axis2 = x-y = 800
        assert_eq!(cmd.steps1, 800);
        assert_eq!(cmd.steps2, 800);

        // No acceleration
        assert_eq!(cmd.accel1, 0);
        assert_eq!(cmd.accel2, 0);
    }

    #[test]
    fn test_planned_move_constant_velocity() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(10.0, 0.0);

        let planned = PlannedMove::constant_velocity(start, end, 25.0, 80.0);

        assert_eq!(planned.commands.len(), 1);
        assert_eq!(planned.total_duration_ms, 400); // 10mm / 25mm/s
        assert_eq!(planned.start, start);
        assert_eq!(planned.end, end);
    }

    #[test]
    fn test_planned_move_with_profile_trapezoidal() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(50.0, 0.0); // Long move

        // Profile that can reach cruise velocity
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 50.0, 500.0);
        assert!(!profile.is_triangular(), "Should be trapezoidal");

        let planned = PlannedMove::with_profile(start, end, &profile, 80.0);

        // Should have up to 3 commands (accel, cruise, decel)
        assert!(
            planned.commands.len() >= 2,
            "Expected at least 2 commands for trapezoidal, got {}",
            planned.commands.len()
        );
        assert!(planned.total_duration_ms > 0);
    }

    #[test]
    fn test_planned_move_with_profile_triangular() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(5.0, 0.0); // Short move

        // Profile that can't reach cruise velocity
        let profile = MotionProfile::calculate(0.0, 0.0, 100.0, 5.0, 500.0);
        assert!(profile.is_triangular(), "Should be triangular");

        let planned = PlannedMove::with_profile(start, end, &profile, 80.0);

        // Should have 2 commands (accel, decel) with no cruise
        assert!(
            !planned.commands.is_empty(),
            "Expected at least 1 command, got {}",
            planned.commands.len()
        );
        assert!(planned.total_duration_ms > 0);
    }

    #[test]
    fn test_planned_move_zero_distance() {
        let start = Point::new(10.0, 10.0);
        let end = Point::new(10.0, 10.0); // Same point

        let planned = PlannedMove::constant_velocity(start, end, 25.0, 80.0);

        assert!(planned.commands.is_empty());
        assert_eq!(planned.total_duration_ms, 0);
    }

    #[test]
    fn test_lm_command_is_valid() {
        // Valid: has steps and rate on axis 1
        let valid1 = LmCommand::new(1000, 100, 0, 0, 0, 0, 100);
        assert!(valid1.is_valid());

        // Valid: has steps and accel on axis 1 (starting from zero rate)
        let valid2 = LmCommand::new(0, 100, 50, 0, 0, 0, 100);
        assert!(valid2.is_valid());

        // Valid: both axes moving
        let valid3 = LmCommand::new(1000, 100, 0, 2000, -100, 0, 100);
        assert!(valid3.is_valid());

        // Invalid: steps but no rate or accel
        let invalid1 = LmCommand::new(0, 100, 0, 0, 0, 0, 100);
        assert!(!invalid1.is_valid());

        // Invalid: no steps on either axis
        let invalid2 = LmCommand::new(1000, 0, 50, 2000, 0, 50, 100);
        assert!(!invalid2.is_valid());

        // Invalid: axis 2 has steps but no rate/accel
        let invalid3 = LmCommand::new(1000, 100, 0, 0, 50, 0, 100);
        assert!(!invalid3.is_valid());
    }

    #[test]
    fn test_lm_command_is_empty() {
        let empty = LmCommand::new(1000, 0, 50, 2000, 0, 50, 100);
        assert!(empty.is_empty());

        let not_empty = LmCommand::new(1000, 100, 0, 2000, 0, 0, 100);
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_planned_move_from_zero_velocity() {
        // This tests the case where we start a stroke from velocity 0
        // which happens at the beginning of every stroke
        let start = Point::new(0.0, 0.0);
        let end = Point::new(10.0, 0.0); // 10mm horizontal move

        // Profile starting from 0, going to cruise 25mm/s, ending at 0
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 10.0, 500.0);

        let planned = PlannedMove::with_profile(start, end, &profile, 80.0);

        println!(
            "Profile: entry={}, cruise={}, exit={}",
            profile.entry_velocity, profile.cruise_velocity, profile.exit_velocity
        );
        println!(
            "Profile distances: accel={}, cruise={}, decel={}",
            profile.accel_distance, profile.cruise_distance, profile.decel_distance
        );
        println!("Generated {} commands:", planned.commands.len());

        for (i, cmd) in planned.commands.iter().enumerate() {
            println!("  Cmd {}: rate1={}, steps1={}, accel1={}, rate2={}, steps2={}, accel2={}, duration={}ms",
                i, cmd.rate1, cmd.steps1, cmd.accel1, cmd.rate2, cmd.steps2, cmd.accel2, cmd.duration_ms);
            println!("    Command string: {}", cmd.to_command_string());
            println!("    is_valid: {}", cmd.is_valid());
        }

        // All generated commands should be valid
        for (i, cmd) in planned.commands.iter().enumerate() {
            assert!(cmd.is_valid(), "Command {} should be valid: {:?}", i, cmd);
        }
    }

    #[test]
    fn test_accel_phase_from_zero() {
        // Test specifically the acceleration phase starting from 0
        // This mimics what happens at the start of a stroke

        let start = Point::new(0.0, 0.0);
        let end = Point::new(5.0, 0.0); // Short 5mm move, likely triangular

        // Entry velocity 0, exit velocity 0, max velocity 25, short distance
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 5.0, 500.0);

        println!("Short move profile:");
        println!("  entry_velocity: {}", profile.entry_velocity);
        println!("  cruise_velocity: {}", profile.cruise_velocity);
        println!("  exit_velocity: {}", profile.exit_velocity);
        println!("  accel_distance: {}", profile.accel_distance);
        println!("  cruise_distance: {}", profile.cruise_distance);
        println!("  decel_distance: {}", profile.decel_distance);
        println!("  is_triangular: {}", profile.is_triangular());

        let planned = PlannedMove::with_profile(start, end, &profile, 80.0);

        println!("Generated {} commands:", planned.commands.len());
        for (i, cmd) in planned.commands.iter().enumerate() {
            println!("  Cmd {}: {}", i, cmd.to_command_string());
            println!(
                "    rate1={}, accel1={}, steps1={}",
                cmd.rate1, cmd.accel1, cmd.steps1
            );

            // Check if starting from rate=0 with accel
            if i == 0 && cmd.rate1 == 0 && cmd.accel1 != 0 {
                println!("    WARNING: Starting from rate=0 with accel!=0");
            }
        }
    }

    // ========================================================================
    // SM Command Tests
    // ========================================================================

    #[test]
    fn test_sm_command_creation() {
        let cmd = SmCommand::new(25, 100, -50);
        assert_eq!(cmd.duration_ms, 25);
        assert_eq!(cmd.steps_axis1, 100);
        assert_eq!(cmd.steps_axis2, -50);
    }

    #[test]
    fn test_sm_command_from_xy_steps() {
        // CoreXY transform: axis1 = x+y, axis2 = x-y
        let cmd = SmCommand::from_xy_steps(25, 100, 50);
        assert_eq!(cmd.duration_ms, 25);
        assert_eq!(cmd.steps_axis1, 150); // 100 + 50
        assert_eq!(cmd.steps_axis2, 50); // 100 - 50
    }

    #[test]
    fn test_sm_command_to_string() {
        let cmd = SmCommand::new(25, 100, -50);
        assert_eq!(cmd.to_command_string(), "SM,25,100,-50");
    }

    #[test]
    fn test_sm_command_is_empty() {
        let empty = SmCommand::new(25, 0, 0);
        assert!(empty.is_empty());

        let not_empty = SmCommand::new(25, 100, 0);
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_sm_command_sleep_time() {
        // Long command: sleep = duration - 30ms buffer
        let long_cmd = SmCommand::new(100, 100, 50);
        assert_eq!(long_cmd.sleep_time_ms(), 70); // 100 - 30

        // Short command: no sleep (commands queue in EBB buffer)
        let short_cmd = SmCommand::new(40, 100, 50);
        assert_eq!(short_cmd.sleep_time_ms(), 0);
    }

    #[test]
    fn test_sm_command_validate_overspeed() {
        // Create command that exceeds max step rate (25 steps/ms)
        // 100 steps in 2ms = 50 steps/ms > 25 max
        let mut cmd = SmCommand::new(2, 100, 50);
        let valid = cmd.validate_and_adjust();

        assert!(valid);
        // Duration should be increased to bring rate below 25 steps/ms
        assert!(cmd.duration_ms > 2);
        let rate = cmd.steps_axis1.abs() as f64 / cmd.duration_ms as f64;
        assert!(
            rate < MAX_STEP_RATE,
            "Rate {} should be < {}",
            rate,
            MAX_STEP_RATE
        );
    }

    #[test]
    fn test_sm_command_validate_underspeed() {
        // Create command with very slow step rate (< 0.002 steps/ms)
        // 1 step in 1000ms = 0.001 steps/ms < 0.002 min
        let mut cmd = SmCommand::new(1000, 1, 100);
        let valid = cmd.validate_and_adjust();

        assert!(valid); // Still valid because axis2 has enough steps
        assert_eq!(cmd.steps_axis1, 0); // Axis1 zeroed out due to underspeed
        assert_eq!(cmd.steps_axis2, 100); // Axis2 unchanged
    }

    #[test]
    fn test_sm_command_validate_both_underspeed() {
        // Both axes too slow - should return false
        let mut cmd = SmCommand::new(1000, 1, 1);
        let valid = cmd.validate_and_adjust();

        assert!(!valid); // Invalid - would result in no motion
    }

    // ========================================================================
    // SM Planned Move Tests
    // ========================================================================

    #[test]
    fn test_sm_planned_move_empty() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(0.0, 0.0);
        let planned = SmPlannedMove::empty(start, end);

        assert!(planned.commands.is_empty());
        assert_eq!(planned.total_duration_ms, 0);
    }

    // ========================================================================
    // generate_sm_commands Tests
    // ========================================================================

    #[test]
    fn test_generate_sm_commands_short_move() {
        // Very short move (<=50ms total time) uses single SM command
        // At 1016 mm/s² acceleration and 25mm/s max velocity:
        // - Time to reach 25mm/s from 0 = 25/1016 = 0.0246s = 24.6ms
        // - Distance during accel = 0.5 * 1016 * 0.0246² = 0.31mm
        // So for a 0.5mm move, we have triangular profile with peak velocity
        // and total time around 44ms, which is < 50ms threshold
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 0.5, DEFAULT_ACCEL_PEN_DOWN);
        let start = Point::new(0.0, 0.0);
        let end = Point::new(0.5, 0.0);

        // Verify this is indeed a "short" move (<=50ms)
        let total_time_ms = (profile.total_time() * 1000.0).round() as u32;
        assert!(
            total_time_ms <= TIME_SLICE_MS * 2,
            "Profile should be <=50ms for short move path, got {}ms",
            total_time_ms
        );

        let planned = generate_sm_commands(&profile, start, end, 80.0);

        assert!(
            !planned.commands.is_empty(),
            "Should have at least one command"
        );
        // Short moves use generate_single_sm_command which produces exactly 1 command
        assert_eq!(
            planned.commands.len(),
            1,
            "Short move should have exactly 1 command"
        );

        // Verify steps are correct (0.5mm * 80 steps/mm = 40 steps in X)
        // For single SM command, steps should be exact
        let total_steps_axis1: i32 = planned.commands.iter().map(|c| c.steps_axis1).sum();
        let total_steps_axis2: i32 = planned.commands.iter().map(|c| c.steps_axis2).sum();
        // CoreXY: axis1 = x+y, axis2 = x-y, for X-only move both should be 40
        assert_eq!(
            total_steps_axis1, 40,
            "Expected 40 axis1 steps, got {}",
            total_steps_axis1
        );
        assert_eq!(
            total_steps_axis2, 40,
            "Expected 40 axis2 steps, got {}",
            total_steps_axis2
        );
    }

    #[test]
    fn test_generate_sm_commands_long_move() {
        // Long move should produce multiple 25ms time slices
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 50.0, DEFAULT_ACCEL_PEN_DOWN);
        let start = Point::new(0.0, 0.0);
        let end = Point::new(50.0, 0.0);

        let planned = generate_sm_commands(&profile, start, end, 80.0);

        // At 25mm/s, 50mm takes ~2 seconds, which is ~80 time slices
        assert!(
            planned.commands.len() > 10,
            "Long move should have many commands, got {}",
            planned.commands.len()
        );

        // Each command should be ~25ms (except possibly the last one)
        for (i, cmd) in planned.commands.iter().enumerate() {
            if i < planned.commands.len() - 1 {
                assert!(
                    cmd.duration_ms >= 20 && cmd.duration_ms <= 30,
                    "Command {} duration {} should be ~25ms",
                    i,
                    cmd.duration_ms
                );
            }
        }

        // Total steps should match expected (50mm * 80 steps/mm = 4000 steps in X)
        let total_steps_axis1: i32 = planned.commands.iter().map(|c| c.steps_axis1).sum();
        let total_steps_axis2: i32 = planned.commands.iter().map(|c| c.steps_axis2).sum();
        assert!(
            (total_steps_axis1 - 4000).abs() <= 10,
            "Expected ~4000 axis1 steps, got {}",
            total_steps_axis1
        );
        assert!(
            (total_steps_axis2 - 4000).abs() <= 10,
            "Expected ~4000 axis2 steps, got {}",
            total_steps_axis2
        );
    }

    #[test]
    fn test_generate_sm_commands_diagonal_move() {
        // Diagonal move: 30mm X, 40mm Y (50mm total distance)
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 50.0, DEFAULT_ACCEL_PEN_DOWN);
        let start = Point::new(0.0, 0.0);
        let end = Point::new(30.0, 40.0);

        let planned = generate_sm_commands(&profile, start, end, 80.0);

        assert!(!planned.commands.is_empty());

        // CoreXY: axis1 = x+y = 30+40 = 70mm = 5600 steps
        // CoreXY: axis2 = x-y = 30-40 = -10mm = -800 steps
        let total_steps_axis1: i32 = planned.commands.iter().map(|c| c.steps_axis1).sum();
        let total_steps_axis2: i32 = planned.commands.iter().map(|c| c.steps_axis2).sum();
        assert!(
            (total_steps_axis1 - 5600).abs() <= 20,
            "Expected ~5600 axis1 steps, got {}",
            total_steps_axis1
        );
        assert!(
            (total_steps_axis2 - (-800)).abs() <= 20,
            "Expected ~-800 axis2 steps, got {}",
            total_steps_axis2
        );
    }

    #[test]
    fn test_generate_sm_commands_trapezoidal_profile() {
        // Long move that will have full trapezoidal profile (accel/cruise/decel)
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 100.0, DEFAULT_ACCEL_PEN_DOWN);
        let start = Point::new(0.0, 0.0);
        let end = Point::new(100.0, 0.0);

        assert!(!profile.is_triangular(), "Profile should be trapezoidal");
        assert!(profile.cruise_distance > 0.0, "Should have cruise phase");

        let planned = generate_sm_commands(&profile, start, end, 80.0);

        // Commands should exist
        assert!(!planned.commands.is_empty());

        // Verify all commands are valid SM commands
        for cmd in &planned.commands {
            assert!(!cmd.is_empty() || cmd.duration_ms == 0);
        }
    }

    #[test]
    fn test_generate_sm_commands_triangular_profile() {
        // Short move that will have triangular profile (accel/decel, no cruise)
        // Use a distance long enough to trigger time-slice path but still triangular
        let profile = MotionProfile::calculate(0.0, 0.0, 100.0, 5.0, DEFAULT_ACCEL_PEN_DOWN);
        let start = Point::new(0.0, 0.0);
        let end = Point::new(5.0, 0.0);

        assert!(profile.is_triangular(), "Profile should be triangular");

        let planned = generate_sm_commands(&profile, start, end, 80.0);

        assert!(!planned.commands.is_empty());

        // Total steps should be close to expected (5mm * 80 steps/mm = 400 steps)
        // Allow ~10% tolerance due to time-slice velocity integration
        let total_steps_axis1: i32 = planned.commands.iter().map(|c| c.steps_axis1).sum();
        let expected = 400;
        let tolerance = (expected as f64 * 0.1) as i32; // 10% tolerance
        assert!(
            (total_steps_axis1 - expected).abs() <= tolerance,
            "Expected ~{} axis1 steps (±{}), got {}",
            expected,
            tolerance,
            total_steps_axis1
        );
    }

    #[test]
    fn test_generate_sm_commands_with_entry_velocity() {
        // Move with non-zero entry velocity (like continuing from a corner)
        let profile = MotionProfile::calculate(10.0, 0.0, 25.0, 20.0, DEFAULT_ACCEL_PEN_DOWN);
        let start = Point::new(0.0, 0.0);
        let end = Point::new(20.0, 0.0);

        let planned = generate_sm_commands(&profile, start, end, 80.0);

        assert!(!planned.commands.is_empty());

        // First command should have some steps (not starting from zero velocity)
        assert!(
            planned.commands[0].steps_axis1 != 0 || planned.commands[0].steps_axis2 != 0,
            "First command should have motion"
        );
    }

    #[test]
    fn test_generate_sm_commands_zero_distance() {
        // Zero distance move should return empty
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 0.0, DEFAULT_ACCEL_PEN_DOWN);
        let start = Point::new(10.0, 10.0);
        let end = Point::new(10.0, 10.0);

        let planned = generate_sm_commands(&profile, start, end, 80.0);

        assert!(planned.commands.is_empty());
    }

    #[test]
    fn test_generate_sm_commands_preserves_total_steps() {
        // Verify that sum of all command steps is close to expected total
        // Short moves (<=50ms total time) use single SM command and are exact
        // Longer moves use time-slice interpolation and may have slight variation

        // Test very short moves (exact steps via generate_single_sm_command)
        // At high acceleration (1016mm/s²), only very short distances qualify
        for distance in [0.1, 0.2, 0.3] {
            let profile =
                MotionProfile::calculate(0.0, 0.0, 25.0, distance, DEFAULT_ACCEL_PEN_DOWN);
            let start = Point::new(0.0, 0.0);
            let end = Point::new(distance, 0.0);

            let total_time_ms = (profile.total_time() * 1000.0).round() as u32;
            if total_time_ms > TIME_SLICE_MS * 2 {
                // Skip if this isn't actually a "short" move
                continue;
            }

            let planned = generate_sm_commands(&profile, start, end, 80.0);

            let expected_steps = (distance * 80.0).round() as i32;
            let total_steps_axis1: i32 = planned.commands.iter().map(|c| c.steps_axis1).sum();

            assert_eq!(
                total_steps_axis1, expected_steps,
                "Short move {}: expected {} steps, got {}",
                distance, expected_steps, total_steps_axis1
            );
        }

        // Test longer moves (time-slice interpolation, allow 10% tolerance)
        // The velocity integration over time slices may not exactly match distance
        for distance in [5.0, 25.0, 50.0, 100.0] {
            let profile =
                MotionProfile::calculate(0.0, 0.0, 25.0, distance, DEFAULT_ACCEL_PEN_DOWN);
            let start = Point::new(0.0, 0.0);
            let end = Point::new(distance, 0.0);

            let planned = generate_sm_commands(&profile, start, end, 80.0);

            let expected_steps = (distance * 80.0).round() as i32;
            let total_steps_axis1: i32 = planned.commands.iter().map(|c| c.steps_axis1).sum();
            let tolerance = (expected_steps as f64 * 0.10) as i32; // 10% tolerance

            assert!(
                (total_steps_axis1 - expected_steps).abs() <= tolerance,
                "Long move {}: expected ~{} steps (±{}), got {}",
                distance,
                expected_steps,
                tolerance,
                total_steps_axis1
            );
        }
    }

    #[test]
    fn test_velocity_at_time_helper() {
        // Test the velocity_at_time function used internally
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 50.0, 500.0);

        // At t=0, velocity should be entry_velocity (0)
        let v0 = velocity_at_time(&profile, 0.0);
        assert!((v0 - 0.0).abs() < 0.1, "v(0) should be ~0, got {}", v0);

        // During cruise phase, velocity should be cruise_velocity
        let total_time = profile.total_time();
        let mid_time = total_time / 2.0;
        let v_mid = velocity_at_time(&profile, mid_time);
        assert!(
            v_mid > 20.0,
            "v(mid) should be near cruise velocity, got {}",
            v_mid
        );

        // At end, velocity should be exit_velocity (0)
        let v_end = velocity_at_time(&profile, total_time);
        assert!(
            (v_end - 0.0).abs() < 0.1,
            "v(end) should be ~0, got {}",
            v_end
        );
    }
}
