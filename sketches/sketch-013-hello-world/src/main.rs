//! Hello World sketch
//!
//! A minimal sketch showing centered "Hello World" text in the Relief SingleLine font
//! on a 180 x 257 mm canvas.
//!
//! Controls:
//! - E: Export to SVG (built-in)
//! - P: Plot to AxiDraw (built-in, requires `hardware` feature)
//! - Space: Fit to window
//! - Escape: Quit

use drawing_text::{TextAlign, TextOptions, TextRenderer, DEFAULT_FONT_NAME};
use drawing_utils::{draw_frame_with_title, FrameOptions};
use sketch_runner::*;

struct HelloWorldSketch;

impl Default for HelloWorldSketch {
    fn default() -> Self {
        Self
    }
}

impl Sketch for HelloWorldSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::new(148.0, 210.0);
        self.generate(&mut drawing, ctx);
        drawing
    }
}

impl HelloWorldSketch {
    fn generate(&self, drawing: &mut Drawing, ctx: &SketchContext) {
        drawing.clear();

        let font = match ctx.fonts.default_font() {
            Some(f) => f,
            None => {
                log::error!("Default font '{}' not available", DEFAULT_FONT_NAME);
                return;
            }
        };

        let renderer = TextRenderer::new();
        let center = drawing.center();

        // Main text - centered
        let options = TextOptions::new(12.0)
            .at((center.x, center.y))
            .align(TextAlign::Center);

        let layout = renderer.layout("Hello World", font.clone(), &options);
        let strokes = layout.to_strokes(ResolvedStyle::default().with_stroke_width(0.5), 0.5);

        for stroke in strokes {
            drawing.add(Element::from_stroke(stroke));
        }

        // Frame with title and signature (10 mm inset)
        let frame_options = FrameOptions::with_default_font(ctx.fonts)
            .expect("Default font not loaded")
            .margin_left(8.0)
            .margin_top(8.0)
            .margin_right(8.0)
            .margin_bottom(16.0)
            .with_signature();

        drawing.add(draw_frame_with_title(
            drawing,
            "Hello World",
            &frame_options,
        ));

        log::info!(
            "Generated hello world with {} strokes",
            drawing.stroke_count(ctx.render)
        );
    }
}

fn main() {
    let sketch = HelloWorldSketch;

    run_with_config(
        sketch,
        RunnerConfig::new("Hello World - Relief SingleLine")
            .with_size(900, 1200)
            .with_animation(false),
    );
}
