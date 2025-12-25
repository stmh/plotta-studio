//! Text rendering sketch demonstrating single-line fonts
//!
//! This sketch shows single-line font rendering using various font formats,
//! which are ideal for pen plotters as they draw text with single strokes.
//!
//! Controls:
//! - F: Cycle through fonts
//! - G: Cycle through sample texts
//! - Arrow Up/Down: Increase/decrease font size
//! - Arrow Left/Right: Adjust letter spacing
//! - E: Export to SVG
//! - Space: Fit to window
//! - Escape: Quit

use drawing_text::{hershey, Font, SvgFont, TextAlign, TextOptions, TextRenderer};
use sketch_runner::*;

/// Available fonts
enum FontType {
    HersheySimplex,
    ReliefSingleLine,
}

impl FontType {
    fn next(&self) -> Self {
        match self {
            FontType::HersheySimplex => FontType::ReliefSingleLine,
            FontType::ReliefSingleLine => FontType::HersheySimplex,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            FontType::HersheySimplex => "Hershey Simplex",
            FontType::ReliefSingleLine => "Relief SingleLine",
        }
    }
}

struct TextSketch {
    font_type: FontType,
    font_size: f64,
    letter_spacing: f64,
    sample_index: usize,
}

const SAMPLE_TEXTS: &[&str] = &[
    "Hello, World!",
    "PLOTTA STUDIO",
    "The quick brown fox\njumps over the lazy dog",
    "ABCDEFGHIJKLM\nNOPQRSTUVWXYZ",
    "0123456789",
    "Single-line fonts\nare perfect for\npen plotters!",
];

/// Embedded Relief SingleLine SVG font
const RELIEF_SINGLE_LINE_SVG: &str =
    include_str!("../../../fonts/svg/ReliefSingleLine-Regular.svg");

impl Default for TextSketch {
    fn default() -> Self {
        Self {
            font_type: FontType::HersheySimplex,
            font_size: 12.0, // 12mm tall text
            letter_spacing: 0.0,
            sample_index: 0,
        }
    }
}

impl Sketch for TextSketch {
    fn setup(&mut self) -> Drawing {
        let mut drawing = Drawing::a4_landscape();
        self.generate(&mut drawing);
        drawing
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing) {
        match key {
            Key::Character(c) if c.as_str() == "f" => {
                self.font_type = self.font_type.next();
                log::info!("Switched to font: {}", self.font_type.name());
                self.generate(drawing);
            }
            Key::Character(c) if c.as_str() == "g" => {
                self.sample_index = (self.sample_index + 1) % SAMPLE_TEXTS.len();
                self.generate(drawing);
            }
            Key::Character(c) if c.as_str() == "e" => {
                if let Err(e) = drawing_svg::export_svg(drawing, "text_drawing.svg") {
                    log::error!("Failed to export SVG: {e}");
                } else {
                    log::info!("Exported to text_drawing.svg");
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.font_size = (self.font_size + 1.0).min(50.0);
                self.generate(drawing);
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.font_size = (self.font_size - 1.0).max(3.0);
                self.generate(drawing);
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.letter_spacing += 0.05;
                self.generate(drawing);
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.letter_spacing = (self.letter_spacing - 0.05).max(-0.2);
                self.generate(drawing);
            }
            _ => {}
        }
    }
}

impl TextSketch {
    fn load_font(&self) -> Option<Box<dyn Font>> {
        match self.font_type {
            FontType::HersheySimplex => match hershey::load_simplex() {
                Ok(f) => Some(Box::new(f)),
                Err(e) => {
                    log::error!("Failed to load Hershey font: {e}");
                    None
                }
            },
            FontType::ReliefSingleLine => match SvgFont::from_str(RELIEF_SINGLE_LINE_SVG) {
                Ok(f) => Some(Box::new(f)),
                Err(e) => {
                    log::error!("Failed to load Relief SingleLine font: {e}");
                    None
                }
            },
        }
    }

    fn generate(&self, drawing: &mut Drawing) {
        drawing.clear();

        let font = match self.load_font() {
            Some(f) => f,
            None => return,
        };

        let renderer = TextRenderer::new();
        let center = drawing.center();

        // Main text - centered
        let text = SAMPLE_TEXTS[self.sample_index];
        let options = TextOptions::new(self.font_size)
            .at((center.x, center.y))
            .align(TextAlign::Center)
            .letter_spacing(self.letter_spacing);

        let layout = renderer.layout(text, font.as_ref(), &options);
        let strokes = layout.to_strokes(Style::default().with_stroke_width(0.5), 0.5);

        for stroke in strokes {
            drawing.add(Element::from_stroke(stroke));
        }

        // Title at top - show current font name
        let title_options = TextOptions::new(5.0) // 5mm
            .at((center.x, 15.0))
            .align(TextAlign::Center);

        let title = self.font_type.name();
        let title_layout = renderer.layout(title, font.as_ref(), &title_options);
        let title_strokes = title_layout.to_strokes(
            Style::default()
                .with_stroke_width(0.3)
                .with_stroke_color(Color::gray(100)),
            0.5,
        );

        for stroke in title_strokes {
            drawing.add(Element::from_stroke(stroke));
        }

        // Info text at bottom
        let info = format!(
            "Size: {:.0}mm  Spacing: {:.2}  (F=font, G=text, Arrows=adjust)",
            self.font_size, self.letter_spacing
        );
        let info_options = TextOptions::new(3.0) // 3mm
            .at((center.x, drawing.height - 15.0))
            .align(TextAlign::Center);

        let info_layout = renderer.layout(&info, font.as_ref(), &info_options);
        let info_strokes = info_layout.to_strokes(
            Style::default()
                .with_stroke_width(0.2)
                .with_stroke_color(Color::gray(120)),
            0.5,
        );

        for stroke in info_strokes {
            drawing.add(Element::from_stroke(stroke));
        }

        // Draw character set samples at smaller size
        let charset_y = drawing.height - 25.0;
        let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let lowercase = "abcdefghijklmnopqrstuvwxyz";
        let numbers = "0123456789";

        for (i, sample) in [uppercase, lowercase, numbers].iter().enumerate() {
            let y = charset_y - (i as f64 * 5.0);
            let sample_options = TextOptions::new(3.5) // 3.5mm
                .at((center.x, y))
                .align(TextAlign::Center);

            let sample_layout = renderer.layout(sample, font.as_ref(), &sample_options);
            let sample_strokes = sample_layout.to_strokes(
                Style::default()
                    .with_stroke_width(0.2)
                    .with_stroke_color(Color::gray(80)),
                0.5,
            );

            for stroke in sample_strokes {
                drawing.add(Element::from_stroke(stroke));
            }
        }

        // Border
        let margin = 10.0;
        drawing.add(
            Element::rect(
                margin,
                margin,
                drawing.width - margin * 2.0,
                drawing.height - margin * 2.0,
            )
            .stroke_width(0.5)
            .stroke_color(Color::gray(200)),
        );

        log::info!(
            "Generated text with {} strokes (font: {}, size: {:.0}mm)",
            drawing.stroke_count(),
            self.font_type.name(),
            self.font_size
        );
    }
}

fn main() {
    let sketch = TextSketch::default();

    run_with_config(
        sketch,
        RunnerConfig::new("Text Rendering - Single Line Fonts")
            .with_size(1400, 900)
            .with_animation(false),
    );
}
