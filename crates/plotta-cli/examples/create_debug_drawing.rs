//! Create a small debug drawing for testing pen up/down behavior
//!
//! Creates a 100x100mm drawing with simple shapes to verify
//! pen transitions work correctly.
//!
//! Run with: cargo run -p plotta-cli --example create_debug_drawing

use drawing_core::{Drawing, Element, Point};

fn main() {
    // Create a small 100x100mm drawing
    let mut drawing = Drawing::new(100.0, 100.0);

    // Border rectangle
    drawing.add(Element::rect(5.0, 5.0, 90.0, 90.0).stroke_width(0.5));

    // Four circles in corners - tests pen up travel between shapes
    let radius = 10.0;
    let margin = 20.0;

    // Top-left circle
    drawing.add(Element::circle(Point::new(margin, margin), radius).stroke_width(0.5));

    // Top-right circle
    drawing.add(Element::circle(Point::new(100.0 - margin, margin), radius).stroke_width(0.5));

    // Bottom-right circle
    drawing
        .add(Element::circle(Point::new(100.0 - margin, 100.0 - margin), radius).stroke_width(0.5));

    // Bottom-left circle
    drawing.add(Element::circle(Point::new(margin, 100.0 - margin), radius).stroke_width(0.5));

    // Center cross - two lines crossing
    let center = drawing.center();
    let cross_size = 15.0;

    // Horizontal line
    drawing.add(
        Element::line(
            Point::new(center.x - cross_size, center.y),
            Point::new(center.x + cross_size, center.y),
        )
        .stroke_width(0.5),
    );

    // Vertical line
    drawing.add(
        Element::line(
            Point::new(center.x, center.y - cross_size),
            Point::new(center.x, center.y + cross_size),
        )
        .stroke_width(0.5),
    );

    // Small rectangle in center
    drawing.add(Element::rect(center.x - 8.0, center.y - 8.0, 16.0, 16.0).stroke_width(0.5));

    // Save the drawing
    let path = "debug-drawing.json";
    drawing.save(path).expect("Failed to save drawing");

    println!("Created: {}", path);
    println!("  Size: {} x {} mm", drawing.width, drawing.height);
    println!("  Elements: {}", drawing.elements.len());
    println!();
    println!("Preview: cargo run -p plotta-cli -- preview {}", path);
    println!("Plot:    cargo run -p plotta-cli -- plot {}", path);
}
