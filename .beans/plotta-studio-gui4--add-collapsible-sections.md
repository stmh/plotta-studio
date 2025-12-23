---
# plotta-studio-gui4
title: Add collapsible sections and presets
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-gui1
---

Add UI organization features like collapsible sections and parameter presets.

## Collapsible Sections

```rust
pub fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui) -> bool) -> bool {
    egui::CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, add_contents)
        .body_returned
        .unwrap_or(false)
}

// Usage:
fn gui(&mut self, ui: &mut egui::Ui, drawing: &mut Drawing) -> bool {
    let mut changed = false;

    changed |= section(ui, "Shape", |ui| {
        let mut c = false;
        c |= int_slider(ui, "Sides", &mut self.sides, 3, 12);
        c |= range_slider(ui, "Radius", &mut self.radius, 10.0, 200.0);
        c
    });

    changed |= section(ui, "Style", |ui| {
        let mut c = false;
        c |= color_widget(ui, "Stroke", &mut self.stroke_color);
        c |= range_slider(ui, "Width", &mut self.stroke_width, 0.5, 10.0);
        c
    });

    changed
}
```

## Parameter Presets

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Preset<T> {
    pub name: String,
    pub params: T,
}

pub fn preset_selector<T: Serialize + for<'de> Deserialize<'de> + Clone>(
    ui: &mut egui::Ui,
    presets: &mut Vec<Preset<T>>,
    current: &mut T,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label("Presets");

        egui::ComboBox::from_id_salt("preset_select")
            .selected_text("Select...")
            .show_ui(ui, |ui| {
                for preset in presets.iter() {
                    if ui.selectable_label(false, &preset.name).clicked() {
                        *current = preset.params.clone();
                        changed = true;
                    }
                }
            });

        if ui.button("Save").clicked() {
            // Show save dialog
        }
    });

    changed
}
```

## Export/Import Parameters

```rust
impl<T: Serialize> Preset<T> {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub fn export_button<T: Serialize>(ui: &mut egui::Ui, name: &str, params: &T) {
    if ui.button("Export JSON").clicked() {
        let preset = Preset {
            name: name.to_string(),
            params,
        };
        if let Ok(json) = preset.to_json() {
            // Copy to clipboard or save file
            ui.output_mut(|o| o.copied_text = json);
        }
    }
}
```

## Files to Modify
- `crates/sketch-runner/src/gui.rs`
