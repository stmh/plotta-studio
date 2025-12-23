---
# plotta-studio-gui3
title: Create parameter widgets
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-gui1
---

Create convenient parameter widgets and helpers for common sketch parameters.

## Widget Types

### Point2D Widget
```rust
pub fn point2d_widget(ui: &mut egui::Ui, label: &str, point: &mut Point, range: f64) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        changed |= ui.add(egui::DragValue::new(&mut point.x).speed(0.5).range(-range..=range)).changed();
        changed |= ui.add(egui::DragValue::new(&mut point.y).speed(0.5).range(-range..=range)).changed();
    });
    changed
}
```

### Color Widget
```rust
pub fn color_widget(ui: &mut egui::Ui, label: &str, color: &mut Color) -> bool {
    let mut rgba = [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0,
    ];

    let changed = ui.horizontal(|ui| {
        ui.label(label);
        ui.color_edit_button_rgba_unmultiplied(&mut rgba).changed()
    }).inner;

    if changed {
        *color = Color::rgba(
            (rgba[0] * 255.0) as u8,
            (rgba[1] * 255.0) as u8,
            (rgba[2] * 255.0) as u8,
            (rgba[3] * 255.0) as u8,
        );
    }

    changed
}
```

### Range Slider
```rust
pub fn range_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    min: f64,
    max: f64,
) -> bool {
    ui.add(egui::Slider::new(value, min..=max).text(label)).changed()
}
```

### Integer Slider
```rust
pub fn int_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut i32,
    min: i32,
    max: i32,
) -> bool {
    ui.add(egui::Slider::new(value, min..=max).text(label)).changed()
}
```

### Seed/Randomize
```rust
pub fn seed_widget(ui: &mut egui::Ui, seed: &mut u64) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Seed");
        changed |= ui.add(egui::DragValue::new(seed)).changed();
        if ui.button("🎲").clicked() {
            *seed = rand::random();
            changed = true;
        }
    });
    changed
}
```

## Files to Create/Modify
- `crates/sketch-runner/src/gui.rs` (new module)
- `crates/sketch-runner/src/lib.rs` (add module)
