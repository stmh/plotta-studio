//! Sketch runner for plotta-studio
//!
//! Provides a simple framework for creating generative drawings:
//! - Implement the `Sketch` trait
//! - Call `run()` with your sketch and config
//!
//! Built-in controls:
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom (toward cursor)
//! - Space: Reset view
//! - S: Save to drawing.json
//! - Escape: Quit

#![allow(hidden_glob_reexports)]

use std::sync::Arc;
use std::time::Instant;
use vello::kurbo::{Affine, BezPath, Rect as KurboRect, Stroke as VelloStroke};
use vello::peniko::{Brush, Fill};
use vello::wgpu;
use vello::{AaConfig, RenderParams, Renderer, RendererOptions, Scene};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

// Re-export drawing-core for convenience
pub use drawing_core::*;

// Re-export keyboard types from winit for sketches to use
pub use winit::keyboard::{Key, NamedKey};

// Re-export log crate for sketches to use
pub use log;

// ============================================================================
// Sketch trait
// ============================================================================

/// Implement this trait for your sketch
pub trait Sketch {
    /// Called once at startup, return initial drawing
    fn setup(&mut self) -> Drawing;

    /// Called every frame when animating
    /// Return true if drawing changed and needs re-render
    fn update(&mut self, drawing: &mut Drawing, ctx: &UpdateContext) -> bool {
        let _ = (drawing, ctx);
        false
    }

    /// Optional: handle keyboard input
    fn key_pressed(&mut self, _key: &Key, _drawing: &mut Drawing) {}

    /// Optional: handle mouse press
    fn mouse_pressed(&mut self, _pos: Point, _drawing: &mut Drawing) {}

    /// Optional: handle mouse release
    fn mouse_released(&mut self, _pos: Point, _drawing: &mut Drawing) {}

    /// Optional: handle mouse drag
    fn mouse_dragged(&mut self, _pos: Point, _drawing: &mut Drawing) {}
}

// ============================================================================
// Update context
// ============================================================================

/// Context passed to update() each frame
#[derive(Debug, Clone)]
pub struct UpdateContext {
    /// Time since sketch started (seconds)
    pub time: f64,
    /// Time since last frame (seconds)
    pub delta: f64,
    /// Frame number
    pub frame: u64,
    /// Mouse position in drawing coordinates
    pub mouse: Point,
    /// Whether left mouse button is pressed
    pub mouse_pressed: bool,
}

// ============================================================================
// Runner config
// ============================================================================

/// Configuration for the sketch runner
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Window title
    pub title: String,
    /// Window width in pixels
    pub window_width: u32,
    /// Window height in pixels
    pub window_height: u32,
    /// Whether to animate (call update each frame)
    pub animate: bool,
    /// Background color for the window (outside the drawing)
    pub window_background: Color,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            title: "Plotta Studio".into(),
            window_width: 1200,
            window_height: 800,
            animate: false,
            window_background: Color::gray(40),
        }
    }
}

impl RunnerConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.window_width = width;
        self.window_height = height;
        self
    }

    pub fn with_animation(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }
}

// ============================================================================
// View state (pan/zoom)
// ============================================================================

struct ViewState {
    zoom: f64,
    pan: Point,
    dragging: bool,
    last_mouse: Point,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Point::new(0.0, 0.0),
            dragging: false,
            last_mouse: Point::new(0.0, 0.0),
        }
    }
}

impl ViewState {
    fn reset(&mut self) {
        self.zoom = 1.0;
        self.pan = Point::new(0.0, 0.0);
    }

    fn fit_drawing(&mut self, drawing: &Drawing, window_width: f64, window_height: f64) {
        let margin = 40.0;
        let available_width = window_width - margin * 2.0;
        let available_height = window_height - margin * 2.0;

        let scale_x = available_width / drawing.width;
        let scale_y = available_height / drawing.height;
        self.zoom = scale_x.min(scale_y);

        // Center the drawing
        let scaled_width = drawing.width * self.zoom;
        let scaled_height = drawing.height * self.zoom;
        self.pan.x = (window_width - scaled_width) / 2.0;
        self.pan.y = (window_height - scaled_height) / 2.0;
    }
}

// ============================================================================
// Render state
// ============================================================================

struct RenderState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
    surface_config: wgpu::SurfaceConfiguration,
}

// ============================================================================
// App state
// ============================================================================

