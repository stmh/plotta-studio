//! Sketch demonstrating ClipGroup with hatched circles
//!
//! Three overlapping circles arranged in a triangle, each with
//! parallel hatch lines rotated at different angles (0, 120, 240 degrees).
//!
//! Controls:
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom
//! - Space: Fit drawing to window
//! - R: Reset view
//! - E: Export to drawing.svg
//! - G: Regenerate drawing
//! - Up/Down: Adjust hatch density
//! - Left/Right: Adjust circle radius
//! - Escape: Quit

use drawing_utils::{draw_frame_with_title, generate_hatch_lines, FrameOptions, HatchOptions};
use sketch_runner::*;
use std::f64::consts::PI;

struct HatchedCirclesSketch {
    hatch_spacing: f64,
    circle_radius: f64,
}

impl Default for HatchedCirclesSketch {
    fn default() -> Self {
        Self {
            hatch_spacing: 2.0,
            circle_radius: 50.0,
        }
    }
}

impl Sketch for HatchedCirclesSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::a4_landscape();
        self.generate(&mut drawing, ctx);
        drawing
    }

    fn update(&mut self, _drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, ctx: &SketchContext) {
        match key {
            Key::Character(c) if c.as_str() == "g" => {
                self.generate(drawing, ctx);
            }
            Key::Character(c) if c.as_str() == "e" => {
                if let Err(e) = drawing_svg::export_svg(drawing, "hatched-circles.svg", ctx.render)
                {
                    log::error!("Failed to export SVG: {e}");
                } else {
                    log::info!("Exported to hatched-circles.svg");
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.hatch_spacing = (self.hatch_spacing - 0.5).max(1.0);
                self.generate(drawing, ctx);
                log::info!("Hatch spacing: {:.1}", self.hatch_spacing);
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.hatch_spacing += 0.5;
                self.generate(drawing, ctx);
                log::info!("Hatch spacing: {:.1}", self.hatch_spacing);
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.circle_radius += 5.0;
                self.generate(drawing, ctx);
                log::info!("Circle radius: {:.1}", self.circle_radius);
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.circle_radius = (self.circle_radius - 5.0).max(20.0);
                self.generate(drawing, ctx);
                log::info!("Circle radius: {:.1}", self.circle_radius);
            }
            _ => {}
        }
    }
}

impl HatchedCirclesSketch {
    fn generate(&self, drawing: &mut Drawing, ctx: &SketchContext) {
        drawing.clear();

        let center = drawing.center();
        let radius = self.circle_radius;

        // Triangle arrangement: circles offset from center
        // Distance from center to each circle center
        let triangle_radius = radius * 0.8;

        // Three circle centers in a triangle (pointing up)
        let angles = [
            -PI / 2.0,                  // Top
            -PI / 2.0 + 2.0 * PI / 3.0, // Bottom left
            -PI / 2.0 + 4.0 * PI / 3.0, // Bottom right
        ];

        let circle_centers: Vec<Point> = angles
            .iter()
            .map(|&angle| {
                Point::new(
                    center.x + angle.cos() * triangle_radius,
                    center.y + angle.sin() * triangle_radius,
                )
            })
            .collect();

        // Hatch rotation angles (0, 120, 240 degrees)
        let hatch_rotations = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];

        // Create hatched circles
        for (i, (&circle_center, &hatch_angle)) in circle_centers
            .iter()
            .zip(hatch_rotations.iter())
            .enumerate()
        {
            // Generate hatch lines using drawing-utils
            let hatch_options = HatchOptions::new()
                .spacing(self.hatch_spacing)
                .angle(hatch_angle);

            let hatch_lines = generate_hatch_lines(circle_center, radius, &hatch_options);

            // Create clip group: circle clips the hatch lines
            let hatched_circle =
                Element::clip(Element::circle(circle_center, radius)).add(hatch_lines);

            drawing.add(hatched_circle);

            log::debug!(
                "Circle {}: center=({:.1}, {:.1}), hatch_angle={:.0}°",
                i + 1,
                circle_center.x,
                circle_center.y,
                hatch_angle.to_degrees()
            );
        }

        // Add frame with title and signature using default font
        let frame_options = FrameOptions::with_default_font(ctx.fonts)
            .expect("Default font not loaded")
            .with_signature();
        drawing.add(draw_frame_with_title(
            drawing,
            "Hatched Circles",
            &frame_options,
        ));

        log::info!(
            "Generated {} elements, {} strokes",
            drawing.elements.len(),
            drawing.stroke_count(ctx.render)
        );
    }
}

fn main() {
    let sketch = HatchedCirclesSketch::default();

    run_with_config(
        sketch,
        RunnerConfig::new("Hatched Circles - ClipGroup Demo")
            .with_size(1400, 900)
            .with_animation(false),
    );
}
