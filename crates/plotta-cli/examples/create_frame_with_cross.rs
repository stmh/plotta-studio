//! Create a 10x10cm drawing with a frame, signature, and rotated cross
//!
//! This example generates a test drawing for verifying plotter setup and alignment.
//!
//! Run with: cargo run -p plotta-cli --example create_frame_with_cross

use std::f64::consts::PI;

use drawing_core::{Drawing, Element, Group, Point};
use drawing_text::FontManager;
use drawing_utils::{draw_frame_with_title, FrameOptions, PlaceholderSignature};

fn main() {
    // Create a 100x100mm (10x10cm) drawing
    let mut drawing = Drawing::new(100.0, 100.0);

    // Load font for the title
    let manager = FontManager::new();
    manager
        .load_relief_single_line()
        .expect("Failed to load font");

    let font = manager
        .default_font()
        .expect("Default font should be loaded");

    // Create frame options with signature enabled
    let frame_options = FrameOptions::new(font)
        .margin(5.0)
        .margin_bottom(10.0)
        .stroke_width(0.4)
        .with_signature(PlaceholderSignature)
        .signature_height(5.0);

    // Add frame with title
    let frame = draw_frame_with_title(&drawing, "Test Pattern", &frame_options);
    drawing.add(frame);

    // Create a cross rotated by 45 degrees in the center
    let cross = create_rotated_cross(drawing.center(), 20.0, PI / 4.0, 0.4);
    drawing.add(cross);

    // Save the drawing
    let path = "frame-with-cross.json";
    drawing.save(path).expect("Failed to save drawing");

    println!("Created: {}", path);
    println!("  Size: {} x {} mm", drawing.width, drawing.height);
    println!("  Elements: {}", drawing.elements.len());
    println!();
    println!("Preview with: cargo run -p plotta-cli -- preview {}", path);
    println!("Plot with:    cargo run -p plotta-cli -- plot {}", path);
}

/// Create a cross (plus sign) rotated around its center
fn create_rotated_cross(
    center: Point,
    arm_length: f64,
    rotation: f64,
    stroke_width: f64,
) -> Element {
    let mut group = Group::new();

    // Calculate the endpoints of the two lines that form the cross
    // Before rotation, the cross has horizontal and vertical arms
    let half_arm = arm_length / 2.0;

    // First line (originally horizontal)
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();

    // Rotate the horizontal endpoints
    let h1 = Point::new(center.x + half_arm * cos_r, center.y + half_arm * sin_r);
    let h2 = Point::new(center.x - half_arm * cos_r, center.y - half_arm * sin_r);

    // Rotate the vertical endpoints (90 degrees from horizontal)
    let v1 = Point::new(center.x + half_arm * (-sin_r), center.y + half_arm * cos_r);
    let v2 = Point::new(center.x - half_arm * (-sin_r), center.y - half_arm * cos_r);

    group.push(Element::line(h1, h2).stroke_width(stroke_width));
    group.push(Element::line(v1, v2).stroke_width(stroke_width));

    Element::group(group)
}
