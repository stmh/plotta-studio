//! Text rendering sketch demonstrating single-line fonts
//!
//! This sketch shows single-line font rendering using various font formats,
//! which are ideal for pen plotters as they draw text with single strokes.
//!
//! Controls:
//! - F: Cycle through fonts
//! - G: Cycle through sample texts
//! - D: Toggle debug visualization (baselines, bounding boxes)
//! - Arrow Up/Down: Increase/decrease font size
//! - Arrow Left/Right: Adjust letter spacing
//! - E: Export to SVG
//! - Space: Fit to window
//! - Escape: Quit

use drawing_text::{FontFormat, TextAlign, TextLayout, TextOptions, TextRenderer};
use sketch_runner::*;

/// Embedded Relief SingleLine SVG font
const RELIEF_SINGLE_LINE_SVG: &str =
    include_str!("../../../fonts/svg/ReliefSingleLine-Regular.svg");

/// Embedded VSF fonts (vintage single-line fonts)
const ASTEROIDS_VSF: &str = include_str!("../../../fonts/vsf/asteroids.vsf");
const APPLE410_VSF: &str = include_str!("../../../fonts/vsf/apple410.vsf");
const MINF_VSF: &str = include_str!("../../../fonts/vsf/minf.vsf");

const SAMPLE_TEXTS: &[&str] = &[
    "Hello, World!",
    "PLOTTA STUDIO",
    "The quick brown fox\njumps over the lazy dog",
    "ABCDEFGHIJKLM\nNOPQRSTUVWXYZ",
    "0123456789",
    "Single-line fonts\nare perfect for\npen plotters!",
];

struct TextSketch {
    font_names: Vec<String>,
    font_index: usize,
    font_size: f64,
    letter_spacing: f64,
    sample_index: usize,
    show_debug: bool,
    fonts_loaded: bool,
}

impl Default for TextSketch {
    fn default() -> Self {
        Self {
            font_names: Vec::new(),
            font_index: 0,
            font_size: 12.0, // 12mm tall text
            letter_spacing: 0.0,
            sample_index: 0,
            show_debug: false,
            fonts_loaded: false,
        }
    }
}

impl Sketch for TextSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        // Load additional fonts (SVG and VSF) into the font manager
        if !self.fonts_loaded {
            // Load SVG font
            if let Err(e) = ctx
                .fonts
                .load_from_str(RELIEF_SINGLE_LINE_SVG, FontFormat::SvgFont)
            {
                log::warn!("Failed to load SVG font: {}", e);
            }

            // Load VSF fonts
            if let Err(e) = ctx.fonts.load_from_str(ASTEROIDS_VSF, FontFormat::Vsf) {
                log::warn!("Failed to load Asteroids VSF font: {}", e);
            }
            if let Err(e) = ctx.fonts.load_from_str(APPLE410_VSF, FontFormat::Vsf) {
                log::warn!("Failed to load Apple410 VSF font: {}", e);
            }
            if let Err(e) = ctx.fonts.load_from_str(MINF_VSF, FontFormat::Vsf) {
                log::warn!("Failed to load Minf VSF font: {}", e);
            }

            // Get sorted list of all available fonts
            self.font_names = ctx.fonts.registry().list();
            self.font_names.sort();
            self.fonts_loaded = true;
        }

        let mut drawing = Drawing::a4_landscape();
        self.generate(&mut drawing, ctx);
        drawing
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, ctx: &SketchContext) {
        match key {
            Key::Character(c) if c.as_str() == "f" => {
                self.font_index = (self.font_index + 1) % self.font_names.len();
                log::info!("Switched to font: {}", self.current_font_name());
                self.generate(drawing, ctx);
            }
            Key::Character(c) if c.as_str() == "g" => {
                self.sample_index = (self.sample_index + 1) % SAMPLE_TEXTS.len();
                self.generate(drawing, ctx);
            }
            Key::Character(c) if c.as_str() == "d" => {
                self.show_debug = !self.show_debug;
                log::info!(
                    "Debug visualization: {}",
                    if self.show_debug { "ON" } else { "OFF" }
                );
                self.generate(drawing, ctx);
            }
            Key::Character(c) if c.as_str() == "e" => {
                if let Err(e) = drawing_svg::export_svg(drawing, "text_drawing.svg", ctx.render) {
                    log::error!("Failed to export SVG: {e}");
                } else {
                    log::info!("Exported to text_drawing.svg");
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.font_size = (self.font_size + 1.0).min(50.0);
                self.generate(drawing, ctx);
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.font_size = (self.font_size - 1.0).max(3.0);
                self.generate(drawing, ctx);
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.letter_spacing += 0.05;
                self.generate(drawing, ctx);
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.letter_spacing = (self.letter_spacing - 0.05).max(-0.2);
                self.generate(drawing, ctx);
            }
            _ => {}
        }
    }
}

impl TextSketch {
    fn current_font_name(&self) -> &str {
        &self.font_names[self.font_index]
    }

    fn load_font(&self, ctx: &SketchContext) -> Option<FontRef> {
        ctx.fonts.registry().get(self.current_font_name())
    }

