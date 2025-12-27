//! Tests for SVG import functionality

use super::*;

#[test]
fn test_import_simple_line() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <line x1="0" y1="0" x2="100" y2="100" stroke="black"/>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert_eq!(result.drawing.width, 100.0);
    assert_eq!(result.drawing.height, 100.0);
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_circle() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <circle cx="50" cy="50" r="25" stroke="red" fill="none"/>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_path() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <path d="M10,10 L90,10 L90,90 L10,90 Z" stroke="black" fill="none"/>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_with_transform() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <g transform="translate(10, 10)">
            <rect x="0" y="0" width="20" height="20" stroke="black" fill="none"/>
        </g>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_filled_shape_convert_to_outline() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect x="10" y="10" width="80" height="80" fill="blue"/>
    </svg>"#;

    let options = ImportOptions {
        fill_behavior: FillBehavior::ConvertToOutline,
        ..Default::default()
    };

    let result = import_svg_string_with_options(svg, &options).unwrap();
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_filled_shape_ignore() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect x="10" y="10" width="80" height="80" fill="blue"/>
    </svg>"#;

    let options = ImportOptions {
        fill_behavior: FillBehavior::Ignore,
        ..Default::default()
    };

    let result = import_svg_string_with_options(svg, &options).unwrap();
    // The filled rect should be ignored
    assert!(result.drawing.elements.is_empty());
}

#[test]
fn test_import_stroke_color() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <line x1="0" y1="0" x2="100" y2="100" stroke="#ff0000" stroke-width="2"/>
    </svg>"##;

    let result = import_svg_string(svg).unwrap();
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_empty_svg() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert_eq!(result.drawing.width, 100.0);
    assert_eq!(result.drawing.height, 100.0);
    assert!(result.drawing.elements.is_empty());
}

#[test]
fn test_import_nested_groups() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <g transform="translate(10, 10)">
            <g transform="rotate(45)">
                <line x1="0" y1="0" x2="50" y2="0" stroke="black"/>
            </g>
        </g>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_multiple_shapes() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect x="10" y="10" width="30" height="30" stroke="black" fill="none"/>
        <circle cx="70" cy="30" r="15" stroke="blue" fill="none"/>
        <ellipse cx="50" cy="70" rx="30" ry="15" stroke="green" fill="none"/>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert!(!result.drawing.elements.is_empty());
    assert!(result.warnings.is_empty());
}

#[test]
fn test_import_bezier_curves() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <path d="M10,10 Q50,0 90,10" stroke="black" fill="none"/>
        <path d="M10,50 C30,30 70,70 90,50" stroke="black" fill="none"/>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_polyline() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <polyline points="10,10 50,50 90,10" stroke="black" fill="none"/>
        <polygon points="10,60 50,90 90,60" stroke="blue" fill="none"/>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    assert!(!result.drawing.elements.is_empty());
}

#[test]
fn test_import_ellipses() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <ellipse cx="100" cy="60" rx="20" ry="40" stroke="black" fill="none"/>
        <ellipse cx="140" cy="100" rx="40" ry="20" stroke="black" fill="none"/>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    // Should have 2 ellipses converted to paths
    assert_eq!(result.drawing.elements.len(), 2, "Expected 2 ellipses");
}

#[test]
fn test_import_group_with_transform() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <ellipse cx="100" cy="50" rx="20" ry="30" stroke="black" fill="none"/>
        <g transform="rotate(45, 100, 100)">
            <ellipse cx="100" cy="50" rx="20" ry="30" stroke="black" fill="none"/>
        </g>
    </svg>"#;

    let result = import_svg_string(svg).unwrap();
    // Should have 1 ellipse + 1 group containing 1 ellipse = 2 elements at root
    assert_eq!(result.drawing.elements.len(), 2, "Expected ellipse + group");
}

