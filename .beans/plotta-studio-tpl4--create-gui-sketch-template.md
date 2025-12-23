---
# plotta-studio-tpl4
title: Create GUI sketch template
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-tpl1
---

Create a template for sketches with egui parameter controls.

## Template Files

### cargo-generate.toml
```toml
[template]
name = "plotta-sketch-gui"
description = "A plotta-studio sketch with GUI controls"
cargo_generate_version = ">=0.18.0"

[placeholders.project-name]
type = "string"
prompt = "Project name?"
regex = "^[a-z][a-z0-9-]*$"

[placeholders.description]
type = "string"
prompt = "Description?"
default = "An interactive generative drawing"
```

### src/main.rs.liquid
```rust
use sketch_runner::*;

struct GuiSketch {
    // Parameters (shown in GUI)
    count: i32,
    radius: f64,
    spacing: f64,
    stroke_width: f64,
    stroke_color: Color,
    seed: u64,
}

impl Default for GuiSketch {
    fn default() -> Self {
        Self {
            count: 10,
            radius: 30.0,
            spacing: 15.0,
            stroke_width: 1.0,
            stroke_color: Color::BLACK,
            seed: 42,
        }
    }
}

impl GuiSketch {
    fn regenerate(&self) -> Drawing {
        let mut drawing = Drawing::a4_landscape();
        let center = drawing.center();

        // Your generative algorithm here
        for i in 0..self.count {
            let r = self.radius + (i as f64) * self.spacing;
            drawing.add(
                Element::circle(center, r)
                    .stroke_width(self.stroke_width)
                    .stroke_color(self.stroke_color)
            );
        }

        drawing
    }
}

impl Sketch for GuiSketch {
    fn setup(&mut self) -> Drawing {
        self.regenerate()
    }
}

impl SketchWithGui for GuiSketch {
    fn gui(&mut self, ui: &mut egui::Ui, drawing: &mut Drawing) -> bool {
        let mut changed = false;

        ui.heading("{{project-name}}");
        ui.separator();

        // Pattern parameters
        changed |= section(ui, "Pattern", |ui| {
            let mut c = false;
            c |= int_slider(ui, "Count", &mut self.count, 1, 50);
            c |= range_slider(ui, "Radius", &mut self.radius, 5.0, 100.0);
            c |= range_slider(ui, "Spacing", &mut self.spacing, 0.0, 30.0);
            c
        });

        // Style parameters
        changed |= section(ui, "Style", |ui| {
            let mut c = false;
            c |= range_slider(ui, "Stroke Width", &mut self.stroke_width, 0.1, 5.0);
            c |= color_widget(ui, "Color", &mut self.stroke_color);
            c
        });

        // Randomization
        changed |= section(ui, "Random", |ui| {
            seed_widget(ui, &mut self.seed)
        });

        // Regenerate if changed
        if changed {
            *drawing = self.regenerate();
        }

        ui.separator();

        // Export button
        if ui.button("Export SVG").clicked() {
            if let Err(e) = drawing_svg::export_svg(drawing, "{{project-name}}.svg") {
                log::error!("Export failed: {e}");
            } else {
                log::info!("Exported to {{project-name}}.svg");
            }
        }

        changed
    }
}

fn main() {
    run_with_gui(GuiSketch::default());
}
```

## Note
This template depends on the egui integration epic (plotta-studio-gui1) being completed first.

## Files to Create
- `templates/sketch-gui/cargo-generate.toml`
- `templates/sketch-gui/Cargo.toml.liquid`
- `templates/sketch-gui/src/main.rs.liquid`
