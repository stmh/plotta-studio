---
# plotta-studio-gui1
title: GUI for parameters (egui)
status: todo
type: epic
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
---

Add an egui-based GUI panel for interactive parameter adjustment in sketches, allowing real-time tweaking of values without recompiling.

## Investigation

### Current State
- `sketch-runner` uses Vello for rendering with winit for windowing
- No GUI currently exists - parameters require code changes
- Sketch trait has `setup()`, `update()`, and event handlers

### Why egui?
- Immediate mode GUI - simple, no complex state management
- `egui-wgpu` backend works with our existing wgpu setup
- Popular in creative coding (nannou, bevy)
- Minimal dependencies

### Integration Options

#### Option A: egui-wgpu (Recommended)
Render egui directly with wgpu alongside Vello:
- Share the wgpu device/queue
- Render egui after Vello scene
- Full control over integration

#### Option B: egui-winit-wgpu
Higher-level integration:
- Handles more boilerplate
- May conflict with our custom event handling

### UI Features to Support

1. **Parameter Sliders**
   - Float sliders with range
   - Integer sliders
   - 2D point pickers

2. **Color Pickers**
   - RGB/HSL color selection
   - Alpha support

3. **Toggles and Buttons**
   - Boolean toggles
   - Action buttons (regenerate, randomize)

4. **Presets**
   - Save/load parameter presets
   - JSON export

### API Design

```rust
/// Trait for sketches with GUI parameters
pub trait SketchWithGui: Sketch {
    /// Define GUI controls
    fn gui(&mut self, ui: &mut egui::Ui, drawing: &mut Drawing) -> bool;
}

/// Helper macro for common patterns
macro_rules! param_slider {
    ($ui:expr, $label:expr, $value:expr, $range:expr) => {
        $ui.add(egui::Slider::new($value, $range).text($label))
    };
}

// Example usage in sketch:
impl SketchWithGui for MySketch {
    fn gui(&mut self, ui: &mut egui::Ui, drawing: &mut Drawing) -> bool {
        let mut changed = false;

        ui.heading("Parameters");

        changed |= ui.add(egui::Slider::new(&mut self.count, 1..=100).text("Count")).changed();
        changed |= ui.add(egui::Slider::new(&mut self.radius, 1.0..=100.0).text("Radius")).changed();

        if ui.button("Randomize").clicked() {
            self.seed = rand::random();
            changed = true;
        }

        changed
    }
}
```

### Dependencies to Add
```toml
egui = "0.29"
egui-wgpu = "0.29"
egui-winit = "0.29"
```
