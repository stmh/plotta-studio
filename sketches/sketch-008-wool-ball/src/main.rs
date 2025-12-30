//! Wool Ball Sketch - Intertwined bezier curves forming a yarn ball pattern
//!
//! Controls:
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom
//! - Space: Fit drawing to window
//! - R: Reset view
//! - E: Export to drawing.svg
//! - G: Regenerate with new seed
//! - P: Plot to AxiDraw (requires `hardware` feature)
//! - Arrow Up/Down: Adjust number of curves
//! - Arrow Left/Right: Adjust curve complexity
//! - Escape: Quit

#[cfg(feature = "hardware")]
use drawing_plotter::{plot_in_background, PlotConfig, PlotEvent, PlotHandle};
use drawing_utils::{draw_frame_with_title, FrameOptions};
use sketch_runner::*;
use std::f64::consts::{PI, TAU};

struct WoolBallSketch {
    num_strands: usize,
    segments_per_strand: usize,
    seed: u64,
    #[cfg(feature = "hardware")]
    plot_handle: Option<PlotHandle>,
}

impl Default for WoolBallSketch {
    fn default() -> Self {
        Self {
            num_strands: 12,
            segments_per_strand: 8,
            seed: 42,
            #[cfg(feature = "hardware")]
            plot_handle: None,
        }
    }
}

/// Simple pseudo-random number generator for deterministic results
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> f64 {
        // LCG parameters
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        // Convert to 0..1 range
        (self.state >> 33) as f64 / (1u64 << 31) as f64
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next() * (max - min)
    }
}

impl Sketch for WoolBallSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::a4_landscape().with_background(Color::WHITE);
        self.generate(&mut drawing, ctx);
        drawing
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
            Key::Character(c) if c.as_str() == "g" => {
                self.seed = self.seed.wrapping_add(1);
                self.generate(drawing, ctx);
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
            Key::Named(NamedKey::ArrowUp) => {
                self.num_strands += 2;
                self.generate(drawing, ctx);
            }
            Key::Named(NamedKey::ArrowDown) => {
                if self.num_strands > 4 {
                    self.num_strands -= 2;
                    self.generate(drawing, ctx);
                }
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.segments_per_strand += 1;
                self.generate(drawing, ctx);
            }
            Key::Named(NamedKey::ArrowLeft) => {
                if self.segments_per_strand > 3 {
                    self.segments_per_strand -= 1;
                    self.generate(drawing, ctx);
                }
            }
            _ => {}
        }
    }
}

