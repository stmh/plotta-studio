---
# plotta-studio-axi4
title: Implement plot drawing function
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-axi1
---

Implement the high-level `plot()` function that takes a Drawing and plots it.

## Implementation Details

```rust
impl AxiDraw {
    pub fn plot(&mut self, drawing: &Drawing, config: &PlotConfig) -> Result<(), PlotterError> {
        self.config = config.clone();

        // Flatten drawing to strokes
        let strokes = drawing.flatten();

        // Optimize stroke order
        let optimized = optimize_strokes(&strokes);

        // Plot each stroke
        self.plot_strokes(&optimized)
    }

    pub fn plot_strokes(&mut self, strokes: &[&Stroke]) -> Result<(), PlotterError> {
        // Ensure pen is up and we're at origin
        self.pen_up()?;
        self.enable_motors()?;

        for stroke in strokes {
            if stroke.points.is_empty() {
                continue;
            }

            // Move to stroke start (pen up)
            self.move_to(stroke.points[0])?;

            // Put pen down
            self.pen_down()?;

            // Draw stroke
            for point in &stroke.points[1..] {
                self.move_to(*point)?;
            }

            // Close if needed
            if stroke.closed && stroke.points.len() > 2 {
                self.move_to(stroke.points[0])?;
            }

            // Lift pen
            self.pen_up()?;
        }

        // Return home
        self.move_to(Point::ZERO)?;
        self.disable_motors()?;

        Ok(())
    }
}
```

## Progress Reporting
Consider adding a callback for progress:

```rust
pub fn plot_with_progress<F>(&mut self, drawing: &Drawing, config: &PlotConfig, progress: F)
    -> Result<(), PlotterError>
where
    F: Fn(usize, usize), // (current_stroke, total_strokes)
```

## Files to Modify
- `crates/drawing-plotter/src/lib.rs`
