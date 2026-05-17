//! Configuration for plotting

use crate::motion::{
    MotionConfig, DEFAULT_ACCEL_PEN_DOWN, DEFAULT_ACCEL_PEN_UP, DEFAULT_JUNCTION_DEVIATION,
};

/// Servo timing constants from Python AxiDraw driver
/// These are used to calculate how long to wait for the servo to physically move.
pub mod servo {
    /// Minimum time (ms) for pen lift/lower of non-zero distance at 100% rate
    /// From axidraw_conf.py: servo_move_min = 45
    pub const MOVE_MIN_MS: f64 = 45.0;

    /// Additional time (ms) per percent of vertical travel at 100% rate
    /// From axidraw_conf.py: servo_move_slope = 2.69
    pub const MOVE_SLOPE_MS_PER_PERCENT: f64 = 2.69;

    /// EBB servo channel period in ms (8 channels * 3ms each)
    /// The servo rate is applied once per period.
    #[cfg_attr(not(feature = "hardware"), allow(dead_code))]
    const SERVO_PERIOD_MS: f64 = 24.0;

    /// Minimum servo pulse width (1ms = ~7500 EBB units)
    #[cfg_attr(not(feature = "hardware"), allow(dead_code))]
    const SERVO_MIN: f64 = 7500.0;

    /// Maximum servo pulse width (2ms = ~28000 EBB units)
    #[cfg_attr(not(feature = "hardware"), allow(dead_code))]
    const SERVO_MAX: f64 = 28000.0;

    /// Calculate servo move time based on pen position delta and rate
    ///
    /// Formula from Python driver: time = (slope * distance + min) * (100 / rate)
    /// where:
    /// - distance is the absolute difference in pen position (0-100 scale)
    /// - rate is the servo speed (1-100, where 100 is fastest)
    ///
    /// At 100% rate, the formula simplifies to: time = slope * distance + min
    pub fn calculate_move_time(from_pos: u8, to_pos: u8, rate: u8) -> u32 {
        let distance = (from_pos as i16 - to_pos as i16).unsigned_abs() as f64;
        if distance < 0.001 {
            return 0;
        }
        let rate = rate.clamp(1, 100) as f64;
        let base_time_ms = MOVE_SLOPE_MS_PER_PERCENT * distance + MOVE_MIN_MS;
        // Scale time inversely by rate (lower rate = longer time)
        let time_ms = base_time_ms * (100.0 / rate);
        time_ms.round() as u32
    }

    /// Calculate the EBB servo rate value (for SC,11/SC,12) from pen positions and rate
    ///
    /// The EBB servo rate is how much the pulse width changes per servo period (24ms).
    /// We calculate this to match the timing from our `calculate_move_time` formula.
    ///
    /// Formula: rate_ebb = distance_ebb_units * SERVO_PERIOD_MS / time_ms
    #[cfg_attr(not(feature = "hardware"), allow(dead_code))]
    pub fn calculate_ebb_rate(from_pos: u8, to_pos: u8, rate: u8) -> u16 {
        let distance_percent = (from_pos as i16 - to_pos as i16).unsigned_abs() as f64;
        if distance_percent < 0.001 {
            return 0;
        }

        // Calculate distance in EBB units
        let from_ebb = SERVO_MIN + (SERVO_MAX - SERVO_MIN) * (from_pos as f64) / 100.0;
        let to_ebb = SERVO_MIN + (SERVO_MAX - SERVO_MIN) * (to_pos as f64) / 100.0;
        let distance_ebb = (from_ebb - to_ebb).abs();

        // Calculate expected move time from our formula
        let time_ms = calculate_move_time(from_pos, to_pos, rate) as f64;
        if time_ms < 1.0 {
            return 0;
        }

        // Calculate rate: change per period = distance / (time / period)
        let rate_ebb = distance_ebb * SERVO_PERIOD_MS / time_ms;

        // Clamp to valid range (1-65535)
        rate_ebb.round().clamp(1.0, 65535.0) as u16
    }
}

