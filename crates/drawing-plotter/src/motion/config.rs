//! Motion configuration and constants

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motion_config_default() {
        let config = MotionConfig::default();
        assert_eq!(config.max_velocity, 25.0);
        assert_eq!(config.max_acceleration, DEFAULT_ACCEL_PEN_DOWN); // 1016.0 mm/s²
        assert_eq!(config.steps_per_mm, 80.0);
    }
}