#[test]
fn test_import_flower_svg() {
    use drawing_core::{FontRegistry, RenderContext};
    use std::sync::Arc;

    // The flower has: 4 outer ellipses + 1 group with 4 inner ellipses + 2 circles
    let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
  <ellipse cx="100" cy="60" rx="20" ry="40" stroke="black" fill="none" stroke-width="1.5"/>
  <ellipse cx="140" cy="100" rx="40" ry="20" stroke="black" fill="none" stroke-width="1.5"/>
  <ellipse cx="100" cy="140" rx="20" ry="40" stroke="black" fill="none" stroke-width="1.5"/>
  <ellipse cx="60" cy="100" rx="40" ry="20" stroke="black" fill="none" stroke-width="1.5"/>
  <g transform="rotate(45, 100, 100)">
    <ellipse cx="100" cy="60" rx="15" ry="35" stroke="black" fill="none" stroke-width="1"/>
    <ellipse cx="140" cy="100" rx="35" ry="15" stroke="black" fill="none" stroke-width="1"/>
    <ellipse cx="100" cy="140" rx="15" ry="35" stroke="black" fill="none" stroke-width="1"/>
    <ellipse cx="60" cy="100" rx="35" ry="15" stroke="black" fill="none" stroke-width="1"/>
  </g>
  <circle cx="100" cy="100" r="15" stroke="black" fill="none" stroke-width="2"/>
  <circle cx="100" cy="100" r="8" stroke="black" fill="none" stroke-width="1.5"/>
</svg>"#;

    let result = import_svg_string(svg).unwrap();

    // 4 ellipses + 1 group + 2 circles = 7 top-level elements
    assert_eq!(result.drawing.elements.len(), 7);
    assert!(result.warnings.is_empty());

    // Flatten and count strokes: 8 ellipses + 2 circles = 10 strokes
    let ctx = RenderContext::new(Arc::new(FontRegistry::new()));
    let strokes = result.drawing.flatten(&ctx);
    assert_eq!(strokes.len(), 10);
}

#[test]
fn test_import_clip_path() {
    use drawing_core::{FontRegistry, RenderContext};
    use std::sync::Arc;

    // Use a line that clearly crosses through the circle interior
    // Circle: center (100, 100), radius 50 -> extends from 50 to 150 in both x and y
    // Line: from (0, 100) to (200, 100) - horizontal line through center
    // This should be clipped to approximately (50, 100) to (150, 100)
    let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
  <defs>
    <clipPath id="circleClip">
      <circle cx="100" cy="100" r="50"/>
    </clipPath>
  </defs>
  <g clip-path="url(#circleClip)">
    <line x1="0" y1="100" x2="200" y2="100" stroke="black" stroke-width="2"/>
  </g>
</svg>"#;

    let result = import_svg_string(svg).unwrap();

    assert_eq!(result.drawing.elements.len(), 1);
    assert!(result.warnings.is_empty());

    let ctx = RenderContext::new(Arc::new(FontRegistry::new()));
    let strokes = result.drawing.flatten(&ctx);

    // Should have clipped line inside the circle
    assert!(!strokes.is_empty(), "Expected clipped strokes");

    // Verify all stroke points are inside or near the clip circle
    for stroke in &strokes {
        for p in &stroke.points {
            let dist = ((p.x - 100.0).powi(2) + (p.y - 100.0).powi(2)).sqrt();
            assert!(
                dist <= 51.0, // Allow small tolerance for bezier approximation
                "Point ({}, {}) is outside clip circle (dist={})",
                p.x,
                p.y,
                dist
            );
        }
    }
}

#[test]
fn test_import_clipped_svg_diagonal_lines() {
    use drawing_core::{FontRegistry, RenderContext};
    use std::sync::Arc;

    // Test specifically for the diagonal lines clipped by circle - same as 04-clipped.svg
    let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
  <defs>
    <clipPath id="circleClip">
      <circle cx="100" cy="100" r="70"/>
    </clipPath>
  </defs>
  <g clip-path="url(#circleClip)">
    <line x1="0" y1="0" x2="200" y2="200" stroke="blue" stroke-width="1.5"/>
    <line x1="0" y1="40" x2="200" y2="240" stroke="blue" stroke-width="1.5"/>
    <line x1="0" y1="80" x2="200" y2="280" stroke="blue" stroke-width="1.5"/>
  </g>
</svg>"#;

    let result = import_svg_string(svg).unwrap();

    assert_eq!(result.drawing.elements.len(), 1); // One ClipGroup
    assert!(result.warnings.is_empty());

    let ctx = RenderContext::new(Arc::new(FontRegistry::new()));
    let strokes = result.drawing.flatten(&ctx);

    eprintln!("Number of strokes: {}", strokes.len());
    for (i, stroke) in strokes.iter().enumerate() {
        eprintln!(
            "Stroke {}: {} points, first=({:.1},{:.1}), last=({:.1},{:.1})",
            i,
            stroke.points.len(),
            stroke.points.first().map(|p| p.x).unwrap_or(0.0),
            stroke.points.first().map(|p| p.y).unwrap_or(0.0),
            stroke.points.last().map(|p| p.x).unwrap_or(0.0),
            stroke.points.last().map(|p| p.y).unwrap_or(0.0),
        );
    }

    // Should have at least 3 clipped line segments (one for each input line)
    // Some lines might be split into multiple segments
    assert!(
        strokes.len() >= 3,
        "Expected at least 3 strokes for 3 diagonal lines, got {}",
        strokes.len()
    );

    // The main diagonal (0,0)-(200,200) should definitely produce a stroke
    // through the circle at (100,100) r=70
    // Entry point: approximately (50.5, 50.5)
    // Exit point: approximately (149.5, 149.5)
}
