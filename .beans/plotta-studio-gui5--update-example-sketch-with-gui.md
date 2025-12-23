---
# plotta-studio-gui5
title: Update example sketch with GUI
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-gui1
---

Update sketch-001 to demonstrate GUI parameter usage.

## Updated Example Sketch

```rust
// sketches/sketch-001/src/main.rs
use sketch_runner::*;

struct MySketch {
    // Parameters
    circle_count: i32,
    base_radius: f64,
    spacing: f64,
    rotation: f64,
    stroke_width: f64,
    stroke_color: Color,
    seed: u64,
}

impl Default for MySketch {
    fn default() -> Self {
        Self {
            circle_count: 12,
            base_radius: 30.0,
            spacing: 15.0,
            rotation: 0.0,
            stroke_width: 1.0,
            stroke_color: Color::BLACK,
            seed: 42,
        }
    }
}

impl Sketch for MySketch {
    fn setup(&mut self) -> Drawing {
        self.regenerate()
    }

    fn update(&mut self, drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        false
    }
}

impl SketchWithGui for MySketch {
    fn gui(&mut self, ui: &mut egui::Ui, drawing: &mut Drawing) -> bool {
        let mut changed = false;

        ui.heading("Circle Pattern");

        changed |= section(ui, "Pattern", |ui| {
            let mut c = false;
            c |= int_slider(ui, "Count", &mut self.circle_count, 1, 50);
            c |= range_slider(ui, "Base Radius", &mut self.base_radius, 5.0, 100.0);
            c |= range_slider(ui, "Spacing", &mut self.spacing, 0.0, 50.0);
            c |= range_slider(ui, "Rotation", &mut self.rotation, 0.0, 360.0);
            c
        });

        changed |= section(ui, "Style", |ui| {
            let mut c = false;
            c |= range_slider(ui, "Stroke Width", &mut self.stroke_width, 0.1, 5.0);
            c |= color_widget(ui, "Color", &mut self.stroke_color);
            c
        });

        changed |= section(ui, "Random", |ui| {
            seed_widget(ui, &mut self.seed)
        });

        if changed {
            *drawing = self.regenerate();
        }

        ui.separator();

        if ui.button("Export SVG").clicked() {
            drawing_svg::export_svg(drawing, "output.svg").ok();
        }

        changed
    }
}

impl MySketch {
    fn regenerate(&self) -> Drawing {
        let mut drawing = Drawing::a4_landscape();
        let center = drawing.center();

        for i in 0..self.circle_count {
            let radius = self.base_radius + (i as f64) * self.spacing;
            drawing.add(
                Element::circle(center, radius)
                    .rotate_deg(self.rotation * (i as f64) / self.circle_count as f64)
                    .stroke_width(self.stroke_width)
                    .stroke_color(self.stroke_color)
            );
        }

        drawing
    }
}

fn main() {
    run_with_gui(MySketch::default());
}
```

## Files to Modify
- `sketches/sketch-001/src/main.rs`
