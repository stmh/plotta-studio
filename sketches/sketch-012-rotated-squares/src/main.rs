//! Rotated Squares Sketch
//!
//! A composition of 15-20 slightly rotated squares, all sharing the same center point.
//! Each square is rotated by a random angle, creating a layered, dynamic pattern.
//! Hidden lines are removed by clipping each square against all squares on top of it.
//!
//! Controls:
//! - G: Regenerate squares with new random rotations
//! - Up/Down: Adjust number of squares
//! - Left/Right: Adjust maximum rotation angle
//! - E: Export to SVG (built-in)
//! - P: Plot to AxiDraw (built-in, requires `hardware` feature)
//! - Space: Fit drawing to window
//! - Escape: Quit

use drawing_utils::{draw_frame_with_title, FrameOptions};
use rand::{Rng, SeedableRng};
use sketch_runner::*;

struct RotatedSquaresSketch {
    square_count: usize,
    max_rotation_deg: f64,
    seed: u64,
}

impl Default for RotatedSquaresSketch {
    fn default() -> Self {
        Self {
            square_count: 17,
            max_rotation_deg: 15.0,
            seed: 42,
        }
    }
}

impl Sketch for RotatedSquaresSketch {
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
                self.seed = rand::random();
                self.generate(drawing, ctx);
                log::info!("Regenerated with new seed: {}", self.seed);
                true
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.square_count = (self.square_count + 1).min(25);
                self.generate(drawing, ctx);
                log::info!("Square count: {}", self.square_count);
                true
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.square_count = (self.square_count - 1).max(5);
                self.generate(drawing, ctx);
                log::info!("Square count: {}", self.square_count);
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.max_rotation_deg = (self.max_rotation_deg + 2.0).min(45.0);
                self.generate(drawing, ctx);
                log::info!("Max rotation: {:.0} deg", self.max_rotation_deg);
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.max_rotation_deg = (self.max_rotation_deg - 2.0).max(1.0);
                self.generate(drawing, ctx);
                log::info!("Max rotation: {:.0} deg", self.max_rotation_deg);
                true
            }
            _ => false,
        }
    }
}

impl RotatedSquaresSketch {
    fn generate(&self, drawing: &mut Drawing, ctx: &SketchContext) {
        drawing.clear();

        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);

        // All squares share the same center point (center of drawing, shifted up for title)
        let center = drawing.center();
        let cx = center.x;
        let cy = center.y - 10.0; // Shift up slightly to account for title at bottom

        // Square size - fit nicely within A4 portrait with margins
        let margin = 20.0;
        let available_size =
            (drawing.width - margin * 2.0).min(drawing.height - margin * 2.0 - 20.0);
        let square_size = available_size * 0.68; // 80% of original 0.85
        let half = square_size / 2.0;

        // First, generate all rotation angles
        let rotations: Vec<f64> = (0..self.square_count)
            .map(|_| {
                rng.gen_range(-self.max_rotation_deg..=self.max_rotation_deg)
                    .to_radians()
            })
            .collect();

        // Generate squares with clipping to remove hidden lines
        // Each square is clipped by ALL squares on top of it (i+1..n)
        for i in 0..self.square_count {
            let rotation_rad = rotations[i];

            // Create the square
            let square = Element::rect(-half, -half, square_size, square_size)
                .rotate(rotation_rad)
                .translate(cx, cy)
                .stroke_width(0.5);

            // If there are squares on top, clip this one with all of them
            if i + 1 < self.square_count {
                // Create a group of all squares on top of this one as the clip shape
                let mut clip_group = Group::new();
                for &clip_rotation in &rotations[(i + 1)..] {
                    clip_group.push(
                        Element::rect(-half, -half, square_size, square_size)
                            .rotate(clip_rotation)
                            .translate(cx, cy),
                    );
                }

                // Apply inverted clip: keep the parts of this square that are OUTSIDE all squares on top
                let clipped_square = Element::clip(Element::group(clip_group))
                    .invert(true)
                    .add(square);
                drawing.add(clipped_square);
            } else {
                // Last square (topmost) doesn't need clipping
                drawing.add(square);
            }
        }

        // Add frame with title and signature
        let frame_options = FrameOptions::with_default_font(ctx.fonts)
            .expect("Default font not loaded")
            .margin_bottom(15.0)
            .with_signature();

        drawing.add(draw_frame_with_title(
            drawing,
            "Shifted Perspectives",
            &frame_options,
        ));

        log::info!(
            "Generated {} squares with max rotation {:.0} deg",
            self.square_count,
            self.max_rotation_deg
        );
    }
}

fn main() {
    let sketch = RotatedSquaresSketch::default();

    run_with_config(
        sketch,
        RunnerConfig::new("Shifted Perspectives")
            .with_size(800, 1100)
            .with_animation(false),
    );
}
