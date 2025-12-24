//! Example sketch demonstrating plotta-studio features
//!
//! Controls:
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom
//! - Space: Fit drawing to window
//! - R: Reset view
//! - S: Save to drawing.json
//! - E: Export to drawing.svg
//! - G: Regenerate drawing
//! - P: Plot to AxiDraw (requires `hardware` feature)
//! - Escape: Quit

#[cfg(feature = "hardware")]
use drawing_plotter::{plot_in_background, PlotConfig, PlotEvent, PlotHandle};
use sketch_runner::*;
use std::f64::consts::{PI, TAU};

struct RadialSketch {
    num_circles: usize,
    num_rays: usize,
    seed: u64,
    #[cfg(feature = "hardware")]
    plot_handle: Option<PlotHandle>,
}

impl Default for RadialSketch {
    fn default() -> Self {
        Self {
            num_circles: 8,
            num_rays: 24,
            seed: 42,
            #[cfg(feature = "hardware")]
            plot_handle: None,
        }
    }
}

impl Sketch for RadialSketch {
    fn setup(&mut self) -> Drawing {
        let mut drawing = Drawing::a4_landscape();
        self.generate(&mut drawing);
        drawing
    }

    fn update(&mut self, _drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        #[cfg(feature = "hardware")]
        {
            // Check for plot events
            if let Some(ref handle) = self.plot_handle {
                for event in handle.drain_events() {
                    match event {
                        PlotEvent::Started { total_strokes } => {
                            log::info!("Plotting started: {} strokes", total_strokes);
                        }
                        PlotEvent::StrokeStart { index, total } => {
                            log::debug!("Starting stroke {}/{}", index + 1, total);
                        }
                        PlotEvent::StrokeComplete { index, total } => {
                            log::info!("Stroke {}/{} complete", index + 1, total);
                        }
                        PlotEvent::MoveTo { position, pen_down } => {
                            log::trace!(
                                "Move to ({:.1}, {:.1}) pen {}",
                                position.x,
                                position.y,
                                if pen_down { "down" } else { "up" }
                            );
                        }
                        PlotEvent::Completed => {
                            log::info!("Plotting completed!");
                        }
                        PlotEvent::Error(e) => {
                            log::error!("Plotting error: {}", e);
                        }
                    }
                }

                // Clean up finished handle
                if !handle.is_running() {
                    self.plot_handle = None;
                }
            }
        }
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing) {
        match key {
            Key::Character(c) if c.as_str() == "g" => {
                // Regenerate with new seed
                self.seed = self.seed.wrapping_add(1);
                self.generate(drawing);
            }
            Key::Character(c) if c.as_str() == "e" => {
                // Export SVG
                if let Err(e) = drawing_svg::export_svg(drawing, "drawing.svg") {
                    log::error!("Failed to export SVG: {e}");
                } else {
                    log::info!("Exported to drawing.svg");
                }
            }
            #[cfg(feature = "hardware")]
            Key::Character(c) if c.as_str() == "p" => {
                // Plot to AxiDraw
                if self.plot_handle.is_some() {
                    log::warn!("Plotting already in progress");
                } else {
                    log::info!("Starting plot...");
                    match plot_in_background(drawing.clone(), PlotConfig::default(), None) {
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
                self.num_circles += 1;
                self.generate(drawing);
            }
            Key::Named(NamedKey::ArrowDown) => {
                if self.num_circles > 1 {
                    self.num_circles -= 1;
                    self.generate(drawing);
                }
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.num_rays += 4;
                self.generate(drawing);
            }
            Key::Named(NamedKey::ArrowLeft) => {
                if self.num_rays > 4 {
                    self.num_rays -= 4;
                    self.generate(drawing);
                }
            }
            _ => {}
        }
    }
}

impl RadialSketch {
    fn generate(&self, drawing: &mut Drawing) {
        drawing.clear();

        let center = drawing.center();
        let max_radius = drawing.width.min(drawing.height) * 0.45;

        // Concentric circles
        for i in 1..=self.num_circles {
            let t = i as f64 / self.num_circles as f64;
            let radius = max_radius * t;

            drawing.add(
                Element::circle(center, radius)
                    .stroke_width(0.5)
                    .stroke_color(Color::gray(100)),
            );
        }

        // Radial rays
        for i in 0..self.num_rays {
            let angle = (i as f64 / self.num_rays as f64) * TAU;
            let inner_radius = max_radius * 0.1;
            let outer_radius = max_radius;

            let inner = Point::new(
                center.x + angle.cos() * inner_radius,
                center.y + angle.sin() * inner_radius,
            );
            let outer = Point::new(
                center.x + angle.cos() * outer_radius,
                center.y + angle.sin() * outer_radius,
            );

            drawing.add(
                Element::line(inner, outer)
                    .stroke_width(0.3)
                    .stroke_color(Color::gray(150)),
            );
        }

        // Decorative elements using groups
        let mut decorations = Group::new();

        for i in 0..6 {
            let angle = (i as f64 / 6.0) * TAU + PI / 6.0;
            let dist = max_radius * 0.6;
            let pos = Point::new(center.x + angle.cos() * dist, center.y + angle.sin() * dist);

            // Hexagon at each position
            decorations.push(
                Element::polygon(pos, 15.0, 6)
                    .rotate_around(angle, pos)
                    .stroke_width(0.8)
                    .stroke_color(Color::BLACK),
            );

            // Small circle inside
            decorations.push(
                Element::circle(pos, 8.0)
                    .stroke_width(0.5)
                    .stroke_color(Color::BLACK),
            );
        }

        drawing.add(Element::group(decorations));

        // Center decoration
        drawing.add(
            Element::polygon(center, 20.0, 8)
                .rotate_deg(22.5)
                .stroke_width(1.0),
        );

        drawing.add(Element::circle(center, 12.0).stroke_width(1.0));

        // Bezier decorations
        for i in 0..4 {
            let base_angle = (i as f64 / 4.0) * TAU;
            let r1 = max_radius * 0.3;
            let r2 = max_radius * 0.7;

            let start = Point::new(
                center.x + base_angle.cos() * r1,
                center.y + base_angle.sin() * r1,
            );

            let ctrl1 = Point::new(
                center.x + (base_angle + 0.3).cos() * r2,
                center.y + (base_angle + 0.3).sin() * r2,
            );

            let ctrl2 = Point::new(
                center.x + (base_angle + 0.6).cos() * r2,
                center.y + (base_angle + 0.6).sin() * r2,
            );

            let end = Point::new(
                center.x + (base_angle + PI / 2.0).cos() * r1,
                center.y + (base_angle + PI / 2.0).sin() * r1,
            );

            let path = Path::new()
                .move_to(start)
                .cubic_to(ctrl1, ctrl2, end);

            drawing.add(
                Element::path(path)
                    .stroke_width(0.6)
                    .stroke_color(Color::rgb(50, 50, 50)),
            );
        }

        // Border rectangle
        let margin = 10.0;
        drawing.add(
            Element::rect(margin, margin, drawing.width - margin * 2.0, drawing.height - margin * 2.0)
                .stroke_width(1.0),
        );

        log::info!(
            "Generated {} elements, {} strokes",
            drawing.elements.len(),
            drawing.stroke_count()
        );
    }
}

fn main() {
    let sketch = RadialSketch::default();

    run_with_config(
        sketch,
        RunnerConfig::new("Radial Sketch")
            .with_size(1400, 900)
            .with_animation(false),
    );
}
