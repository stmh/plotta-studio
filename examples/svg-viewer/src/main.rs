//! SVG Viewer Sketch - Cycles through imported SVG files
//!
//! Controls:
//! - Left/Right arrows: Cycle through SVGs
//! - Middle mouse drag: Pan
//! - Scroll wheel: Zoom
//! - Space: Fit drawing to window
//! - R: Reset view
//! - E: Export to SVG (built-in)
//! - P: Plot to AxiDraw (built-in, requires `hardware` feature)
//! - Escape: Quit

use drawing_svg::{import_svg_string, ImportResult};
use drawing_text::{FontManager, Hershey};
use sketch_runner::*;

/// Embedded SVG files
const SVGS: &[(&str, &str)] = &[
    ("01-spiral.svg", include_str!("../assets/01-spiral.svg")),
    ("02-star.svg", include_str!("../assets/02-star.svg")),
    ("03-nurse.svg", include_str!("../assets/03-nurse.svg")),
    ("04-clipped.svg", include_str!("../assets/04-clipped.svg")),
    ("05-flower.svg", include_str!("../assets/05-flower.svg")),
    ("06-cat-face.svg", include_str!("../assets/06-cat-face.svg")),
];

struct SvgViewerSketch {
    current_index: usize,
    imported_svgs: Vec<ImportResult>,
    font_manager: FontManager,
}

impl SvgViewerSketch {
    fn new() -> Self {
        // Pre-import all SVGs
        let imported_svgs: Vec<ImportResult> = SVGS
            .iter()
            .map(|(name, content)| {
                import_svg_string(content).unwrap_or_else(|e| {
                    log::error!("Failed to import {}: {}", name, e);
                    // Return empty result on error
                    ImportResult {
                        drawing: Drawing::new(200.0, 200.0),
                        warnings: vec![],
                    }
                })
            })
            .collect();

        let font_manager = FontManager::new();
        if let Err(e) = font_manager.load_hershey(Hershey::Simplex) {
            log::error!("Failed to load font: {}", e);
        }

        Self {
            current_index: 0,
            imported_svgs,
            font_manager,
        }
    }

    fn current_name(&self) -> &str {
        SVGS[self.current_index].0
    }

    fn current_svg(&self) -> &ImportResult {
        &self.imported_svgs[self.current_index]
    }

    fn next(&mut self) {
        self.current_index = (self.current_index + 1) % SVGS.len();
    }

    fn prev(&mut self) {
        self.current_index = (self.current_index + SVGS.len() - 1) % SVGS.len();
    }

    fn build_drawing(&self) -> Drawing {
        let mut drawing = Drawing::a4_landscape();
        let margin = 20.0;

        // Add title showing current SVG name
        let title = format!(
            "{} ({}/{})",
            self.current_name(),
            self.current_index + 1,
            SVGS.len()
        );

        if let Some(font) = self.font_manager.get("Simplex") {
            drawing.add(
                Element::text(&title, font)
                    .text_size(12.0)
                    .translate(margin, margin + 10.0),
            );
        }

        // Get the imported SVG content
        let imported = self.current_svg();
        let svg_drawing = &imported.drawing;

        // Calculate scaling to fit the SVG in the available space
        let available_width = drawing.width - margin * 2.0;
        let available_height = drawing.height - margin * 3.0 - 20.0; // Extra space for title

        let scale_x = available_width / svg_drawing.width;
        let scale_y = available_height / svg_drawing.height;
        let scale = scale_x.min(scale_y).min(2.0); // Cap at 2x to avoid too large

        // Center the SVG
        let scaled_width = svg_drawing.width * scale;
        let scaled_height = svg_drawing.height * scale;
        let offset_x = margin + (available_width - scaled_width) / 2.0;
        let offset_y = margin + 30.0 + (available_height - scaled_height) / 2.0;

        // Create a group for the imported content
        // We need to: translate to position, then scale around that position
        // Transform order in kurbo: operations are applied right-to-left
        // So we build: translate(offset) * scale * translate(-svg_center) * content
        // This centers the SVG at origin, scales it, then moves to final position
        let svg_center_x = svg_drawing.width / 2.0;
        let svg_center_y = svg_drawing.height / 2.0;
        let final_center_x = offset_x + scaled_width / 2.0;
        let final_center_y = offset_y + scaled_height / 2.0;

        let mut content_group = Group::new();
        for element in &svg_drawing.elements {
            content_group.push(element.clone());
        }

        // Translate so SVG center is at origin, scale, then translate to final position
        drawing.add(
            Element::group(content_group)
                .translate(-svg_center_x, -svg_center_y)
                .scale_uniform(scale)
                .translate(final_center_x, final_center_y),
        );

        // Add border around the SVG area
        drawing.add(
            Element::rect(
                offset_x - 5.0,
                offset_y - 5.0,
                scaled_width + 10.0,
                scaled_height + 10.0,
            )
            .stroke_width(0.5)
            .stroke_color(Color::gray(180)),
        );

        // Show warnings if any
        if !imported.warnings.is_empty() {
            log::warn!(
                "{} has {} import warnings",
                self.current_name(),
                imported.warnings.len()
            );
            for warning in &imported.warnings {
                log::warn!("  {:?}", warning);
            }
        }

        // Add navigation hint at bottom
        if let Some(font) = self.font_manager.get("Simplex") {
            drawing.add(
                Element::text("< Left/Right arrows to navigate >", font)
                    .text_size(8.0)
                    .translate(drawing.width / 2.0 - 80.0, drawing.height - margin),
            );
        }

        log::info!(
            "Viewing: {} ({} elements, {} strokes)",
            self.current_name(),
            svg_drawing.elements.len(),
            svg_drawing.elements.len() // approximate
        );

        drawing
    }
}

impl Sketch for SvgViewerSketch {
    fn setup(&mut self, _ctx: &SketchContext) -> Drawing {
        self.build_drawing()
    }

    fn update(&mut self, _drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, _ctx: &SketchContext) -> bool {
        match key {
            Key::Named(NamedKey::ArrowRight) => {
                self.next();
                *drawing = self.build_drawing();
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.prev();
                *drawing = self.build_drawing();
                true
            }
            _ => false,
        }
    }
}

fn main() {
    let sketch = SvgViewerSketch::new();

    run_with_config(
        sketch,
        RunnerConfig::new("SVG Viewer")
            .with_size(1200, 800)
            .with_animation(false),
    );
}
