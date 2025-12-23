---
# plotta-studio-gui2
title: Integrate egui with wgpu renderer
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-gui1
---

Set up egui to render alongside Vello using the shared wgpu context.

## Implementation

### Add Dependencies
```toml
# crates/sketch-runner/Cargo.toml
egui = "0.29"
egui-wgpu = "0.29"
egui-winit = "0.29"
```

### Modify RenderState
```rust
struct RenderState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
    surface_config: wgpu::SurfaceConfiguration,
    // Add egui
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
}
```

### Initialize egui in resumed()
```rust
fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    // ... existing wgpu setup ...

    // Initialize egui
    let egui_ctx = egui::Context::default();
    let egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );
    let egui_renderer = egui_wgpu::Renderer::new(
        &device,
        surface_format,
        None,
        1,
        false,
    );

    // ... store in RenderState ...
}
```

### Handle egui Events
```rust
fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    // Let egui process events first
    if let Some(state) = &mut self.render_state {
        let response = state.egui_state.on_window_event(&state.window, &event);
        if response.consumed {
            return; // egui consumed the event
        }
    }

    // ... existing event handling ...
}
```

### Render egui After Vello
```rust
fn render(&mut self, state: &mut RenderState) {
    // ... existing Vello render ...

    // Render egui
    let raw_input = state.egui_state.take_egui_input(&state.window);
    let full_output = state.egui_ctx.run(raw_input, |ctx| {
        egui::SidePanel::right("params").show(ctx, |ui| {
            self.sketch.gui(ui, &mut self.drawing);
        });
    });

    state.egui_state.handle_platform_output(&state.window, full_output.platform_output);

    let paint_jobs = state.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

    // ... encode and render egui ...
}
```

## Files to Modify
- `crates/sketch-runner/Cargo.toml`
- `crates/sketch-runner/src/lib.rs`