struct AppState<S: Sketch> {
    sketch: S,
    config: RunnerConfig,
    drawing: Drawing,
    ctx: UpdateContext,
    view: ViewState,
    start_time: Instant,
    last_frame_time: Instant,
    render_state: Option<RenderState>,
    needs_initial_fit: bool,
    cached_strokes: Vec<Stroke>,
    strokes_dirty: bool,
}

impl<S: Sketch> AppState<S> {
    fn new(mut sketch: S, config: RunnerConfig) -> Self {
        let drawing = sketch.setup();
        let now = Instant::now();

        Self {
            sketch,
            config,
            drawing,
            ctx: UpdateContext {
                time: 0.0,
                delta: 0.0,
                frame: 0,
                mouse: Point::ZERO,
                mouse_pressed: false,
            },
            view: ViewState::default(),
            start_time: now,
            last_frame_time: now,
            render_state: None,
            needs_initial_fit: true,
            cached_strokes: Vec::new(),
            strokes_dirty: true,
        }
    }

    fn refresh_strokes(&mut self) {
        if self.strokes_dirty {
            self.cached_strokes = self.drawing.flatten();
            self.strokes_dirty = false;
        }
    }

    fn render(&mut self, state: &mut RenderState) {
        self.refresh_strokes();

        let mut scene = Scene::new();

        let width = state.surface_config.width as f64;
        let height = state.surface_config.height as f64;

        // Window background
        let window_bg = color_to_vello(self.config.window_background);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            window_bg,
            None,
            &KurboRect::new(0.0, 0.0, width, height),
        );

        // Build transform: pan then zoom
        let transform =
            Affine::translate((self.view.pan.x, self.view.pan.y)) * Affine::scale(self.view.zoom);

        // Drawing background (paper)
        let drawing_bg = color_to_vello(self.drawing.background);
        scene.fill(
            Fill::NonZero,
            transform,
            drawing_bg,
            None,
            &KurboRect::new(0.0, 0.0, self.drawing.width, self.drawing.height),
        );

        // Draw all strokes
        for stroke in &self.cached_strokes {
            if stroke.points.len() < 2 {
                continue;
            }

            let mut path = BezPath::new();
            path.move_to((stroke.points[0].x, stroke.points[0].y));

            for pt in &stroke.points[1..] {
                path.line_to((pt.x, pt.y));
            }

            if stroke.closed {
                path.close_path();
            }

            let brush = Brush::Solid(color_to_vello(stroke.style.stroke_color));
            let style = VelloStroke::new(stroke.style.stroke_width)
                .with_caps(vello::kurbo::Cap::Round)
                .with_join(vello::kurbo::Join::Round);

            scene.stroke(&style, transform, &brush, None, &path);
        }

