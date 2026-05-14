//! SM (Stepper Move) command - Time-slice based motion (recommended approach)

use drawing_core::Point;

use super::config::{BUFFER_LEAD_MS, MAX_STEP_RATE, MIN_STEP_RATE, TIME_SLICE_MS};
use super::profile::MotionProfile;

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

        // Check for underspeed (motor resonance) on the Cartesian axes.
        //
        // IMPORTANT: this check must be done in X/Y space, not on the CoreXY
        // motor axes. Zeroing a single motor axis while the other retains
        // steps causes the carriage to move diagonally by half a step in both
        // X and Y (Δx = (Δa1+Δa2)/2, Δy = (Δa1-Δa2)/2), and also makes
        // `xy_steps()` lose half a step to integer truncation. The net effect
        // is a sub-mm positional skew on short detail strokes (e.g. dots on
        // i's, t-crossbars).
        //
        // By construction `steps_axis1 ± steps_axis2` is even (we always
        // build commands via `from_xy_steps`), so we can recover the original
        // X/Y steps exactly here.
        let steps_x = (self.steps_axis1 + self.steps_axis2) / 2;
        let steps_y = (self.steps_axis1 - self.steps_axis2) / 2;
        let rate_x = steps_x.abs() as f64 / self.duration_ms as f64;
        let rate_y = steps_y.abs() as f64 / self.duration_ms as f64;

        let mut new_x = steps_x;
        let mut new_y = steps_y;
        if rate_x > 0.0 && rate_x < MIN_STEP_RATE {
            new_x = 0;
        }
        if rate_y > 0.0 && rate_y < MIN_STEP_RATE {
            new_y = 0;
        }
        if new_x != steps_x || new_y != steps_y {
            self.steps_axis1 = new_x + new_y;
            self.steps_axis2 = new_x - new_y;
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

    // Exact integer-step target for the whole move. We force the planner's
    // cumulative target on the LAST slice to equal these values, and emit a
    // correction command at the end if validation dropped any steps, so that
    // the SM-based motion lands on the same endpoint the LM-based `move_to`
    // would produce (within 0.5 step). Without this, time-slice midpoint
    // integration accumulates fractional errors and the carriage ends short
    // of the requested endpoint — which manifests as every subsequent path
    // being dislocated by the residual.
    let total_target_x = (delta.x * steps_per_mm).round() as i32;
    let total_target_y = (delta.y * steps_per_mm).round() as i32;

    let mut commands = Vec::with_capacity(num_slices as usize + 1);
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

        // Calculate target steps (total from start). On the final slice,
        // snap to the exact integer-step total so the move ends precisely at
        // `end`, regardless of midpoint-velocity integration error.
        let (target_steps_x, target_steps_y) = if slice_idx == num_slices - 1 {
            (total_target_x, total_target_y)
        } else {
            (
                (target_x * steps_per_mm).round() as i32,
                (target_y * steps_per_mm).round() as i32,
            )
        };

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

    // Correction command: if validation dropped any slices' steps (e.g. an
    // underspeed slice at the tail of a decel phase was zeroed out), append
    // a final small SM command to carry the missing steps. This guarantees
    // SM-based motion lands on the same integer-step endpoint as LM `move_to`
    // would, which is critical for path-to-path positioning accuracy.
    let missing_x = total_target_x - actual_total_steps_x;
    let missing_y = total_target_y - actual_total_steps_y;
    if missing_x != 0 || missing_y != 0 {
        // Pick a duration that keeps the slowest non-zero axis above
        // MIN_STEP_RATE. `validate_and_adjust` will further increase duration
        // if either axis would exceed MAX_STEP_RATE.
        let max_steps = missing_x.abs().max(missing_y.abs()) as f64;
        // Aim for ~half of MAX_STEP_RATE for a comfortable rate.
        let target_rate = MAX_STEP_RATE * 0.5;
        let duration_ms = ((max_steps / target_rate).ceil() as u32).max(1);
        let mut cmd = SmCommand::from_xy_steps(duration_ms, missing_x, missing_y);
        if cmd.validate_and_adjust() {
            let (actual_x, actual_y) = cmd.xy_steps();
            actual_total_steps_x += actual_x;
            actual_total_steps_y += actual_y;
            total_duration_ms += cmd.duration_ms;
            commands.push(cmd);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::config::DEFAULT_ACCEL_PEN_DOWN;

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
    fn test_sm_command_validate_underspeed_xy_space() {
        // Underspeed check operates in X/Y (Cartesian) space, not motor space.
        // A pure-X move of 100 steps over 1000ms => rate_x = 0.1 (OK), rate_y = 0 (ignored).
        // CoreXY encoding: axis1 = x+y = 100, axis2 = x-y = 100.
        let mut cmd = SmCommand::from_xy_steps(1000, 100, 0);
        let valid = cmd.validate_and_adjust();

        assert!(valid);
        // Pure-X move should be preserved exactly (no asymmetric zeroing).
        let (x, y) = cmd.xy_steps();
        assert_eq!(x, 100);
        assert_eq!(y, 0);
    }

    #[test]
    fn test_sm_command_validate_y_underspeed_zeroed() {
        // Y component is too slow (1 step in 1000ms = 0.001 < MIN_STEP_RATE),
        // X is fine. Y should be zeroed in X/Y space, keeping motion straight along X.
        let mut cmd = SmCommand::from_xy_steps(1000, 100, 1);
        let valid = cmd.validate_and_adjust();

        assert!(valid);
        let (x, y) = cmd.xy_steps();
        assert_eq!(x, 100);
        assert_eq!(y, 0); // Y zeroed because of underspeed
                          // Motor axes must remain symmetric (no diagonal skew).
        assert_eq!(cmd.steps_axis1, 100);
        assert_eq!(cmd.steps_axis2, 100);
    }

    #[test]
    fn test_sm_command_validate_both_underspeed() {
        // Both X and Y too slow - should return false
        let mut cmd = SmCommand::from_xy_steps(1000, 1, 1);
        let valid = cmd.validate_and_adjust();

        assert!(!valid); // Invalid - would result in no motion
    }

    #[test]
    fn test_sm_planned_move_empty() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(0.0, 0.0);
        let planned = SmPlannedMove::empty(start, end);

        assert!(planned.commands.is_empty());
        assert_eq!(planned.total_duration_ms, 0);
    }

    #[test]
    fn test_generate_sm_commands_short_move() {
        // Very short move (<=50ms total time) uses single SM command
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
        let profile = MotionProfile::calculate(0.0, 0.0, 100.0, 5.0, DEFAULT_ACCEL_PEN_DOWN);
        let start = Point::new(0.0, 0.0);
        let end = Point::new(5.0, 0.0);

        assert!(profile.is_triangular(), "Profile should be triangular");

        let planned = generate_sm_commands(&profile, start, end, 80.0);

        assert!(!planned.commands.is_empty());

        // Total steps should be close to expected (5mm * 80 steps/mm = 400 steps)
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

        // Test very short moves (exact steps via generate_single_sm_command)
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
    fn test_generate_sm_commands_exact_endpoint() {
        // Regression: SM-based motion must land on the exact integer-step
        // endpoint that LM `move_to` would produce (within 0.5 step), so that
        // pen-up travel doesn't dislocate the next stroke. Previously,
        // time-slice midpoint integration left the carriage short of the
        // target by a fractional amount that accumulated across moves.
        let steps_per_mm = 80.0;
        // Try a variety of distances and directions, including ones that
        // exercise both the multi-slice and trapezoidal-vs-triangular paths.
        let test_cases: &[(f64, f64)] = &[
            (5.0, 0.0),
            (10.0, 0.0),
            (50.0, 0.0),
            (3.0, 4.0),
            (7.3, 2.1),
            (0.5, 0.0),
            (1.7, -2.3),
            (-12.4, 8.7),
            (100.0, 50.0),
        ];
        for &(dx, dy) in test_cases {
            let dist = (dx * dx + dy * dy).sqrt();
            let profile = MotionProfile::calculate(0.0, 0.0, 75.0, dist, 1524.0); // pen-up profile
            let start = Point::new(0.0, 0.0);
            let end = Point::new(dx, dy);
            let planned = generate_sm_commands(&profile, start, end, steps_per_mm);

            let expected_x = (dx * steps_per_mm).round() as i32;
            let expected_y = (dy * steps_per_mm).round() as i32;
            assert_eq!(
                planned.actual_steps_x, expected_x,
                "endpoint X mismatch for delta=({}, {}): expected {} steps, got {}",
                dx, dy, expected_x, planned.actual_steps_x
            );
            assert_eq!(
                planned.actual_steps_y, expected_y,
                "endpoint Y mismatch for delta=({}, {}): expected {} steps, got {}",
                dx, dy, expected_y, planned.actual_steps_y
            );
        }
    }

    #[test]
    fn test_velocity_at_time_helper() {
        // Test the velocity_at_time function
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
