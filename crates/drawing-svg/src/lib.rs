//! SVG import/export for plotta-studio
//!
//! Note: Import is lossy - only path/line data is preserved.
//! Complex SVG features (gradients, filters, text, etc.) are ignored.

mod import;

use drawing_core::{Color, Drawing, RenderContext, Stroke};
use std::io::Write;
use std::path::Path;
use thiserror::Error;

// Re-export import functionality
pub use import::{
    import_svg, import_svg_string, import_svg_string_with_options, import_svg_with_options,
    FillBehavior, ImportOptions, ImportResult, ImportWarning,
};

#[derive(Error, Debug)]
pub enum SvgError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Export a drawing to SVG format
pub fn export_svg(
    drawing: &Drawing,
    path: impl AsRef<Path>,
    ctx: &RenderContext,
) -> Result<(), SvgError> {
    let svg = drawing_to_svg_string(drawing, ctx);
    std::fs::write(path, svg)?;
    Ok(())
}

/// Convert a drawing to an SVG string
pub fn drawing_to_svg_string(drawing: &Drawing, ctx: &RenderContext) -> String {
    let mut svg = String::new();

    // SVG header
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" 
     width="{width}mm" 
     height="{height}mm" 
     viewBox="0 0 {width} {height}">
"#,
        width = drawing.width,
        height = drawing.height
    ));

    // Background
    if drawing.background != Color::WHITE {
        svg.push_str(&format!(
            r#"  <rect width="{}" height="{}" fill="{}"/>
"#,
            drawing.width,
            drawing.height,
            color_to_hex(drawing.background)
        ));
    }

    // Flatten and export strokes
    let strokes = drawing.flatten(ctx);
    for stroke in strokes {
        if stroke.points.len() < 2 {
            continue;
        }

        svg.push_str(&stroke_to_svg(&stroke));
    }

    svg.push_str("</svg>\n");
    svg
}

/// Generate SVG path data (M/L commands) from points
fn points_to_svg_path_data(points: &[drawing_core::Point], closed: bool) -> String {
    if points.is_empty() {
        return String::new();
    }

    let mut d = String::new();
    d.push_str(&format!("M{:.3},{:.3}", points[0].x, points[0].y));

    for pt in &points[1..] {
        d.push_str(&format!(" L{:.3},{:.3}", pt.x, pt.y));
    }

    if closed {
        d.push_str(" Z");
    }

    d
}

fn stroke_to_svg(stroke: &Stroke) -> String {
    let d = points_to_svg_path_data(&stroke.points, stroke.closed);

    format!(
        r#"  <path d="{}" fill="none" stroke="{}" stroke-width="{:.3}" stroke-linecap="round" stroke-linejoin="round"/>
"#,
        d,
        color_to_hex(stroke.style.stroke_color),
        stroke.style.stroke_width
    )
}

fn color_to_hex(c: Color) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        format!("rgba({},{},{},{:.3})", c.r, c.g, c.b, c.a as f64 / 255.0)
    }
}

/// Export drawing to SVG and write to a writer
pub fn write_svg<W: Write>(
    drawing: &Drawing,
    writer: &mut W,
    ctx: &RenderContext,
) -> Result<(), SvgError> {
    let svg = drawing_to_svg_string(drawing, ctx);
    writer.write_all(svg.as_bytes())?;
    Ok(())
}

// ============================================================================
// SVG Recording - Record optimized strokes as they would be plotted
// ============================================================================

use drawing_plotter::OwnedOptimizedStroke;

/// Options for recording optimized strokes to SVG
#[derive(Debug, Clone)]
pub struct RecordOptions {
    /// Show pen-up travel paths as dashed lines
    pub show_travel: bool,
    /// Color for travel lines (default: light gray)
    pub travel_color: Color,
    /// Show direction arrows at stroke starts
    pub show_direction: bool,
    /// Size of direction arrows in mm (default: 2.0)
    pub arrow_size: f64,
    /// Unified stroke width for all strokes in mm
    pub stroke_width: f64,
}

impl Default for RecordOptions {
    fn default() -> Self {
        Self {
            show_travel: false,
            travel_color: Color::rgb(200, 200, 200), // Light gray
            show_direction: false,
            arrow_size: 2.0,
            stroke_width: 0.3,
        }
    }
}

impl RecordOptions {
    /// Create options with travel lines enabled
    pub fn with_travel(mut self) -> Self {
        self.show_travel = true;
        self
    }

