//! Drawing statistics for preview and time estimation

use crate::config::PlotConfig;
use crate::motion::{MotionConfig, MotionPlanner, MotionProfile};
use crate::optimize::{
    optimize_strokes_with_reversal, pen_down_distance_optimized, travel_distance_optimized,
};
use drawing_core::{Point, Stroke};
use std::time::Duration;

/// Statistics about a drawing for preview and time estimation
#[derive(Debug, Clone)]
pub struct DrawingStats {
    /// Total number of strokes
    pub stroke_count: usize,
    /// Total pen-down distance (mm)
    pub pen_down_distance: f64,
    /// Total pen-up travel distance (mm)
    pub travel_distance: f64,
    /// Estimated plot time
    pub estimated_time: Duration,
    /// Number of strokes that will be drawn in reverse
    pub reversed_strokes: usize,
    /// Number of strokes folded into a previous stroke by the adjacent-stroke
    /// merge optimization. Each merge saves one pen-up + pen-down cycle.
    pub merged_strokes: usize,
}

impl DrawingStats {
    /// Calculate statistics for a set of strokes
    pub fn calculate(strokes: &[Stroke], config: &PlotConfig) -> Self {
        // Optimize with reversal support (same as actual plotting)
        let optimized = optimize_strokes_with_reversal(strokes, true);

        let pen_down = pen_down_distance_optimized(&optimized);
        let travel = travel_distance_optimized(&optimized);
        let reversed_count = optimized.iter().filter(|s| s.reversed).count();

        let time = estimate_plot_time_optimized(&optimized, config);

        Self {
            stroke_count: strokes.len(),
            pen_down_distance: pen_down,
            travel_distance: travel,
            estimated_time: time,
            reversed_strokes: reversed_count,
            merged_strokes: 0,
        }
    }

    /// Format the estimated time as a human-readable string
    pub fn format_time(&self) -> String {
        let secs = self.estimated_time.as_secs();
        if secs < 60 {
            format!("~{}s", secs)
        } else if secs < 3600 {
            let mins = secs / 60;
            let remaining_secs = secs % 60;
            format!("~{}m {}s", mins, remaining_secs)
        } else {
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            format!("~{}h {}m", hours, mins)
        }
    }
}

use crate::optimize::OwnedOptimizedStroke;

/// Estimate plot time for optimized strokes
///
/// Takes into account:
/// - Pen-down drawing speed
/// - Pen-up travel speed (including the return home after the last stroke)
/// - Pen up/down delays
/// - Stroke count (for pen transitions)
/// - Motion planning (acceleration/deceleration) when enabled
pub fn estimate_plot_time_optimized(
    strokes: &[OwnedOptimizedStroke],
    config: &PlotConfig,
) -> Duration {
    if strokes.is_empty() {
        return Duration::ZERO;
    }

    let (pen_down_time_ms, travel_time_ms) = if config.motion_planning_enabled {
        // Use motion planning for more accurate time estimation
        estimate_time_with_motion_planning(strokes, config)
    } else {
        // Simple constant velocity calculation
        let pen_down = pen_down_distance_optimized(strokes);
        let travel = travel_distance_optimized(strokes);

        let pen_down_time = (pen_down / config.pen_down_speed) * 1000.0;
        let travel_time = (travel / config.pen_up_speed) * 1000.0;

        (pen_down_time, travel_time)
    };

    // Time for pen transitions (each stroke: pen down + pen up)
    // Uses total time which includes servo move time + optional delay
    let pen_transition_time_ms =
        (config.pen_down_total_time() + config.pen_up_total_time()) as f64 * strokes.len() as f64;

    let total_ms = pen_down_time_ms + travel_time_ms + pen_transition_time_ms;

    Duration::from_millis(total_ms as u64)
}