/// Configuration for plotting
#[derive(Debug, Clone)]
pub struct PlotConfig {
    /// Speed for pen-down movement (mm/s)
    pub pen_down_speed: f64,
    /// Speed for pen-up movement (mm/s)
    pub pen_up_speed: f64,
    /// Pen down position (servo units, 0-100)
    pub pen_down_pos: u8,
    /// Pen up position (servo units, 0-100)
    pub pen_up_pos: u8,
    /// Rate of raising pen (1-100, where 100 is fastest)
    /// From axidraw_conf.py: pen_rate_raise = 75
    pub pen_rate_raise: u8,
    /// Rate of lowering pen (1-100, where 100 is fastest)
    /// From axidraw_conf.py: pen_rate_lower = 50
    pub pen_rate_lower: u8,
    /// Optional additional delay after pen down (ms) - added after servo move completes
    pub pen_down_delay: u32,
    /// Optional additional delay after pen up (ms) - added after servo move completes
    pub pen_up_delay: u32,

    // Motion planning configuration
    /// Maximum acceleration for pen-down moves (mm/s^2)
    /// Higher values allow faster direction changes but may cause harsher motor sounds.
    pub max_acceleration: f64,
    /// Maximum acceleration for pen-up (travel) moves (mm/s^2)
    /// Used when planning inter-stroke travel. Typically higher than
    /// `max_acceleration` because the pen is not in contact with paper.
    pub pen_up_acceleration: f64,
    /// Junction deviation for corner velocity calculation (mm)
    /// Controls the trade-off between speed and accuracy at corners.
    /// Smaller values = slower corners, larger = faster but less accurate.
    pub junction_deviation: f64,
    /// Enable motion planning for pen-down moves
    /// When enabled, uses trapezoidal velocity profiles with corner velocity planning.
    /// When disabled, uses constant velocity (legacy behavior).
    pub motion_planning_enabled: bool,

    /// Enable position verification logging (diagnostic mode)
    /// When enabled, queries hardware position (QS) after each stroke and logs
    /// any discrepancy between tracked and actual position.
    /// Warning: Adds ~10ms latency per stroke. Use for diagnosing drift issues only.
    pub verify_position: bool,

    /// Merge adjacent strokes whose endpoints are within `merge_tolerance`.
    /// Reduces pen up/down cycles when consecutive optimized strokes meet at
    /// (or very near) the same point. Enabled by default.
    pub merge_strokes: bool,
    /// Distance tolerance (mm) for considering two stroke endpoints to be the
    /// same point during stroke merging. Defaults to 0.05mm (matches the curve
    /// flattening tolerance).
    pub merge_tolerance: f64,

    /// Also connect adjacent strokes whose endpoints are within
    /// `connect_distance_factor * stroke_width` of each other. The short gap
    /// gets bridged by an extra pen-down segment that the pen's own width hides
    /// visually. Enabled by default.
    pub connect_close_strokes: bool,
    /// Fraction of the stroke width to use as the connect threshold. Defaults
    /// to `0.5` — a gap up to half the pen width gets bridged.
    pub connect_distance_factor: f64,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            pen_down_speed: 25.0,
            pen_up_speed: 75.0,
            // Default pen positions matching Python driver
            pen_down_pos: 30,
            pen_up_pos: 60,
            // Servo rates - using 50 for both (slower than Python's 75/50)
            // to prevent ghost lines from insufficient pen-up time
            pen_rate_raise: 50,
            pen_rate_lower: 50,
            // Default pen-down settle delay (ms). Gives the pen tip time to
            // stop bouncing after the servo reaches the down position before
            // motion begins. Without this, short detail strokes (i-dots,
            // t-crossbars) can record servo/tip oscillation as positional
            // skew. The Inkscape AxiDraw extension uses a similar settle.
            pen_down_delay: 50,
            pen_up_delay: 0,
            // Motion planning defaults (matches Python AxiDraw driver)
            max_acceleration: DEFAULT_ACCEL_PEN_DOWN,
            pen_up_acceleration: DEFAULT_ACCEL_PEN_UP,
            junction_deviation: DEFAULT_JUNCTION_DEVIATION,
            motion_planning_enabled: true,
            // Position verification disabled by default (diagnostic feature)
            verify_position: false,
            // Stroke merging on by default - cheap, and reduces pen wear.
            merge_strokes: true,
            merge_tolerance: 0.05,
            // Bridge gaps up to half a pen width by default.
            connect_close_strokes: true,
            connect_distance_factor: 0.5,
        }
    }
}

