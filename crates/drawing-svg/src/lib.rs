//! SVG import/export for plotta-studio
//!
//! Note: Import is lossy - only path/line data is preserved.
//! Complex SVG features (gradients, filters, text, etc.) are ignored.

use drawing_core::{Color, Drawing, Stroke, Style};
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
    d.push_str(&format!("M{:.3},{:.3}", stroke.points[0].x, stroke.points[0].y));

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
    use drawing_core::{Element, Point};

    #[test]
    fn test_basic_export() {
        let mut drawing = Drawing::new(100.0, 100.0);
        drawing.add(Element::line(Point::new(0.0, 0.0), Point::new(100.0, 100.0)));

        let svg = drawing_to_svg_string(&drawing);
        assert!(svg.contains("svg"));
        assert!(svg.contains("path"));
        assert!(svg.contains("M0.000,0.000"));
    }
}
