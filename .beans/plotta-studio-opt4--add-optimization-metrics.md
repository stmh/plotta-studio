---
# plotta-studio-opt4
title: Add optimization metrics and reporting
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-opt1
---

Add detailed metrics for optimization results and plot time estimation.

## Implementation

```rust
/// Statistics about a plot
#[derive(Debug, Clone)]
pub struct PlotStats {
    /// Number of strokes
    pub stroke_count: usize,
    /// Total number of points
    pub point_count: usize,
    /// Total pen-down distance (drawing)
    pub pen_down_distance: f64,
    /// Total pen-up distance (travel)
    pub pen_up_distance: f64,
    /// Number of pen lifts
    pub pen_lifts: usize,
    /// Estimated plot time in seconds
    pub estimated_time: f64,
}

impl PlotStats {
    pub fn from_strokes(strokes: &[&Stroke], config: &PlotConfig) -> Self {
        let mut pen_down = 0.0;
        let mut pen_up = 0.0;
        let mut point_count = 0;
        let mut pos = Point::ZERO;

        for stroke in strokes {
            if stroke.points.is_empty() {
                continue;
            }

            // Travel to start
            pen_up += pos.distance(stroke.points[0]);

            // Draw stroke
            for pts in stroke.points.windows(2) {
                pen_down += pts[0].distance(pts[1]);
            }

            point_count += stroke.points.len();
            pos = *stroke.points.last().unwrap();
        }

        // Return to origin
        pen_up += pos.distance(Point::ZERO);

        // Estimate time
        let draw_time = pen_down / config.pen_down_speed;
        let travel_time = pen_up / config.pen_up_speed;
        let pen_time = (strokes.len() as f64) *
            (config.pen_down_delay + config.pen_up_delay) as f64 / 1000.0;

        Self {
            stroke_count: strokes.len(),
            point_count,
            pen_down_distance: pen_down,
            pen_up_distance: pen_up,
            pen_lifts: strokes.len(),
            estimated_time: draw_time + travel_time + pen_time,
        }
    }

    pub fn total_distance(&self) -> f64 {
        self.pen_down_distance + self.pen_up_distance
    }

    pub fn format_time(&self) -> String {
        let mins = (self.estimated_time / 60.0).floor() as u32;
        let secs = (self.estimated_time % 60.0).round() as u32;
        format!("{}:{:02}", mins, secs)
    }
}

/// Compare before/after optimization
pub fn compare_optimization(
    original: &[Stroke],
    optimized: &[&Stroke],
    config: &PlotConfig,
) -> (PlotStats, PlotStats, f64) {
    let orig_refs: Vec<_> = original.iter().collect();
    let before = PlotStats::from_strokes(&orig_refs, config);
    let after = PlotStats::from_strokes(optimized, config);

    let improvement = (before.pen_up_distance - after.pen_up_distance)
        / before.pen_up_distance * 100.0;

    (before, after, improvement)
}
```

## Files to Modify
- `crates/drawing-plotter/src/lib.rs`