impl PlotConfig {
    /// Calculate the servo move time for pen up (from down to up position)
    pub fn pen_up_move_time(&self) -> u32 {
        servo::calculate_move_time(self.pen_down_pos, self.pen_up_pos, self.pen_rate_raise)
    }

    /// Calculate the servo move time for pen down (from up to down position)
    pub fn pen_down_move_time(&self) -> u32 {
        servo::calculate_move_time(self.pen_up_pos, self.pen_down_pos, self.pen_rate_lower)
    }

    /// Total time to wait after pen up (servo move + optional delay)
    pub fn pen_up_total_time(&self) -> u32 {
        self.pen_up_move_time() + self.pen_up_delay
    }

    /// Total time to wait after pen down (servo move + optional delay)
    pub fn pen_down_total_time(&self) -> u32 {
        self.pen_down_move_time() + self.pen_down_delay
    }

    /// Create a MotionConfig for pen-down (drawing) moves from this PlotConfig.
    pub fn motion_config(&self) -> MotionConfig {
        MotionConfig {
            max_velocity: self.pen_down_speed,
            max_acceleration: self.max_acceleration,
            junction_deviation: self.junction_deviation,
            steps_per_mm: 80.0, // AxiDraw constant
        }
    }

    /// Create a MotionConfig for pen-up (travel) moves from this PlotConfig.
    ///
    /// Uses `pen_up_speed` and `pen_up_acceleration` so that inter-stroke
    /// travel uses trapezoidal velocity profiles instead of constant velocity,
    /// reducing touchdown jerk on the next stroke.
    pub fn motion_config_pen_up(&self) -> MotionConfig {
        MotionConfig {
            max_velocity: self.pen_up_speed,
            max_acceleration: self.pen_up_acceleration,
            junction_deviation: self.junction_deviation,
            steps_per_mm: 80.0,
        }
    }

    /// Enable motion planning with custom acceleration
    pub fn with_motion_planning(mut self, max_acceleration: f64) -> Self {
        self.motion_planning_enabled = true;
        self.max_acceleration = max_acceleration;
        self
    }

    /// Disable motion planning (use constant velocity)
    pub fn without_motion_planning(mut self) -> Self {
        self.motion_planning_enabled = false;
        self
    }

    /// Enable stroke merging with a custom tolerance (mm).
    pub fn with_stroke_merging(mut self, tolerance: f64) -> Self {
        self.merge_strokes = true;
        self.merge_tolerance = tolerance;
        self
    }

    /// Disable stroke merging.
    pub fn without_stroke_merging(mut self) -> Self {
        self.merge_strokes = false;
        self
    }

    /// Enable bridging of close strokes with a custom width factor.
    /// Gaps up to `factor * stroke_width` get bridged.
    pub fn with_close_stroke_bridging(mut self, factor: f64) -> Self {
        self.connect_close_strokes = true;
        self.connect_distance_factor = factor;
        self
    }

