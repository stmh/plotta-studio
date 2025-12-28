//! Drawing statistics for preview and time estimation

use crate::config::PlotConfig;
use crate::optimize::{
    optimize_strokes_with_reversal, pen_down_distance_optimized, travel_distance_optimized,
};
use drawing_core::Stroke;
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

use crate::optimize::OptimizedStroke;

/// Estimate plot time for optimized strokes
///
/// Takes into account:
/// - Pen-down drawing speed
/// - Pen-up travel speed
/// - Pen up/down delays
/// - Stroke count (for pen transitions)
pub fn estimate_plot_time_optimized(
    strokes: &[OptimizedStroke<'_>],
    config: &PlotConfig,
) -> Duration {
    if strokes.is_empty() {
        return Duration::ZERO;
    }

    let pen_down = pen_down_distance_optimized(strokes);
    let travel = travel_distance_optimized(strokes);

    // Time for drawing (pen down)
    let pen_down_time_ms = (pen_down / config.pen_down_speed) * 1000.0;

    // Time for travel (pen up)
    let travel_time_ms = (travel / config.pen_up_speed) * 1000.0;

    // Time for pen transitions (each stroke: pen down + pen up)
    let pen_transitions = strokes.len() * 2; // Up and down for each stroke
    let transition_time_ms =
        (config.pen_down_delay + config.pen_up_delay) as f64 * pen_transitions as f64;

    let total_ms = pen_down_time_ms + travel_time_ms + transition_time_ms;

    Duration::from_millis(total_ms as u64)
}

/// Estimate plot time for a set of strokes (legacy API)
pub fn estimate_plot_time(strokes: &[&Stroke], config: &PlotConfig) -> Duration {
    let optimized: Vec<_> = strokes
        .iter()
        .map(|s| OptimizedStroke::new(s, false))
        .collect();
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
        assert!((stats.travel_distance - 0.0).abs() < 0.001); // Starts at origin
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
        assert!((stats.travel_distance - 20.0).abs() < 0.001); // 10 to start + 10 between
    }

    #[test]
    fn test_estimate_plot_time_empty() {
        let strokes: Vec<&Stroke> = vec![];
        let time = estimate_plot_time(&strokes, &PlotConfig::default());
        assert_eq!(time, Duration::ZERO);
    }

    #[test]
    fn test_estimate_plot_time_includes_delays() {
        let stroke = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(25.0, 0.0), // 25mm at 25mm/s = 1s pen down
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke];
        let config = PlotConfig::default();
        let time = estimate_plot_time(&strokes, &config);

        // Should include drawing time + pen delays
        // Drawing: 25mm / 25mm/s = 1000ms
        // Transitions: 2 * (150 + 150) = 600ms
        // Total: ~1600ms
        assert!(time.as_millis() >= 1500);
        assert!(time.as_millis() <= 1700);
    }

    #[test]
    fn test_format_time_seconds() {
        let stats = DrawingStats {
            stroke_count: 1,
            pen_down_distance: 0.0,
            travel_distance: 0.0,
            estimated_time: Duration::from_secs(45),
            reversed_strokes: 0,
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
            estimated_time: Duration::from_secs(3720), // 1h 2m
        };
        assert_eq!(stats.format_time(), "~1h 2m");
    }
}
