//! Motion segment and junction velocity calculations

use drawing_core::Point;

use super::config::MotionConfig;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

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
}