    /// Create options with direction arrows enabled
    pub fn with_direction(mut self) -> Self {
        self.show_direction = true;
        self
    }

    /// Set the stroke width
    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }
}

/// Record optimized strokes to SVG
///
/// This creates an SVG showing exactly what would be plotted, including:
/// - Stroke order optimization (strokes are rendered in plot order)
/// - Stroke reversal (strokes are drawn in the direction they would be plotted)
/// - Optional travel lines showing pen-up movements
/// - Optional direction arrows showing stroke direction
///
/// Unlike `drawing_to_svg_string`, this takes pre-optimized strokes rather than
/// a Drawing, so it shows the actual plot output.
pub fn record_strokes_to_svg(
    strokes: &[OwnedOptimizedStroke],
    width: f64,
    height: f64,
    options: &RecordOptions,
) -> String {
    let mut svg = String::new();

    // SVG header
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" 
     width="{width}mm" 
     height="{height}mm" 
     viewBox="0 0 {width} {height}">
"#,
        width = width,
        height = height
    ));

    // Add a group for travel lines (rendered first, so they're behind strokes)
    if options.show_travel {
        svg.push_str("  <g id=\"travel\" opacity=\"0.5\">\n");
        let mut current_pos = drawing_core::Point::ZERO;

        for stroke in strokes {
            if stroke.is_empty() {
                continue;
            }

            let start = stroke.start();

            // Draw travel line from current position to stroke start
            if current_pos.distance(start) > 0.01 {
                svg.push_str(&travel_line_svg(current_pos, start, options.travel_color));
            }

            current_pos = stroke.end();
        }
        svg.push_str("  </g>\n");
    }

    // Add a group for strokes
    svg.push_str("  <g id=\"strokes\">\n");

    let mut current_pos = drawing_core::Point::ZERO;

    for stroke in strokes {
        if stroke.is_empty() {
            continue;
        }

        // Draw direction arrow at stroke start
        if options.show_direction {
            let start = stroke.start();
            let points: Vec<_> = stroke.points_iter().collect();
            if points.len() >= 2 {
                let direction = points[1] - points[0];
                svg.push_str(&arrow_svg(
                    start,
                    direction,
                    options.arrow_size,
                    stroke.style.stroke_color,
                ));
            }
        }

        // Draw the stroke
        svg.push_str(&optimized_stroke_to_svg(stroke, options.stroke_width));

        current_pos = stroke.end();
    }

    // Suppress unused variable warning when show_direction is false
    let _ = current_pos;

    svg.push_str("  </g>\n");
    svg.push_str("</svg>\n");
    svg
}

/// Convert an optimized stroke to SVG path element
fn optimized_stroke_to_svg(stroke: &OwnedOptimizedStroke, stroke_width: f64) -> String {
    let points: Vec<_> = stroke.points_iter().collect();

    if points.len() < 2 {
        return String::new();
    }

    let d = points_to_svg_path_data(&points, stroke.closed);

    format!(
        r#"    <path d="{}" fill="none" stroke="{}" stroke-width="{:.3}" stroke-linecap="round" stroke-linejoin="round"/>
"#,
        d,
        color_to_hex(stroke.style.stroke_color),
        stroke_width
    )
}

/// Generate SVG for a dashed travel line
fn travel_line_svg(from: drawing_core::Point, to: drawing_core::Point, color: Color) -> String {
    format!(
        r#"    <line x1="{:.3}" y1="{:.3}" x2="{:.3}" y2="{:.3}" stroke="{}" stroke-width="0.1" stroke-dasharray="1,1"/>
"#,
        from.x,
        from.y,
        to.x,
        to.y,
        color_to_hex(color)
    )
}

/// Generate SVG for a direction arrow at the start of a stroke
fn arrow_svg(
    position: drawing_core::Point,
    direction: drawing_core::Vec2,
    size: f64,
    color: Color,
) -> String {
    let len = direction.hypot();
    if len < 0.001 {
        return String::new();
    }

    // Normalize direction
    let dir_x = direction.x / len;
    let dir_y = direction.y / len;

    // Perpendicular direction
    let perp_x = -dir_y;
    let perp_y = dir_x;

    // Arrow tip is at position, base is behind
    let tip = position;
    let base_center_x = position.x - dir_x * size;
    let base_center_y = position.y - dir_y * size;

    let half_width = size * 0.4;
    let base_left_x = base_center_x + perp_x * half_width;
    let base_left_y = base_center_y + perp_y * half_width;
    let base_right_x = base_center_x - perp_x * half_width;
    let base_right_y = base_center_y - perp_y * half_width;

    format!(
        r#"    <polygon points="{:.3},{:.3} {:.3},{:.3} {:.3},{:.3}" fill="{}" opacity="0.7"/>
"#,
        tip.x,
        tip.y,
        base_left_x,
        base_left_y,
        base_right_x,
        base_right_y,
        color_to_hex(color)
    )
}

