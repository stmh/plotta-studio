//! SVG import/export for plotta-studio
//!
//! Note: Import is lossy - only path/line data is preserved.
//! Complex SVG features (gradients, filters, text, etc.) are ignored.

use drawing_core::{Color, Drawing, Stroke};
use std::io::Write;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SvgError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Export a drawing to SVG format
pub fn export_svg(drawing: &Drawing, path: impl AsRef<Path>) -> Result<(), SvgError> {
    let svg = drawing_to_svg_string(drawing);
    std::fs::write(path, svg)?;
    Ok(())
}

/// Convert a drawing to an SVG string
pub fn drawing_to_svg_string(drawing: &Drawing) -> String {
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
    let strokes = drawing.flatten();
    for stroke in strokes {
        if stroke.points.len() < 2 {
            continue;
        }

        svg.push_str(&stroke_to_svg(&stroke));
    }

    svg.push_str("</svg>\n");
    svg
}

fn stroke_to_svg(stroke: &Stroke) -> String {
    let mut d = String::new();

    // Move to first point
    d.push_str(&format!(
        "M{:.3},{:.3}",
        stroke.points[0].x, stroke.points[0].y
    ));

    // Line to remaining points
    for pt in &stroke.points[1..] {
        d.push_str(&format!(" L{:.3},{:.3}", pt.x, pt.y));
    }

    if stroke.closed {
        d.push_str(" Z");
    }

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
pub fn write_svg<W: Write>(drawing: &Drawing, writer: &mut W) -> Result<(), SvgError> {
    let svg = drawing_to_svg_string(drawing);
    writer.write_all(svg.as_bytes())?;
    Ok(())
}

// TODO: SVG import
// This would require parsing SVG paths and converting to our primitives.
// Consider using `usvg` for robust parsing.

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_core::{Element, Point, Style};

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
            Style::default(),
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
            Style::default(),
        )
        .closed();
        let svg = stroke_to_svg(&stroke);

        assert!(svg.contains(" Z"));
    }

    #[test]
    fn test_stroke_to_svg_open_path() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
            Style::default(),
        );
        let svg = stroke_to_svg(&stroke);

        assert!(!svg.contains(" Z"));
    }

    #[test]
    fn test_stroke_to_svg_stroke_width() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)],
            Style::new(2.5, Color::BLACK),
        );
        let svg = stroke_to_svg(&stroke);

        assert!(svg.contains("stroke-width=\"2.500\""));
    }

    #[test]
    fn test_stroke_to_svg_stroke_color() {
        let stroke = Stroke::new(
            vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)],
            Style::new(1.0, Color::RED),
        );
        let svg = stroke_to_svg(&stroke);

        assert!(svg.contains("stroke=\"#ff0000\""));
    }

    #[test]
    fn test_stroke_to_svg_negative_coordinates() {
        let stroke = Stroke::new(
            vec![Point::new(-10.0, -20.0), Point::new(10.0, 20.0)],
            Style::default(),
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
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::line(
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
        ));

        let svg = drawing_to_svg_string(&drawing);
        assert!(svg.contains("svg"));
        assert!(svg.contains("path"));
        assert!(svg.contains("M0.000,0.000"));
    }

    #[test]
    fn test_drawing_dimensions() {
        let drawing = Drawing::new(297.0, 210.0);
        let svg = drawing_to_svg_string(&drawing);

        assert!(svg.contains("width=\"297mm\""));
        assert!(svg.contains("height=\"210mm\""));
        assert!(svg.contains("viewBox=\"0 0 297 210\""));
    }

    #[test]
    fn test_drawing_empty() {
        let drawing = Drawing::new(100.0, 100.0);
        let svg = drawing_to_svg_string(&drawing);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // Should not contain any path elements
        assert!(!svg.contains("<path"));
    }

    #[test]
    fn test_drawing_white_background_omitted() {
        let drawing = Drawing::new(100.0, 100.0).with_background(Color::WHITE);
        let svg = drawing_to_svg_string(&drawing);

        // White background should not create a rect element
        assert!(!svg.contains("<rect"));
    }

    #[test]
    fn test_drawing_non_white_background() {
        let drawing = Drawing::new(100.0, 100.0).with_background(Color::BLACK);
        let svg = drawing_to_svg_string(&drawing);

        assert!(svg.contains("<rect"));
        assert!(svg.contains("fill=\"#000000\""));
    }

    #[test]
    fn test_drawing_colored_background() {
        let drawing = Drawing::new(100.0, 100.0).with_background(Color::rgb(200, 220, 240));
        let svg = drawing_to_svg_string(&drawing);

        assert!(svg.contains("<rect"));
        assert!(svg.contains("fill=\"#c8dcf0\""));
    }

    #[test]
    fn test_drawing_multiple_elements() {
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::line(Point::new(0.0, 0.0), Point::new(50.0, 50.0)));
        drawing.add(Element::line(
            Point::new(50.0, 50.0),
            Point::new(100.0, 0.0),
        ));

        let svg = drawing_to_svg_string(&drawing);

        // Should contain two path elements
        let path_count = svg.matches("<path").count();
        assert_eq!(path_count, 2);
    }

    #[test]
    fn test_drawing_circle() {
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::circle(Point::new(50.0, 50.0), 25.0));

        let svg = drawing_to_svg_string(&drawing);

        // Circle is flattened to a path with many points
        assert!(svg.contains("<path"));
        // Path should start with M (move to) and contain multiple L (line to)
        assert!(svg.contains(" L"));
        // Circle path should be closed
        assert!(svg.contains(" Z"));
    }

    #[test]
    fn test_drawing_xml_header() {
        let drawing = Drawing::new(100.0, 100.0);
        let svg = drawing_to_svg_string(&drawing);

        assert!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_drawing_xmlns() {
        let drawing = Drawing::new(100.0, 100.0);
        let svg = drawing_to_svg_string(&drawing);

        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    // ========================================================================
    // Integration tests
    // ========================================================================

    #[test]
    fn test_write_svg_to_buffer() {
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::line(
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
        ));

        let mut buffer = Vec::new();
        write_svg(&drawing, &mut buffer).unwrap();

        let svg = String::from_utf8(buffer).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path"));
    }
}
