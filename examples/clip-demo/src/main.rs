//! Sketch demonstrating inverted clip functionality
//!
//! A square filled with hatch lines, with three overlapping circles
//! where the hatches are clipped out using the inverted clip feature.
//!
//! This demonstrates:
//! - Normal clipping (hatches clipped to square boundary)
//! - Inverted clipping (hatches removed where circles overlap)
//!
//! Controls:
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom
//! - Space: Fit drawing to window
//! - R: Reset view
//! - E: Export to SVG (built-in)
//! - P: Plot to AxiDraw (built-in, requires `hardware` feature)
//! - G: Regenerate drawing
//! - Up/Down: Adjust hatch density
//! - Left/Right: Adjust circle radius
//! - Escape: Quit

use drawing_utils::{
    draw_frame_with_title, generate_hatch_lines_rect, FrameOptions, HatchOptions,
    PlaceholderSignature,
};
use sketch_runner::*;
use std::f64::consts::PI;

struct ClipDemoSketch {
    hatch_spacing: f64,
    circle_radius: f64,
    position_jitter: f64,
    endpoint_jitter: f64,
    angle_jitter_deg: f64,
}

impl Default for ClipDemoSketch {
    fn default() -> Self {
        Self {
            hatch_spacing: 2.5,
            circle_radius: 28.0,
            position_jitter: 0.2,
            endpoint_jitter: 1.0,
            angle_jitter_deg: 1.0,
        }
    }
}

impl Sketch for ClipDemoSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::a4_portrait();
        self.generate(&mut drawing, ctx);
        drawing
    }

    fn update(&mut self, _drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, ctx: &SketchContext) -> bool {
        match key {
            Key::Character(c) if c.as_str() == "g" => {
                self.generate(drawing, ctx);
                true
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.hatch_spacing = (self.hatch_spacing - 0.5).max(1.0);
                self.generate(drawing, ctx);
                log::info!("Hatch spacing: {:.1}", self.hatch_spacing);
                true
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.hatch_spacing += 0.5;
                self.generate(drawing, ctx);
                log::info!("Hatch spacing: {:.1}", self.hatch_spacing);
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.circle_radius += 5.0;
                self.generate(drawing, ctx);
                log::info!("Circle radius: {:.1}", self.circle_radius);
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.circle_radius = (self.circle_radius - 5.0).max(10.0);
                self.generate(drawing, ctx);
                log::info!("Circle radius: {:.1}", self.circle_radius);
                true
            }
            _ => false,
        }
    }
}

impl ClipDemoSketch {
    fn generate(&self, drawing: &mut Drawing, ctx: &SketchContext) {
        drawing.clear();

        let center = drawing.center();

        // Square dimensions - make it fit nicely in A4 portrait
        let square_size = 140.0;
        let square_x = center.x - square_size / 2.0;
        let square_y = center.y - square_size / 2.0 - 20.0; // Shift up a bit for title

        // Three overlapping circles arranged in a triangle pattern
        let circle_radius = self.circle_radius;
        let triangle_radius = square_size * 0.18; // Distance from center to each circle center

        // Circle centers in a triangle (pointing up)
        let angles = [
            -PI / 2.0,                  // Top
            -PI / 2.0 + 2.0 * PI / 3.0, // Bottom left
            -PI / 2.0 + 4.0 * PI / 3.0, // Bottom right
        ];

        let square_center = Point::new(square_x + square_size / 2.0, square_y + square_size / 2.0);

        let circle_centers: Vec<Point> = angles
            .iter()
            .map(|&angle| {
                Point::new(
                    square_center.x + angle.cos() * triangle_radius,
                    square_center.y + angle.sin() * triangle_radius,
                )
            })
            .collect();

        // Generate hatch lines covering the square area with randomness
        let hatch_options = HatchOptions::new()
            .spacing(self.hatch_spacing)
            .angle_deg(45.0)
            .position_jitter(self.position_jitter)
            .endpoint_jitter(self.endpoint_jitter)
            .angle_jitter_deg(self.angle_jitter_deg);

        let hatch_lines =
            generate_hatch_lines_rect(square_x, square_y, square_size, square_size, &hatch_options);

        // Create the three circles as a group for the inverted clip
        let circles_group = Element::group(
            Group::new()
                .add(Element::circle(circle_centers[0], circle_radius))
                .add(Element::circle(circle_centers[1], circle_radius))
                .add(Element::circle(circle_centers[2], circle_radius)),
        );

        // First, clip the hatches to the square (normal clip - keep inside)
        // Then, clip out the circles using inverted clip (keep outside circles)
        let hatches_clipped_to_square =
            Element::clip(Element::rect(square_x, square_y, square_size, square_size))
                .add(hatch_lines);

        // Now apply inverted clip to remove the circles from the hatched area
        let final_hatches = Element::clip(circles_group)
            .invert(true)
            .add(hatches_clipped_to_square);

        drawing.add(final_hatches);

        // Add frame with title and signature
        let frame_options = FrameOptions::with_default_font(ctx.fonts)
            .expect("Default font not loaded")
            .with_signature(PlaceholderSignature);
        drawing.add(draw_frame_with_title(
            drawing,
            "Three Moons Rising",
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
    let sketch = ClipDemoSketch::default();

    run_with_config(
        sketch,
        RunnerConfig::new("Three Moons Rising")
            .with_size(900, 1200)
            .with_animation(false),
    );
}