/// Estimate time with motion planning (acceleration/deceleration)
///
/// This calculates the actual time considering:
/// - Acceleration from rest at the start of each stroke
/// - Deceleration at corners based on angle
/// - Triangular profiles for short segments
fn estimate_time_with_motion_planning(
    strokes: &[OwnedOptimizedStroke],
    config: &PlotConfig,
) -> (f64, f64) {
    let motion_config = MotionConfig {
        max_velocity: config.pen_down_speed,
        max_acceleration: config.max_acceleration,
        junction_deviation: config.junction_deviation,
        steps_per_mm: 80.0, // Standard AxiDraw steps/mm
    };

    let planner = MotionPlanner::new(motion_config);
    let mut total_pen_down_time = 0.0;
    let mut total_travel_time = 0.0;

    let mut current_pos = Point::new(0.0, 0.0);

    // Travel moves start and end at rest and can reach max travel speed
    let travel_time_ms = |distance: f64| {
        if distance > 0.01 {
            MotionProfile::calculate(
                0.0,
                0.0,
                config.pen_up_speed,
                distance,
                config.max_acceleration,
            )
            .total_time()
                * 1000.0
        } else {
            0.0
        }
    };

    for stroke in strokes {
        let points: Vec<Point> = stroke.points_iter().collect();
        if points.is_empty() {
            continue;
        }

        let stroke_start = points[0];
        let stroke_end = points[points.len() - 1];

        // Calculate travel time to stroke start
        total_travel_time += travel_time_ms((stroke_start - current_pos).length());

        // Calculate drawing time for this stroke using motion planner
        if points.len() >= 2 {
            let segments = planner.plan(&points);
            let profiles = planner.generate_profiles(&segments);
            for profile in &profiles {
                total_pen_down_time += profile.total_time() * 1000.0;
            }
        }

        current_pos = stroke_end;
    }

    // Return home after the last stroke
    total_travel_time += travel_time_ms((Point::ZERO - current_pos).length());

    (total_pen_down_time, total_travel_time)
}