impl WoolBallSketch {
    fn generate(&self, drawing: &mut Drawing, ctx: &SketchContext) {
        drawing.clear();

        let center = drawing.center();
        let max_radius = drawing.width.min(drawing.height) * 0.35;

        let mut rng = SimpleRng::new(self.seed);

        // Generate multiple strands of intertwined bezier curves
        for strand in 0..self.num_strands {
            let strand_offset = (strand as f64 / self.num_strands as f64) * TAU;
            let strand_tilt = rng.range(-0.3, 0.3); // Slight variation in tilt

            // Each strand is a continuous path of connected bezier curves
            let mut path = Path::new();
            let mut first_point = true;

            // Starting angle for this strand
            let mut angle = strand_offset;

            // Varying radius creates the 3D ball effect
            let base_radius = max_radius * rng.range(0.6, 0.95);

            for seg in 0..self.segments_per_strand {
                let t = seg as f64 / self.segments_per_strand as f64;

                // Create oscillating radius to simulate going "behind" and "in front"
                let depth_oscillation = (t * TAU * 2.0 + strand_offset).sin();
                let r = base_radius * (0.7 + 0.3 * depth_oscillation);

                // Add some controlled randomness
                let r_variation = rng.range(0.9, 1.1);
                let current_r = r * r_variation;

                // Calculate point on the "surface" of the wool ball
                let x = center.x + angle.cos() * current_r;
                let y =
                    center.y + (angle.sin() * current_r * 0.8) + (strand_tilt * current_r * 0.3);

                let current_point = Point::new(x, y);

                if first_point {
                    path = path.move_to(current_point);
                    first_point = false;
                } else {
                    // Calculate control points for smooth bezier curves
                    let ctrl_angle1 = angle - rng.range(0.3, 0.6);
                    let ctrl_angle2 = angle - rng.range(0.1, 0.3);

                    let ctrl_r1 = current_r * rng.range(0.8, 1.3);
                    let ctrl_r2 = current_r * rng.range(0.9, 1.2);

                    let ctrl1 = Point::new(
                        center.x + ctrl_angle1.cos() * ctrl_r1,
                        center.y + ctrl_angle1.sin() * ctrl_r1 * 0.8 + strand_tilt * ctrl_r1 * 0.3,
                    );

                    let ctrl2 = Point::new(
                        center.x + ctrl_angle2.cos() * ctrl_r2,
                        center.y + ctrl_angle2.sin() * ctrl_r2 * 0.8 + strand_tilt * ctrl_r2 * 0.3,
                    );

                    path = path.cubic_to(ctrl1, ctrl2, current_point);
                }

                // Progress around the ball with some variation
                angle += PI * rng.range(0.3, 0.5);
            }

            // Add this strand to the drawing
            let gray_value = (50.0 + rng.next() * 100.0) as u8;
            drawing.add(
                Element::path(path)
                    .stroke_width(0.35 + rng.next() * 0.2)
                    .stroke_color(Color::gray(gray_value)),
            );
        }

        // Add some crossing strands for more intertwined look
        for i in 0..self.num_strands / 2 {
            let mut rng_cross = SimpleRng::new(self.seed.wrapping_add(1000 + i as u64));

            let start_angle = rng_cross.range(0.0, TAU);
            let arc_span = rng_cross.range(PI * 0.8, PI * 1.5);

            let r_start = max_radius * rng_cross.range(0.5, 0.9);
            let r_end = max_radius * rng_cross.range(0.5, 0.9);

            let start = Point::new(
                center.x + start_angle.cos() * r_start,
                center.y + start_angle.sin() * r_start * 0.85,
            );

            let end_angle = start_angle + arc_span;
            let end = Point::new(
                center.x + end_angle.cos() * r_end,
                center.y + end_angle.sin() * r_end * 0.85,
            );

            // Control points that create a nice arc
            let bulge = rng_cross.range(0.3, 0.6);
            let ctrl_r = max_radius * (1.0 + bulge);

            let ctrl1 = Point::new(
                center.x + (start_angle + arc_span * 0.33).cos() * ctrl_r,
                center.y + (start_angle + arc_span * 0.33).sin() * ctrl_r * 0.85,
            );

            let ctrl2 = Point::new(
                center.x + (start_angle + arc_span * 0.66).cos() * ctrl_r,
                center.y + (start_angle + arc_span * 0.66).sin() * ctrl_r * 0.85,
            );

            let path = Path::new().move_to(start).cubic_to(ctrl1, ctrl2, end);

            drawing.add(
                Element::path(path)
                    .stroke_width(0.4)
                    .stroke_color(Color::gray(70)),
            );
        }

        // Add a subtle center highlight (small clustered curves)
        for i in 0..5 {
            let mut rng_center = SimpleRng::new(self.seed.wrapping_add(2000 + i as u64));
            let small_r = max_radius * rng_center.range(0.1, 0.25);

            let start_angle = rng_center.range(0.0, TAU);
            let start = Point::new(
                center.x + start_angle.cos() * small_r,
                center.y + start_angle.sin() * small_r,
            );

            let end_angle = start_angle + rng_center.range(PI * 0.5, PI);
            let end = Point::new(
                center.x + end_angle.cos() * small_r * rng_center.range(0.8, 1.2),
                center.y + end_angle.sin() * small_r * rng_center.range(0.8, 1.2),
            );

            let ctrl1 = Point::new(
                center.x + rng_center.range(-small_r, small_r),
                center.y + rng_center.range(-small_r, small_r),
            );

            let ctrl2 = Point::new(
                center.x + rng_center.range(-small_r, small_r),
                center.y + rng_center.range(-small_r, small_r),
            );

            let path = Path::new().move_to(start).cubic_to(ctrl1, ctrl2, end);

            drawing.add(
                Element::path(path)
                    .stroke_width(0.3)
                    .stroke_color(Color::gray(40)),
            );
        }

        // Add frame with title and signature
        if let Some(frame_options) = FrameOptions::with_default_font(ctx.fonts) {
            let frame_options = frame_options
                .margin(8.0)
                .margin_bottom(12.0)
                .font_size(4.5)
                .stroke_width(0.4)
                .with_signature()
                .signature_height(4.5);

            drawing.add(draw_frame_with_title(
                drawing,
                "Tangled Thoughts",
                &frame_options,
            ));
        }

        log::info!(
            "Generated wool ball with {} strands, {} segments each ({} total curves)",
            self.num_strands,
            self.segments_per_strand,
            self.num_strands + self.num_strands / 2 + 5
        );
    }
}

fn main() {
    let sketch = WoolBallSketch::default();

    run_with_config(
        sketch,
        RunnerConfig::new("Tangled Thoughts")
            .with_size(1400, 900)
            .with_animation(false),
    );
}
