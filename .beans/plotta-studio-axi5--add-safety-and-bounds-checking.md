---
# plotta-studio-axi5
title: Add safety and bounds checking
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-axi1
---

Implement safety features to prevent damage to the plotter or artwork.

## Safety Features

### Bounds Checking
```rust
pub struct PlotBounds {
    pub min: Point,
    pub max: Point,
}

impl PlotBounds {
    pub fn axidraw_v3() -> Self {
        // AxiDraw V3 work area: 297 x 218 mm (A4)
        Self {
            min: Point::ZERO,
            max: Point::new(297.0, 218.0),
        }
    }

    pub fn axidraw_v3_a3() -> Self {
        // AxiDraw V3/A3 work area: 430 x 297 mm
        Self {
            min: Point::ZERO,
            max: Point::new(430.0, 297.0),
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }
}

impl AxiDraw {
    pub fn with_bounds(mut self, bounds: PlotBounds) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn validate_drawing(&self, drawing: &Drawing) -> Result<(), PlotterError> {
        if let Some(bounds) = &self.bounds {
            let strokes = drawing.flatten();
            for stroke in &strokes {
                for point in &stroke.points {
                    if !bounds.contains(*point) {
                        return Err(PlotterError::Device(format!(
                            "Point ({:.1}, {:.1}) outside bounds",
                            point.x, point.y
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}
```

### Emergency Stop
```rust
impl AxiDraw {
    /// Emergency stop - immediately halt motors
    pub fn emergency_stop(&mut self) -> Result<(), PlotterError> {
        // Send reset command
        self.send_command("R")?;
        self.pen_is_down = false;
        Ok(())
    }
}
```

### Pause/Resume
```rust
pub struct PlotState {
    pub strokes: Vec<Stroke>,
    pub current_stroke: usize,
    pub current_point: usize,
    pub paused: bool,
}

impl AxiDraw {
    pub fn pause(&mut self) -> Result<(), PlotterError> {
        self.pen_up()?;
        // Save state for resume
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), PlotterError> {
        // Resume from saved state
        Ok(())
    }
}
```

## Files to Modify
- `crates/drawing-plotter/src/lib.rs`