    /// Disable bridging of close strokes.
    pub fn without_close_stroke_bridging(mut self) -> Self {
        self.connect_close_strokes = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servo_timing_calculation_at_100_percent_rate() {
        // From Python: servo_move_slope = 2.69, servo_move_min = 45
        // At 100% rate, time = slope * distance + min
        // For 30% travel (from 30 to 60): 2.69 * 30 + 45 = 125.7 -> 126ms
        let time = servo::calculate_move_time(30, 60, 100);
        assert_eq!(time, 126);

        // Same distance in reverse direction
        let time_reverse = servo::calculate_move_time(60, 30, 100);
        assert_eq!(time_reverse, 126);

        // Zero distance = 0 time
        let zero_time = servo::calculate_move_time(50, 50, 100);
        assert_eq!(zero_time, 0);

        // Full range (0 to 100): 2.69 * 100 + 45 = 314ms
        let full_time = servo::calculate_move_time(0, 100, 100);
        assert_eq!(full_time, 314);
    }

    #[test]
    fn test_servo_timing_with_rate_scaling() {
        // At 50% rate, time is doubled: (slope * distance + min) * (100 / 50)
        // For 30% travel: (2.69 * 30 + 45) * 2 = 125.7 * 2 = 251.4 -> 251ms
        let time_50 = servo::calculate_move_time(30, 60, 50);
        assert_eq!(time_50, 251);

        // At 75% rate (Python default for pen raise): (125.7) * (100/75) = 167.6 -> 168ms
        let time_75 = servo::calculate_move_time(30, 60, 75);
        assert_eq!(time_75, 168);

        // At 25% rate, time is 4x: 125.7 * 4 = 502.8 -> 503ms
        let time_25 = servo::calculate_move_time(30, 60, 25);
        assert_eq!(time_25, 503);

        // Rate is clamped to minimum of 1 (no division by zero)
        let time_min = servo::calculate_move_time(30, 60, 0);
        // Clamped to rate=1: 125.7 * 100 = 12570ms
        assert_eq!(time_min, 12570);
    }

    #[test]
    fn test_default_config_timing() {
        let config = PlotConfig::default();

        // Default: pen_down_pos = 30, pen_up_pos = 60, delta = 30%
        // Base time at 100% rate: 2.69 * 30 + 45 = 125.7ms

        // Both rates are 50 (slower than Python defaults to prevent ghost lines)
        // Time = 125.7 * (100/50) = 251.4 -> 251ms
        assert_eq!(config.pen_up_move_time(), 251);
        assert_eq!(config.pen_down_move_time(), 251);

        // Pen-up has no extra delay by default: total == move time.
        assert_eq!(config.pen_up_total_time(), 251);
        // Pen-down has a default 50ms settle delay to prevent tip-bounce
        // ghosting on short detail strokes (i-dots, t-crossbars).
        assert_eq!(config.pen_down_total_time(), 301); // 251 + 50
    }

    #[test]
    fn test_config_with_additional_delay() {
        let config = PlotConfig {
            pen_down_delay: 50,
            pen_up_delay: 100,
            ..PlotConfig::default()
        };

        // Move times use default rates (both at 50)
        assert_eq!(config.pen_up_move_time(), 251);
        assert_eq!(config.pen_down_move_time(), 251);

        // Total time includes the additional delay
        assert_eq!(config.pen_up_total_time(), 351); // 251 + 100
        assert_eq!(config.pen_down_total_time(), 301); // 251 + 50
    }

    #[test]
    fn test_calculate_ebb_rate() {
        // Test that EBB rate is consistent with timing calculation
        // For 30% travel (from 30 to 60) at 100% rate: time = 126ms
        // Distance in EBB units: (28000-7500) * 30 / 100 = 6150
        // Rate = 6150 * 24 / 126 = 1171.4
        let rate = servo::calculate_ebb_rate(30, 60, 100);
        assert!(
            (1170..=1172).contains(&rate),
            "Expected ~1171, got {}",
            rate
        );

        // At 50% rate: time = 251ms
        // Rate = 6150 * 24 / 251 = 588
        let rate_50 = servo::calculate_ebb_rate(30, 60, 50);
        assert!(
            (587..=589).contains(&rate_50),
            "Expected ~588, got {}",
            rate_50
        );

        // Zero distance should return 0
        let rate_zero = servo::calculate_ebb_rate(50, 50, 50);
        assert_eq!(rate_zero, 0);
    }
}
