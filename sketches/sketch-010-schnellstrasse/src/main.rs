//! Schnellstrasse SVG Sketch - A4 landscape with frame and imported SVG
//!
//! Controls:
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom
//! - Space: Fit drawing to window
//! - R: Reset view
//! - E: Export to SVG (built-in)
//! - P: Plot to AxiDraw (built-in, requires `hardware` feature)
//! - T: Toggle SVG rendering on/off
//! - Escape: Quit

use drawing_svg::import_svg_string;
use drawing_utils::{draw_frame_with_title, FrameOptions};
use sketch_runner::*;

const SCHNELLSTRASSE_SVG: &str = include_str!("../assets/schnellstrasse.svg");

struct SchnellstrasseSketch {
    show_svg: bool,
}

impl SchnellstrasseSketch {
    fn new() -> Self {
        Self {
            show_svg: true, // Start with SVG enabled - press T to toggle
        }
    }

    fn build_drawing(&self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::a4_landscape().with_background(Color::WHITE);

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
                "Schnellstrasse, 2025",
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

        // Import the SVG (only if enabled)
        if self.show_svg {
            log::info!("Starting SVG import...");
            let import_start = std::time::Instant::now();

            match import_svg_string(SCHNELLSTRASSE_SVG) {
                Ok(imported) => {
                    log::info!("SVG import took {:?}", import_start.elapsed());
                    let svg_drawing = &imported.drawing;
                    log::info!("SVG has {} elements", svg_drawing.elements.len());

                    // Clip rect: 1.5cm from left/top/right, 2cm from bottom
                    let clip_left = 15.0;
                    let clip_top = 15.0;
                    let clip_right = 15.0;
                    let clip_bottom = 20.0;
                    let clip_width = drawing.width - clip_left - clip_right;
                    let clip_height = drawing.height - clip_top - clip_bottom;

                    // Calculate scaling to fit the SVG within the clip area
                    let scale_x = clip_width / svg_drawing.width;
                    let scale_y = clip_height / svg_drawing.height;
                    let scale = scale_x.min(scale_y);

                    // Center the SVG in the clip area
                    let scaled_width = svg_drawing.width * scale;
                    let scaled_height = svg_drawing.height * scale;
                    let offset_x = clip_left + (clip_width - scaled_width) / 2.0;
                    let offset_y = clip_top + (clip_height - scaled_height) / 2.0;

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

                    // Create clip rect and add SVG content clipped to it
                    let clip_rect = Element::rect(clip_left, clip_top, clip_width, clip_height);
                    let svg_content = Element::group(content_group)
                        .translate(-svg_center_x, -svg_center_y)
                        .scale_uniform(scale)
                        .translate(final_center_x, final_center_y);

                    log::info!("Adding clipped SVG to drawing...");
                    drawing.add(Element::clip(clip_rect).add(svg_content));
                    log::info!("Clipped SVG added to drawing");

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
                        "Imported schnellstrasse.svg: {} elements, scaled to {:.1}% (stroke widths NOT scaled)",
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
                    log::error!("Failed to import schnellstrasse.svg: {}", e);
                }
            }
        } else {
            log::info!("SVG rendering disabled (press T to enable)");
        }

        drawing
    }
}

impl Sketch for SchnellstrasseSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        self.build_drawing(ctx)
    }

    fn update(&mut self, _drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, ctx: &SketchContext) -> bool {
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
                true
            }
            _ => false,
        }
    }
}

fn main() {
    let sketch = SchnellstrasseSketch::new();

    run_with_config(
        sketch,
        RunnerConfig::new("Schnellstrasse, 2025")
            .with_size(1100, 800) // Landscape aspect ratio
            .with_animation(false),
    );
}