    fn generate(&self, drawing: &mut Drawing, ctx: &SketchContext) {
        drawing.clear();

        let font = match self.load_font(ctx) {
            Some(f) => f,
            None => {
                log::error!("Failed to load font: {}", self.current_font_name());
                return;
            }
        };

        let renderer = TextRenderer::new();
        let center = drawing.center();

        // Main text - centered
        let text = SAMPLE_TEXTS[self.sample_index];
        let options = TextOptions::new(self.font_size)
            .at((center.x, center.y))
            .align(TextAlign::Center)
            .letter_spacing(self.letter_spacing);

        let layout = renderer.layout(text, font.clone(), &options);
        let strokes = layout.to_strokes(ResolvedStyle::default().with_stroke_width(0.5), 0.5);

        for stroke in strokes {
            drawing.add(Element::from_stroke(stroke));
        }

        // Debug visualization
        if self.show_debug {
            self.draw_debug(drawing, &layout);
        }

        // Title at top - show current font name
        let title_options = TextOptions::new(5.0) // 5mm
            .at((center.x, 15.0))
            .align(TextAlign::Center);

        let title = self.current_font_name();
        let title_layout = renderer.layout(title, font.clone(), &title_options);
        let title_strokes = title_layout.to_strokes(
            ResolvedStyle::default()
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

        let info_layout = renderer.layout(&info, font.clone(), &info_options);
        let info_strokes = info_layout.to_strokes(
            ResolvedStyle::default()
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

            let sample_layout = renderer.layout(sample, font.clone(), &sample_options);
            let sample_strokes = sample_layout.to_strokes(
                ResolvedStyle::default()
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
            "Generated text with {} strokes (font: {}, size: {:.0}mm{})",
            drawing.stroke_count(ctx.render),
            self.current_font_name(),
            self.font_size,
            if self.show_debug { ", debug ON" } else { "" }
        );
    }

    /// Draw debug visualization: baselines, ascender/descender lines, glyph bounding boxes
    fn draw_debug(&self, drawing: &mut Drawing, layout: &TextLayout) {
        let metrics = layout.font.metrics();

        // Colors for debug elements
        let baseline_color = Color::rgb(255, 0, 0); // Red for baseline
        let ascender_color = Color::rgb(0, 150, 0); // Green for ascender
        let descender_color = Color::rgb(0, 0, 255); // Blue for descender
        let bbox_color = Color::rgb(255, 128, 0); // Orange for glyph bounding boxes
        let advance_color = Color::rgb(128, 0, 255); // Purple for advance width markers

        let debug_stroke_width = 0.15;

        // Track lines we've already drawn baselines for (to avoid duplicates)
        let mut drawn_baselines: std::collections::HashSet<i32> = std::collections::HashSet::new();

        for positioned_glyph in &layout.glyphs {
            let pos = positioned_glyph.position;
            let scale = positioned_glyph.scale;
            let c = positioned_glyph.char;

            // Look up the actual glyph from the font
            let glyph = match layout.font.glyph(c) {
                Some(g) => g,
                None => continue,
            };

            // Calculate metric lines (scaled to drawing units)
            // Font metrics use Y-up convention, rendering negates Y for screen coordinates
            // So: ascender (positive in font) should be above baseline (smaller screen Y)
            //     descender (negative in font) should be below baseline (larger screen Y)
            let baseline_y = pos.y;
            let ascender_y = pos.y - metrics.ascender * scale; // ascender is positive, subtract to go up
            let descender_y = pos.y - metrics.descender * scale; // descender is negative, subtracting negative goes down

            // Use baseline_y as key to check if we've drawn this line
            let line_key = (baseline_y * 100.0) as i32;

            // Draw baseline, ascender, descender lines (once per text line)
            if !drawn_baselines.contains(&line_key) {
                drawn_baselines.insert(line_key);

                // Find extent of this line (approximate with layout bounds or use wide line)
                let line_start_x = if let Some(bounds) = &layout.bounds {
                    bounds.x0 - 5.0
                } else {
                    pos.x - 50.0
                };
                let line_end_x = if let Some(bounds) = &layout.bounds {
                    bounds.x1 + 5.0
                } else {
                    pos.x + 200.0
                };

                // Baseline (red)
                drawing.add(
                    Element::line((line_start_x, baseline_y), (line_end_x, baseline_y))
                        .stroke_width(debug_stroke_width)
                        .stroke_color(baseline_color),
                );

                // Ascender line (green)
                drawing.add(
                    Element::line((line_start_x, ascender_y), (line_end_x, ascender_y))
                        .stroke_width(debug_stroke_width)
                        .stroke_color(ascender_color),
                );

                // Descender line (blue)
                drawing.add(
                    Element::line((line_start_x, descender_y), (line_end_x, descender_y))
                        .stroke_width(debug_stroke_width)
                        .stroke_color(descender_color),
                );
            }

            // Draw glyph bounding box (orange)
            if let Some(glyph_bounds) = glyph.bounds() {
                // Scale and translate the glyph bounds
                // Font coords are Y-up, screen is Y-down, so negate Y
                // y1 is top in font coords, becomes top in screen (smaller Y)
                // y0 is bottom in font coords, becomes bottom in screen (larger Y)
                let x = glyph_bounds.x0 * scale + pos.x;
                let y = -glyph_bounds.y1 * scale + pos.y; // Use y1 (top) and negate for screen top
                let w = glyph_bounds.width() * scale;
                let h = glyph_bounds.height() * scale;

                drawing.add(
                    Element::rect(x, y, w, h)
                        .stroke_width(debug_stroke_width)
                        .stroke_color(bbox_color),
                );
            }

            // Draw advance width marker (purple vertical line at next glyph position)
            let advance_x = pos.x + glyph.advance_width * scale;
            drawing.add(
                Element::line((advance_x, ascender_y), (advance_x, descender_y))
                    .stroke_width(debug_stroke_width * 0.5)
                    .stroke_color(advance_color),
            );
        }
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
