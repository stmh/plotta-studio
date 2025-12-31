//! Altoetting SVG Sketch - A4 portrait with frame and imported SVG
//!
//! Controls:
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom
//! - Space: Fit drawing to window
//! - R: Reset view
//! - E: Export to drawing.svg
//! - P: Plot to AxiDraw (requires `hardware` feature)
//! - T: Toggle SVG rendering on/off
//! - Escape: Quit

#[cfg(feature = "hardware")]
use drawing_plotter::{plot_in_background, PlotConfig, PlotEvent, PlotHandle};
use drawing_svg::import_svg_string;
use drawing_utils::{draw_frame_with_title, FrameOptions};
use sketch_runner::*;

const ALTOETTING_SVG: &str = include_str!("../assets/altoetting.svg");

struct AltoettingSketch {
    show_svg: bool,
    #[cfg(feature = "hardware")]
    plot_handle: Option<PlotHandle>,
}

impl AltoettingSketch {
    fn new() -> Self {
        Self {
            show_svg: true, // Start with SVG enabled - press T to toggle
            #[cfg(feature = "hardware")]
            plot_handle: None,
        }
    }

    fn build_drawing(&self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::a4_portrait().with_background(Color::WHITE);

        // Add frame with title and signature using default font
        if let Some(frame_options) = FrameOptions::with_default_font(ctx.fonts) {
            let frame_options = frame_options
                .margin(5.0)
                .margin_bottom(10.0)
                .font_size(4.0)
                .stroke_width(0.35)
                .with_signature()
                .signature_height(4.0);

            drawing.add(draw_frame_with_title(
                &drawing,
                "Altoetting, 2026",
                &frame_options,
            ));
        } else {
            log::error!("Default font not loaded! Adding frame without title.");
        }

        log::debug!(
            "Drawing has {} elements, size {}x{}",
            drawing.elements.len(),
            drawing.width,
            drawing.height
        );

        // Import the Hamburg SVG (only if enabled)
        if self.show_svg {
            log::info!("Starting SVG import...");
            let import_start = std::time::Instant::now();

            match import_svg_string(ALTOETTING_SVG) {
                Ok(imported) => {
                    log::info!("SVG import took {:?}", import_start.elapsed());
                    let svg_drawing = &imported.drawing;
                    log::info!("SVG has {} elements", svg_drawing.elements.len());

                    // Calculate area inside the frame
                    let margin = 10.0;
                    let padding = 5.0; // Extra padding inside frame
                    let available_x = margin + padding;
                    let available_y = margin + padding;
                    let available_width = drawing.width - margin * 2.0 - padding * 2.0;
                    let available_height = drawing.height - margin - 15.0 - padding * 2.0; // Account for bottom margin

                    // Calculate scaling to fit the SVG
                    let scale_x = available_width / svg_drawing.width;
                    let scale_y = available_height / svg_drawing.height;
                    let scale = scale_x.min(scale_y);

                    // Center the SVG in the available area
                    let scaled_width = svg_drawing.width * scale;
                    let scaled_height = svg_drawing.height * scale;
                    let offset_x = available_x + (available_width - scaled_width) / 2.0;
                    let offset_y = available_y + (available_height - scaled_height) / 2.0;

                    // Build transform: translate to position and scale
                    let svg_center_x = svg_drawing.width / 2.0;
                    let svg_center_y = svg_drawing.height / 2.0;
                    let final_center_x = offset_x + scaled_width / 2.0;
                    let final_center_y = offset_y + scaled_height / 2.0;

                    log::info!("Building content group...");
                    let group_start = std::time::Instant::now();
                    let mut content_group = Group::new();
                    for element in &svg_drawing.elements {
                        content_group.push(element.clone());
                    }
                    log::info!("Content group built in {:?}", group_start.elapsed());

                    log::info!("Adding transformed group to drawing...");
                    drawing.add(
                        Element::group(content_group)
                            .translate(-svg_center_x, -svg_center_y)
                            .scale_uniform(scale)
                            .translate(final_center_x, final_center_y),
                    );
                    log::info!("Group added to drawing");

                    // Log any import warnings (limit to first 5)
                    if !imported.warnings.is_empty() {
                        log::warn!(
                            "{} import warnings (showing first 5):",
                            imported.warnings.len()
                        );
                        for warning in imported.warnings.iter().take(5) {
                            log::warn!("  {:?}", warning);
                        }
                    }

                    log::info!(
                        "Imported altoetting.svg: {} elements, scaled to {:.1}% (stroke widths NOT scaled)",
                        svg_drawing.elements.len(),
                        scale * 100.0
                    );
                    log::info!(
                        "SVG original size: {:.1}x{:.1}, needs stroke width scaling by {:.3}",
                        svg_drawing.width,
                        svg_drawing.height,
                        scale
                    );
                }
                Err(e) => {
                    log::error!("Failed to import altoetting.svg: {}", e);
                }
            }
        } else {
            log::info!("SVG rendering disabled (press T to enable)");
        }

        drawing
    }
}

