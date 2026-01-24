//! LM (Low-level Move) command - Legacy approach (kept for compatibility)

use drawing_core::Point;

use super::config::{ISR_INTERVAL_SECS, RATE_FACTOR};
use super::profile::MotionProfile;

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
    use crate::motion::config::MotionConfig;

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
        let start = Point::new(0.0, 0.0);
        let end = Point::new(10.0, 0.0); // 10mm horizontal move

        // Profile starting from 0, going to cruise 25mm/s, ending at 0
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 10.0, 500.0);

        let planned = PlannedMove::with_profile(start, end, &profile, 80.0);

        // All generated commands should be valid
        for (i, cmd) in planned.commands.iter().enumerate() {
            assert!(cmd.is_valid(), "Command {} should be valid: {:?}", i, cmd);
        }
    }

    #[test]
    fn test_accel_phase_from_zero() {
        // Test specifically the acceleration phase starting from 0
        let start = Point::new(0.0, 0.0);
        let end = Point::new(5.0, 0.0); // Short 5mm move, likely triangular

        // Entry velocity 0, exit velocity 0, max velocity 25, short distance
        let profile = MotionProfile::calculate(0.0, 0.0, 25.0, 5.0, 500.0);

        let _planned = PlannedMove::with_profile(start, end, &profile, 80.0);
        // Just verify it doesn't panic
    }
}