/// Estimate plot time for a set of strokes (legacy API, not optimized)
pub fn estimate_plot_time(strokes: &[Stroke], config: &PlotConfig) -> Duration {
    use crate::optimize::optimize_strokes_with_reversal;
    let optimized = optimize_strokes_with_reversal(strokes, false);
    estimate_plot_time_optimized(&optimized, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_core::{Point, ResolvedStyle};

    #[test]
    fn test_drawing_stats_empty() {
        let strokes: Vec<Stroke> = vec![];
        let stats = DrawingStats::calculate(&strokes, &PlotConfig::default());

        assert_eq!(stats.stroke_count, 0);
        assert!((stats.pen_down_distance - 0.0).abs() < 0.001);
        assert!((stats.travel_distance - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_drawing_stats_single_stroke() {
        let strokes = vec![Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            ResolvedStyle::default(),
        )];
        let stats = DrawingStats::calculate(&strokes, &PlotConfig::default());

        assert_eq!(stats.stroke_count, 1);
        assert!((stats.pen_down_distance - 100.0).abs() < 0.001);
        assert!((stats.travel_distance - 100.0).abs() < 0.001); // Starts at origin, 100 back home
    }

    #[test]
    fn test_drawing_stats_with_travel() {
        let strokes = vec![
            Stroke::line(
                Point::new(10.0, 0.0), // 10mm from origin
                Point::new(20.0, 0.0), // 10mm stroke
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(30.0, 0.0), // 10mm travel from previous end
                Point::new(40.0, 0.0), // 10mm stroke
                ResolvedStyle::default(),
            ),
        ];
        let stats = DrawingStats::calculate(&strokes, &PlotConfig::default());

        assert_eq!(stats.stroke_count, 2);
        assert!((stats.pen_down_distance - 20.0).abs() < 0.001);
        // 10 to start + 10 between + 40 back home
        assert!((stats.travel_distance - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_estimate_plot_time_empty() {
        let strokes: Vec<Stroke> = vec![];
        let time = estimate_plot_time(&strokes, &PlotConfig::default());
        assert_eq!(time, Duration::ZERO);
    }

    #[test]
    fn test_estimate_plot_time_constant_velocity() {
        let strokes = vec![Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(25.0, 0.0), // 25mm at 25mm/s = 1s pen down
            ResolvedStyle::default(),
        )];
        // Disable motion planning for constant velocity test
        let config = PlotConfig {
            motion_planning_enabled: false,
            ..PlotConfig::default()
        };
        let time = estimate_plot_time(&strokes, &config);

        // Should include drawing time + return travel + servo timing (rate-adjusted) + settle delay
        // Drawing: 25mm / 25mm/s = 1000ms
        // Return home: 25mm / 75mm/s = 333ms
        // Servo timing with both rates at 50:
        //   pen_down (rate=50): 251ms move + 50ms default settle delay = 301ms
        //   pen_up   (rate=50): 251ms move + 0ms delay                = 251ms
        // Total: ~1885ms
        assert!(time.as_millis() >= 1845);
        assert!(time.as_millis() <= 1925);
    }

    #[test]
    fn test_estimate_plot_time_includes_return_home() {
        // Drawing (0,0)->(25,0) ends 25mm from home; drawing it the other way
        // starts 25mm from home and ends there. Both need 25mm of pen-up travel.
        let away = OwnedOptimizedStroke {
            points: vec![Point::new(0.0, 0.0), Point::new(25.0, 0.0)],
            style: ResolvedStyle::default(),
            closed: false,
            reversed: false,
        };
        let back = OwnedOptimizedStroke {
            reversed: true,
            ..away.clone()
        };
        for motion_planning_enabled in [false, true] {
            let config = PlotConfig {
                motion_planning_enabled,
                ..PlotConfig::default()
            };
            let t_away = estimate_plot_time_optimized(std::slice::from_ref(&away), &config);
            let t_back = estimate_plot_time_optimized(std::slice::from_ref(&back), &config);
            assert!(
                t_away.as_millis().abs_diff(t_back.as_millis()) <= 1,
                "motion_planning={}: {:?} vs {:?}",
                motion_planning_enabled,
                t_away,
                t_back
            );
        }
    }

    #[test]
    fn test_estimate_plot_time_with_motion_planning() {
        let strokes = vec![Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(25.0, 0.0), // 25mm
            ResolvedStyle::default(),
        )];
        let config = PlotConfig {
            motion_planning_enabled: true,
            ..PlotConfig::default()
        };
        let time = estimate_plot_time(&strokes, &config);

        // With motion planning, time will be longer due to accel/decel
        // from rest to rest. The time will be > constant velocity estimate.
        // Constant velocity: 25mm / 25mm/s = 1000ms
        // With accel/decel starting/stopping at 0, we need more time
        // Expected: ~1500-2500ms for motion + ~500ms for servo timing
        assert!(
            time.as_millis() >= 1500,
            "Motion planned time should be >= 1500ms, got {}ms",
            time.as_millis()
        );
        assert!(
            time.as_millis() <= 3000,
            "Motion planned time should be <= 3000ms, got {}ms",
            time.as_millis()
        );
    }

    #[test]
    fn test_format_time_seconds() {
        let stats = DrawingStats {
            stroke_count: 1,
            pen_down_distance: 0.0,
            travel_distance: 0.0,
            estimated_time: Duration::from_secs(45),
            reversed_strokes: 0,
            merged_strokes: 0,
        };
        assert_eq!(stats.format_time(), "~45s");
    }

    #[test]
    fn test_format_time_minutes() {
        let stats = DrawingStats {
            stroke_count: 1,
            pen_down_distance: 0.0,
            travel_distance: 0.0,
            estimated_time: Duration::from_secs(185), // 3m 5s
            reversed_strokes: 0,
            merged_strokes: 0,
        };
        assert_eq!(stats.format_time(), "~3m 5s");
    }

    #[test]
    fn test_format_time_hours() {
        let stats = DrawingStats {
            stroke_count: 1,
            pen_down_distance: 0.0,
            travel_distance: 0.0,
            reversed_strokes: 0,
            merged_strokes: 0,
            estimated_time: Duration::from_secs(3720), // 1h 2m
        };
        assert_eq!(stats.format_time(), "~1h 2m");
    }
}
