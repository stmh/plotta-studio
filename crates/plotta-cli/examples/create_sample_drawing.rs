//! Create a sample drawing JSON file for testing the CLI
//!
//! Run with: cargo run -p plotta-cli --example create_sample_drawing

use drawing_core::{Color, Drawing, Element, Point};
use std::f64::consts::TAU;

fn main() {
    let mut drawing = Drawing::a4_landscape();

    // Add a rectangle border
    drawing.add(Element::rect(10.0, 10.0, 277.0, 190.0).stroke_width(1.0));

    // Add concentric circles
    let center = drawing.center();
    for i in 1..=5 {
        let t = i as f64 / 5.0;
        drawing.add(
            Element::circle(center, 20.0 + t * 50.0)
                .stroke_width(0.5)
                .stroke_color(Color::gray(100)),
        );
    }

    // Add radial lines
    for i in 0..12 {
        let angle = (i as f64 / 12.0) * TAU;
        let r = 70.0;
        let from = Point::new(center.x + angle.cos() * 20.0, center.y + angle.sin() * 20.0);
        let to = Point::new(center.x + angle.cos() * r, center.y + angle.sin() * r);
        drawing.add(Element::line(from, to).stroke_width(0.3));
    }

    // Save the drawing
    let path = "sample-drawing.json";
    drawing.save(path).expect("Failed to save drawing");
    println!("Created: {}", path);
    println!("  Size: {} x {} mm", drawing.width, drawing.height);
    println!("  Elements: {}", drawing.elements.len());
    println!();
    println!("Test with: cargo run -p plotta-cli -- preview {}", path);
}