impl Sketch for AltoettingSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        self.build_drawing(ctx)
    }

    fn update(&mut self, _drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        #[cfg(feature = "hardware")]
        {
            if let Some(ref handle) = self.plot_handle {
                for event in handle.drain_events() {
                    match event {
                        PlotEvent::Started { total_strokes } => {
                            log::info!("Plotting started: {} strokes", total_strokes);
                        }
                        PlotEvent::StrokeComplete { index, total } => {
                            log::info!("Stroke {}/{} complete", index + 1, total);
                        }
                        PlotEvent::Completed => {
                            log::info!("Plotting completed!");
                        }
                        PlotEvent::Error(e) => {
                            log::error!("Plotting error: {}", e);
                        }
                        _ => {}
                    }
                }

                if !handle.is_running() {
                    self.plot_handle = None;
                }
            }
        }
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, ctx: &SketchContext) {
        match key {
            Key::Character(c) if c.as_str() == "t" => {
                self.show_svg = !self.show_svg;
                log::info!(
                    "SVG rendering {}",
                    if self.show_svg { "enabled" } else { "disabled" }
                );
                log::info!("Rebuilding drawing...");
                let start = std::time::Instant::now();
                *drawing = self.build_drawing(ctx);
                log::info!("Drawing rebuilt in {:?}", start.elapsed());
                log::info!("Drawing has {} elements total", drawing.elements.len());
            }
            Key::Character(c) if c.as_str() == "e" => {
                if let Err(e) = drawing_svg::export_svg(drawing, "drawing.svg", ctx.render) {
                    log::error!("Failed to export SVG: {e}");
                } else {
                    log::info!("Exported to drawing.svg");
                }
            }
            #[cfg(feature = "hardware")]
            Key::Character(c) if c.as_str() == "p" => {
                if self.plot_handle.is_some() {
                    log::warn!("Plotting already in progress");
                } else {
                    log::info!("Starting plot...");
                    let plot_ctx = RenderContext::new(ctx.fonts.registry().clone());
                    match plot_in_background(drawing.clone(), PlotConfig::default(), plot_ctx, None)
                    {
                        Ok(handle) => {
                            self.plot_handle = Some(handle);
                            log::info!("Plot started in background thread");
                        }
                        Err(e) => {
                            log::error!("Failed to start plot: {e}");
                        }
                    }
                }
            }
            #[cfg(not(feature = "hardware"))]
            Key::Character(c) if c.as_str() == "p" => {
                log::warn!("Plotting requires the 'hardware' feature. Run with: cargo run --features hardware");
            }
            _ => {}
        }
    }
}

fn main() {
    let sketch = AltoettingSketch::new();

    run_with_config(
        sketch,
        RunnerConfig::new("Altoetting, 2026")
            .with_size(800, 1100) // Portrait aspect ratio
            .with_animation(false),
    );
}
