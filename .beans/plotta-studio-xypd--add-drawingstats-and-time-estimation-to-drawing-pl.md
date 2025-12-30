---
# plotta-studio-xypd
title: Add DrawingStats and time estimation to drawing-plotter
status: todo
type: task
priority: normal
created_at: 2025-12-28T15:31:41Z
updated_at: 2025-12-28T15:32:10Z
parent: plotta-studio-7wzz
blocking:
    - plotta-studio-vd8d
---

Add reusable statistics and time estimation functions to `drawing-plotter` crate.

## New Types

```rust
pub struct DrawingStats {
    pub stroke_count: usize,
    pub pen_down_distance: f64,  // mm
    pub travel_distance: f64,    // mm (pen-up movement)
    pub estimated_time: Duration,
}
```

## New Functions

```rust
/// Calculate statistics for a drawing
pub fn calculate_stats(drawing: &Drawing, config: &PlotConfig) -> DrawingStats

/// Estimate plotting time based on strokes and config
pub fn estimate_plot_time(strokes: &[Stroke], config: &PlotConfig) -> Duration
```

## Implementation Notes

- Use existing `pen_down_distance()` and `total_travel_distance()` functions
- Time estimation formula:
  - pen-down time = pen_down_distance / pen_down_speed
  - travel time = travel_distance / pen_up_speed
  - pen transitions = stroke_count * 2 * (pen_down_delay + pen_up_delay) / 2
  - total = pen-down time + travel time + pen transitions

## Location

Add to `crates/drawing-plotter/src/stats.rs` (new file) and re-export from `lib.rs`