/// Export recorded strokes to a file
pub fn export_recorded_svg(
    strokes: &[OwnedOptimizedStroke],
    width: f64,
    height: f64,
    path: impl AsRef<std::path::Path>,
    options: &RecordOptions,
) -> Result<(), SvgError> {
    let svg = record_strokes_to_svg(strokes, width, height, options);
    std::fs::write(path, svg)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_core::{Element, FontRegistry, Point, ResolvedStyle};
    use std::sync::Arc;

    fn test_ctx() -> RenderContext {
        RenderContext::new(Arc::new(FontRegistry::new()))
    }

    // ========================================================================
    // color_to_hex tests
    // ========================================================================

    #[test]
    fn test_color_to_hex_rgb() {
        let color = Color::rgb(255, 128, 0);
        assert_eq!(color_to_hex(color), "#ff8000");
    }

    #[test]
    fn test_color_to_hex_black() {
        assert_eq!(color_to_hex(Color::BLACK), "#000000");
    }

    #[test]
    fn test_color_to_hex_white() {
        assert_eq!(color_to_hex(Color::WHITE), "#ffffff");
    }

    #[test]
    fn test_color_to_hex_with_alpha() {
        let color = Color::rgba(255, 0, 0, 128);
        let result = color_to_hex(color);
        assert!(result.starts_with("rgba("));
        assert!(result.contains("255,0,0,"));
        // Alpha should be approximately 0.502
        assert!(result.contains("0.5"));
    }

    #[test]
    fn test_color_to_hex_transparent() {
        let color = Color::rgba(100, 150, 200, 0);
        let result = color_to_hex(color);
        assert!(result.starts_with("rgba("));
        assert!(result.contains(",0.000)"));
    }

    #[test]
    fn test_color_to_hex_full_alpha_uses_hex() {
        // Full alpha (255) should use hex format, not rgba
        let color = Color::rgba(128, 64, 32, 255);
        let result = color_to_hex(color);
        assert!(result.starts_with("#"));
        assert_eq!(result, "#804020");
    }

    // ========================================================================
    // stroke_to_svg tests
    // ========================================================================

    #[test]
    fn test_stroke_to_svg_simple_line() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(100.0, 50.0)],
            ResolvedStyle::default(),
        );
        let svg = stroke_to_svg(&stroke);

        assert!(svg.contains("M0.000,0.000"));
        assert!(svg.contains("L100.000,50.000"));
        assert!(svg.contains("fill=\"none\""));
        assert!(svg.contains("stroke="));
    }

    #[test]
    fn test_stroke_to_svg_closed_path() {
        let stroke = Stroke::new(
            vec![
                Point::new(0.0, 0.0),
                Point::new(100.0, 0.0),
                Point::new(50.0, 100.0),
            ],
            ResolvedStyle::default(),
        )
        .closed();
        let svg = stroke_to_svg(&stroke);

        assert!(svg.contains(" Z"));
    }

    #[test]
    fn test_stroke_to_svg_open_path() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
            ResolvedStyle::default(),
        );
        let svg = stroke_to_svg(&stroke);

        assert!(!svg.contains(" Z"));
    }

    #[test]
    fn test_stroke_to_svg_stroke_width() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)],
            ResolvedStyle::new(2.5, Color::BLACK),
        );
        let svg = stroke_to_svg(&stroke);

        assert!(svg.contains("stroke-width=\"2.500\""));
    }

    #[test]
    fn test_stroke_to_svg_stroke_color() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)],
            ResolvedStyle::new(1.0, Color::RED),
        );
        let svg = stroke_to_svg(&stroke);

        assert!(svg.contains("stroke=\"#ff0000\""));
    }

    #[test]
    fn test_stroke_to_svg_negative_coordinates() {
        let stroke = Stroke::new(
            vec![Point::new(-10.0, -20.0), Point::new(10.0, 20.0)],
            ResolvedStyle::default(),
        );
        let svg = stroke_to_svg(&stroke);

        assert!(svg.contains("M-10.000,-20.000"));
        assert!(svg.contains("L10.000,20.000"));
    }

    // ========================================================================
    // drawing_to_svg_string tests
    // ========================================================================

    #[test]
    fn test_basic_export() {
        let ctx = test_ctx();
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::line(
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
        ));

        let svg = drawing_to_svg_string(&drawing, &ctx);
        assert!(svg.contains("svg"));
        assert!(svg.contains("path"));
        assert!(svg.contains("M0.000,0.000"));
    }

    #[test]
    fn test_drawing_dimensions() {
        let ctx = test_ctx();
        let drawing = Drawing::new(297.0, 210.0);
        let svg = drawing_to_svg_string(&drawing, &ctx);

        assert!(svg.contains("width=\"297mm\""));
        assert!(svg.contains("height=\"210mm\""));
        assert!(svg.contains("viewBox=\"0 0 297 210\""));
    }

    #[test]
    fn test_drawing_empty() {
        let ctx = test_ctx();
        let drawing = Drawing::new(100.0, 100.0);
        let svg = drawing_to_svg_string(&drawing, &ctx);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // Should not contain any path elements
        assert!(!svg.contains("<path"));
    }

    #[test]
    fn test_drawing_white_background_omitted() {
        let ctx = test_ctx();
        let drawing = Drawing::new(100.0, 100.0).with_background(Color::WHITE);
        let svg = drawing_to_svg_string(&drawing, &ctx);

        // White background should not create a rect element
        assert!(!svg.contains("<rect"));
    }

    #[test]
    fn test_drawing_non_white_background() {
        let ctx = test_ctx();
        let drawing = Drawing::new(100.0, 100.0).with_background(Color::BLACK);
        let svg = drawing_to_svg_string(&drawing, &ctx);

        assert!(svg.contains("<rect"));
        assert!(svg.contains("fill=\"#000000\""));
    }

    #[test]
    fn test_drawing_colored_background() {
        let ctx = test_ctx();
        let drawing = Drawing::new(100.0, 100.0).with_background(Color::rgb(200, 220, 240));
        let svg = drawing_to_svg_string(&drawing, &ctx);

        assert!(svg.contains("<rect"));
        assert!(svg.contains("fill=\"#c8dcf0\""));
    }

    #[test]
    fn test_drawing_multiple_elements() {
        let ctx = test_ctx();
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::line(Point::new(0.0, 0.0), Point::new(50.0, 50.0)));
        drawing.add(Element::line(
            Point::new(50.0, 50.0),
            Point::new(100.0, 0.0),
        ));

        let svg = drawing_to_svg_string(&drawing, &ctx);

        // Should contain two path elements
        let path_count = svg.matches("<path").count();
        assert_eq!(path_count, 2);
    }

    #[test]
    fn test_drawing_circle() {
        let ctx = test_ctx();
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::circle(Point::new(50.0, 50.0), 25.0));

        let svg = drawing_to_svg_string(&drawing, &ctx);

        // Circle is flattened to a path with many points
        assert!(svg.contains("<path"));
        // Path should start with M (move to) and contain multiple L (line to)
        assert!(svg.contains(" L"));
        // Circle path should be closed
        assert!(svg.contains(" Z"));
    }

    #[test]
    fn test_drawing_xml_header() {
        let ctx = test_ctx();
        let drawing = Drawing::new(100.0, 100.0);
        let svg = drawing_to_svg_string(&drawing, &ctx);

        assert!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_drawing_xmlns() {
        let ctx = test_ctx();
        let drawing = Drawing::new(100.0, 100.0);
        let svg = drawing_to_svg_string(&drawing, &ctx);

        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    // ========================================================================
    // Integration tests
    // ========================================================================

    #[test]
    fn test_write_svg_to_buffer() {
        let ctx = test_ctx();
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::line(
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
        ));

        let mut buffer = Vec::new();
        write_svg(&drawing, &mut buffer, &ctx).unwrap();

        let svg = String::from_utf8(buffer).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path"));
    }

    // ========================================================================
    // Record strokes tests
    // ========================================================================

    fn create_test_stroke(points: Vec<Point>, reversed: bool) -> OwnedOptimizedStroke {
        OwnedOptimizedStroke {
            points,
            style: ResolvedStyle::default(),
            closed: false,
            reversed,
        }
    }

    #[test]
    fn test_record_strokes_empty() {
        let strokes: Vec<OwnedOptimizedStroke> = vec![];
        let svg = record_strokes_to_svg(&strokes, 100.0, 100.0, &RecordOptions::default());

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("width=\"100mm\""));
        assert!(svg.contains("height=\"100mm\""));
    }

    #[test]
    fn test_record_strokes_single_stroke() {
        let strokes = vec![create_test_stroke(
            vec![Point::new(10.0, 10.0), Point::new(50.0, 50.0)],
            false,
        )];
        let svg = record_strokes_to_svg(&strokes, 100.0, 100.0, &RecordOptions::default());

        assert!(svg.contains("<path"));
        assert!(svg.contains("M10.000,10.000"));
        assert!(svg.contains("L50.000,50.000"));
    }

    #[test]
    fn test_record_strokes_reversed() {
        let strokes = vec![create_test_stroke(
            vec![Point::new(10.0, 10.0), Point::new(50.0, 50.0)],
            true, // reversed
        )];
        let svg = record_strokes_to_svg(&strokes, 100.0, 100.0, &RecordOptions::default());

        // Reversed stroke should start at 50,50 and go to 10,10
        assert!(svg.contains("M50.000,50.000"));
        assert!(svg.contains("L10.000,10.000"));
    }

    #[test]
    fn test_record_strokes_with_travel() {
        let strokes = vec![
            create_test_stroke(vec![Point::new(10.0, 10.0), Point::new(20.0, 20.0)], false),
            create_test_stroke(vec![Point::new(80.0, 80.0), Point::new(90.0, 90.0)], false),
        ];
        let options = RecordOptions::default().with_travel();
        let svg = record_strokes_to_svg(&strokes, 100.0, 100.0, &options);

        // Should contain travel group
        assert!(svg.contains("<g id=\"travel\""));
        // Should contain travel line (dashed)
        assert!(svg.contains("<line"));
        assert!(svg.contains("stroke-dasharray"));
    }

    #[test]
    fn test_record_strokes_without_travel() {
        let strokes = vec![
            create_test_stroke(vec![Point::new(10.0, 10.0), Point::new(20.0, 20.0)], false),
            create_test_stroke(vec![Point::new(80.0, 80.0), Point::new(90.0, 90.0)], false),
        ];
        let svg = record_strokes_to_svg(&strokes, 100.0, 100.0, &RecordOptions::default());

        // Should NOT contain travel group
        assert!(!svg.contains("<g id=\"travel\""));
        assert!(!svg.contains("<line"));
    }

    #[test]
    fn test_record_strokes_with_direction() {
        let strokes = vec![create_test_stroke(
            vec![Point::new(10.0, 10.0), Point::new(50.0, 50.0)],
            false,
        )];
        let options = RecordOptions::default().with_direction();
        let svg = record_strokes_to_svg(&strokes, 100.0, 100.0, &options);

        // Should contain direction arrow (polygon)
        assert!(svg.contains("<polygon"));
    }

    #[test]
    fn test_record_strokes_stroke_width() {
        let strokes = vec![create_test_stroke(
            vec![Point::new(10.0, 10.0), Point::new(50.0, 50.0)],
            false,
        )];
        let options = RecordOptions::default().with_stroke_width(0.5);
        let svg = record_strokes_to_svg(&strokes, 100.0, 100.0, &options);

        assert!(svg.contains("stroke-width=\"0.500\""));
    }

    #[test]
    fn test_record_strokes_preserves_color() {
        let mut stroke =
            create_test_stroke(vec![Point::new(10.0, 10.0), Point::new(50.0, 50.0)], false);
        stroke.style = ResolvedStyle::new(1.0, Color::RED);

        let strokes = vec![stroke];
        let svg = record_strokes_to_svg(&strokes, 100.0, 100.0, &RecordOptions::default());

        assert!(svg.contains("stroke=\"#ff0000\""));
    }

    #[test]
    fn test_record_options_default() {
        let options = RecordOptions::default();
        assert!(!options.show_travel);
        assert!(!options.show_direction);
        assert!((options.stroke_width - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_record_options_builder() {
        let options = RecordOptions::default()
            .with_travel()
            .with_direction()
            .with_stroke_width(0.5);

        assert!(options.show_travel);
        assert!(options.show_direction);
        assert!((options.stroke_width - 0.5).abs() < 0.001);
    }
}
