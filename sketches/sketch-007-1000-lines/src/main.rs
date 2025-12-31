//! 1000 Lines Sketch - diagonal lines with trapezoid spacing
//!
//! Creates 1000 diagonal lines where the spacing at the top is narrower
//! than at the bottom, creating a trapezoid effect. The lines are clipped
//! to a rectangle with 1.5cm margins on A4 landscape paper.
//!
//! Controls:
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom
//! - Space: Fit drawing to window
//! - R: Reset view
//! - E: Export to SVG (built-in)
//! - G: Regenerate drawing
//! - P: Plot to AxiDraw (built-in, requires `hardware` feature)
//! - Escape: Quit

use drawing_utils::{draw_frame_with_title, FrameOptions};
use sketch_runner::*;

const NUM_LINES: usize = 1000;
const MARGIN: f64 = 15.0; // 1.5 cm margin on all sides

struct LinesSketch;

impl LinesSketch {
    fn new() -> Self {
        Self
    }

    fn build_drawing(&self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::a4_landscape().with_background(Color::WHITE);

        // Calculate the clipping rectangle (content area with 1.5cm margin)
        let _clip_x = MARGIN;
        let _clip_y = MARGIN;
        let clip_width = drawing.width - MARGIN * 2.0;
        let clip_height = drawing.height - MARGIN * 2.0;

        // Create a group for all the diagonal lines
        let mut lines_group = Group::new();

        // We want narrower spacing at top, wider at bottom
        // This creates a trapezoid effect
        //
        // For each line i (0..NUM_LINES):
        //   - Top point x varies from left to right with narrower distribution
        //   - Bottom point x varies from left to right with wider distribution
        //
        // The lines go from top to bottom, diagonal

        for i in 0..NUM_LINES {
            let t = i as f64 / (NUM_LINES - 1) as f64;

            let top_squeeze = 1.0; // Top is 40% of the width (centered)
            let top_margin = (1.50 - top_squeeze) / 2.0;
            let top_x = top_margin + t * drawing.width * top_squeeze;

            // At the bottom: full width distribution
            let bottom_x = drawing.width * 1.5 * t - drawing.width * 0.25;

            // Y positions
            let top_y = 0.0;
            let bottom_y = drawing.height;

            lines_group.push(
                Element::line((top_x, top_y), (bottom_x, bottom_y))
                    .stroke_width(0.3)
                    .stroke_color(Color::BLACK),
            );
        }

        // Create a clipped group - clip the lines to the content area
        let clip_rect = Element::rect(10.0, 10.0, drawing.width - 20.0, drawing.height - 25.0);
        let clipped_lines = Element::clip(clip_rect).add(Element::group(lines_group));

        drawing.add(clipped_lines);

        // Add frame with title and signature
        if let Some(frame_options) = FrameOptions::with_default_font(ctx.fonts) {
            let frame_options = frame_options
                .margin(5.0)
                .margin_bottom(10.0) // Extra space for title
                .with_signature();

            drawing.add(draw_frame_with_title(
                &drawing,
                "1000 lines #1",
                &frame_options,
            ));
        } else {
            log::warn!("Default font not loaded, adding frame without title");
            // Add simple frame without title
            drawing.add(
                Element::rect(MARGIN, MARGIN, clip_width, clip_height)
                    .stroke_width(0.35)
                    .stroke_color(Color::BLACK),
            );
        }

        log::info!(
            "Generated {} lines, {} total strokes",
            NUM_LINES,
            drawing.stroke_count(ctx.render)
        );

        drawing
    }
}

impl Sketch for LinesSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        self.build_drawing(ctx)
    }

    fn update(&mut self, _drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, ctx: &SketchContext) -> bool {
        match key {
            Key::Character(c) if c.as_str() == "g" => {
                log::info!("Regenerating drawing...");
                *drawing = self.build_drawing(ctx);
                true
            }
            _ => false,
        }
    }
}

fn main() {
    let sketch = LinesSketch::new();

    run_with_config(
        sketch,
        RunnerConfig::new("1000 Lines #1")
            .with_size(1400, 900) // Landscape aspect ratio
            .with_animation(false),
    );
}
