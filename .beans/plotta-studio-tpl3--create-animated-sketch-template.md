---
# plotta-studio-tpl3
title: Create animated sketch template
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-tpl1
---

Create a template for animated sketches that update each frame.

## Template Files

### cargo-generate.toml
```toml
[template]
name = "plotta-sketch-animated"
description = "An animated plotta-studio sketch"
cargo_generate_version = ">=0.18.0"

[placeholders.project-name]
type = "string"
prompt = "Project name?"
regex = "^[a-z][a-z0-9-]*$"

[placeholders.description]
type = "string"
prompt = "Description?"
default = "An animated generative drawing"

[placeholders.fps]
type = "string"
prompt = "Target FPS?"
choices = ["30", "60"]
default = "60"
```

### src/main.rs.liquid
```rust
use sketch_runner::*;

struct AnimatedSketch {
    time: f64,
    phase: f64,
}

impl Default for AnimatedSketch {
    fn default() -> Self {
        Self {
            time: 0.0,
            phase: 0.0,
        }
    }
}

impl Sketch for AnimatedSketch {
    fn setup(&mut self) -> Drawing {
        Drawing::a4_landscape()
    }

    fn update(&mut self, drawing: &mut Drawing, ctx: &UpdateContext) -> bool {
        self.time = ctx.time;
        self.phase = (ctx.time * 2.0).sin();

        // Clear and redraw
        drawing.clear();
        let center = drawing.center();

        // Animated elements
        let radius = 50.0 + self.phase * 20.0;
        drawing.add(Element::circle(center, radius));

        // Add rotating elements
        let n = 8;
        for i in 0..n {
            let angle = (i as f64 / n as f64) * std::f64::consts::TAU + ctx.time;
            let x = center.x + angle.cos() * 80.0;
            let y = center.y + angle.sin() * 80.0;
            drawing.add(Element::circle((x, y), 10.0));
        }

        true // Drawing changed
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing) {
        match key {
            Key::Character(c) if c.as_str() == "e" => {
                if let Err(e) = drawing_svg::export_svg(drawing, "{{project-name}}.svg") {
                    log::error!("Failed to export: {e}");
                } else {
                    log::info!("Exported to {{project-name}}.svg");
                }
            }
            _ => {}
        }
    }
}

fn main() {
    run_with_config(
        AnimatedSketch::default(),
        RunnerConfig::new("{{project-name}}")
            .with_animation(true)
    );
}
```

## Files to Create
- `templates/sketch-animated/cargo-generate.toml`
- `templates/sketch-animated/Cargo.toml.liquid`
- `templates/sketch-animated/src/main.rs.liquid`
