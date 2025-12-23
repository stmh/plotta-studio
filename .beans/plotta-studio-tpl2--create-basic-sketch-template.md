---
# plotta-studio-tpl2
title: Create basic sketch template
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-tpl1
---

Create the basic sketch template with minimal boilerplate.

## Template Files

### cargo-generate.toml
```toml
[template]
name = "plotta-sketch-basic"
description = "A basic plotta-studio sketch"
cargo_generate_version = ">=0.18.0"

[placeholders.project-name]
type = "string"
prompt = "Project name?"
regex = "^[a-z][a-z0-9-]*$"

[placeholders.description]
type = "string"
prompt = "Description?"
default = "A generative drawing sketch"

[placeholders.paper-size]
type = "string"
prompt = "Paper size?"
choices = ["a4-landscape", "a4-portrait", "a3-landscape", "a3-portrait"]
default = "a4-landscape"
```

### Cargo.toml.liquid
```toml
[package]
name = "{{project-name}}"
version = "0.1.0"
edition = "2021"
description = "{{description}}"

[dependencies]
sketch-runner = { path = "../../crates/sketch-runner" }
drawing-svg = { path = "../../crates/drawing-svg" }
```

### src/main.rs.liquid
```rust
use sketch_runner::*;

struct MySketch;

impl Sketch for MySketch {
    fn setup(&mut self) -> Drawing {
        {% if paper-size == "a4-landscape" %}
        let mut drawing = Drawing::a4_landscape();
        {% elsif paper-size == "a4-portrait" %}
        let mut drawing = Drawing::a4_portrait();
        {% elsif paper-size == "a3-landscape" %}
        let mut drawing = Drawing::a3_landscape();
        {% elsif paper-size == "a3-portrait" %}
        let mut drawing = Drawing::a3_portrait();
        {% endif %}

        let center = drawing.center();

        // Add your drawing elements here
        drawing.add(Element::circle(center, 50.0));

        drawing
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing) {
        match key {
            Key::Character(c) if c.as_str() == "e" => {
                // Export to SVG
                if let Err(e) = drawing_svg::export_svg(drawing, "{{project-name}}.svg") {
                    log::error!("Failed to export SVG: {e}");
                } else {
                    log::info!("Exported to {{project-name}}.svg");
                }
            }
            _ => {}
        }
    }
}

fn main() {
    run(MySketch);
}
```

## Files to Create
- `templates/sketch-basic/cargo-generate.toml`
- `templates/sketch-basic/Cargo.toml.liquid`
- `templates/sketch-basic/src/main.rs.liquid`