        // Render
        let surface_texture = state.surface.get_current_texture().unwrap();
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        state
            .renderer
            .render_to_texture(
                &state.device,
                &state.queue,
                &scene,
                &texture_view,
                &RenderParams {
                    base_color: vello::peniko::Color::WHITE,
                    width: state.surface_config.width,
                    height: state.surface_config.height,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .unwrap();

        surface_texture.present();
    }
}

impl<S: Sketch> ApplicationHandler for AppState<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render_state.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(&self.config.title)
                        .with_inner_size(LogicalSize::new(
                            self.config.window_width,
                            self.config.window_height,
                        )),
                )
                .unwrap(),
        );

        let instance = wgpu::Instance::default();

        // Safety: window lives as long as surface due to Arc
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("Failed to find adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        ))
        .expect("Failed to create device");

        let size = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let renderer = Renderer::new(
            &device,
            RendererOptions {
                surface_format: Some(surface_config.format),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: None,
            },
        )
        .expect("Failed to create renderer");

        self.render_state = Some(RenderState {
            window,
            surface,
            device,
            queue,
            renderer,
            surface_config,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.render_state.is_none() {
            return;
        }

        // Track if we need to request a redraw at the end
        let mut needs_redraw = false;

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(state) = &mut self.render_state {
                        state.surface_config.width = size.width;
                        state.surface_config.height = size.height;
                        state.surface.configure(&state.device, &state.surface_config);
                    }

                    if self.needs_initial_fit {
                        self.view
                            .fit_drawing(&self.drawing, size.width as f64, size.height as f64);
                        self.needs_initial_fit = false;
                    }

                    needs_redraw = true;
                }
            }

            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                if button == MouseButton::Middle {
                    self.view.dragging = btn_state == ElementState::Pressed;
                }
                if button == MouseButton::Left {
                    let was_pressed = self.ctx.mouse_pressed;
                    self.ctx.mouse_pressed = btn_state == ElementState::Pressed;

                    if self.ctx.mouse_pressed && !was_pressed {
                        self.sketch.mouse_pressed(self.ctx.mouse, &mut self.drawing);
                        self.strokes_dirty = true;
                        needs_redraw = true;
                    } else if !self.ctx.mouse_pressed && was_pressed {
                        self.sketch
                            .mouse_released(self.ctx.mouse, &mut self.drawing);
                        self.strokes_dirty = true;
                        needs_redraw = true;
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let mouse = Point::new(position.x, position.y);

                if self.view.dragging {
                    self.view.pan.x += mouse.x - self.view.last_mouse.x;
                    self.view.pan.y += mouse.y - self.view.last_mouse.y;
                    needs_redraw = true;
                }

                self.view.last_mouse = mouse;
                // Inline screen_to_drawing calculation to avoid borrow conflict
                self.ctx.mouse = Point::new(
                    (mouse.x - self.view.pan.x) / self.view.zoom,
                    (mouse.y - self.view.pan.y) / self.view.zoom,
                );

                if self.ctx.mouse_pressed {
                    self.sketch.mouse_dragged(self.ctx.mouse, &mut self.drawing);
                    self.strokes_dirty = true;
                    needs_redraw = true;
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64 * 0.1,
                    MouseScrollDelta::PixelDelta(p) => p.y * 0.001,
                };

                // Zoom toward mouse position
                let old_zoom = self.view.zoom;
                self.view.zoom = (self.view.zoom * (1.0 + scroll)).clamp(0.01, 100.0);

                let factor = self.view.zoom / old_zoom;
                self.view.pan.x =
                    self.view.last_mouse.x - (self.view.last_mouse.x - self.view.pan.x) * factor;
                self.view.pan.y =
                    self.view.last_mouse.y - (self.view.last_mouse.y - self.view.pan.y) * factor;

                needs_redraw = true;
            }

            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match &event.logical_key {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),

                    Key::Named(NamedKey::Space) => {
                        // Fit drawing to window
                        if let Some(state) = &self.render_state {
                            let width = state.surface_config.width as f64;
                            let height = state.surface_config.height as f64;
                            self.view.fit_drawing(&self.drawing, width, height);
                        }
                        needs_redraw = true;
                    }

                    Key::Character(c) if c.as_str() == "s" => {
                        // Save drawing
                        let path = std::path::Path::new("drawing.json");
                        match self.drawing.save(path) {
                            Ok(_) => log::info!("Saved to {}", path.display()),
                            Err(e) => log::error!("Failed to save: {e}"),
                        }
                    }

                    Key::Character(c) if c.as_str() == "r" => {
                        // Reset view
                        self.view.reset();
                        needs_redraw = true;
                    }

                    key => {
                        self.sketch.key_pressed(key, &mut self.drawing);
                        self.strokes_dirty = true;
                        needs_redraw = true;
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                // Take ownership of render_state temporarily to avoid borrow conflict
                if let Some(mut state) = self.render_state.take() {
                    self.render(&mut state);
                    if self.config.animate {
                        state.window.request_redraw();
                    }
                    self.render_state = Some(state);
                }
            }

            _ => {}
        }

        // Request redraw if needed
        if needs_redraw {
            if let Some(state) = &self.render_state {
                state.window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.config.animate && self.render_state.is_some() {
            let now = Instant::now();
            self.ctx.delta = now.duration_since(self.last_frame_time).as_secs_f64();
            self.ctx.time = now.duration_since(self.start_time).as_secs_f64();
            self.ctx.frame += 1;
            self.last_frame_time = now;

            if self.sketch.update(&mut self.drawing, &self.ctx) {
                self.strokes_dirty = true;
                if let Some(state) = &self.render_state {
                    state.window.request_redraw();
                }
            }
        }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

fn color_to_vello(c: Color) -> vello::peniko::Color {
    vello::peniko::Color::new([
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    ])
}

// ============================================================================
// Public API
// ============================================================================

/// Run a sketch with default configuration
pub fn run<S: Sketch>(sketch: S) {
    run_with_config(sketch, RunnerConfig::default());
}

/// Run a sketch with custom configuration
pub fn run_with_config<S: Sketch>(sketch: S, config: RunnerConfig) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = AppState::new(sketch, config);

    event_loop.run_app(&mut app).expect("Event loop failed");
}
