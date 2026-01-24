//! Motion profile and planner

use drawing_core::Point;

use super::config::MotionConfig;
use super::segment::{
    calculate_junction_velocity, distance_to_velocity, time_to_velocity, velocity_after_acceleration,
    MotionSegment,
};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
