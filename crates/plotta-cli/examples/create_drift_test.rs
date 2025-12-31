//! Create drift test drawings to diagnose cumulative position errors
//!
//! Creates two test patterns:
//! 1. Concentric squares - visual alignment test (all start/end at same corner)
//! 2. Zigzag pattern - rapid direction changes returning to origin
//!
//! If there's position drift, the return paths won't align with start points.
//!
//! Run with: cargo run -p plotta-cli --example create_drift_test
//!
//! Then plot with:
//!   cargo run -p plotta-cli -- plot drift-test-squares.json --verify-position -v
//!   cargo run -p plotta-cli -- plot drift-test-zigzag.json --verify-position -v

use drawing_core::{Drawing, Element, Point};

fn main() {
    create_concentric_squares();
    create_zigzag_pattern();
    println!();
    println!("To test for drift:");
    println!("  cargo run -p plotta-cli -- plot drift-test-squares.json -v");
    println!("  cargo run -p plotta-cli -- plot drift-test-zigzag.json -v");
}

/// Concentric squares that all start/end at the same corner
///
/// If drift accumulates, the corners won't align when plotted.
/// Each square is drawn as a closed path starting from bottom-left corner.
fn create_concentric_squares() {
    let mut drawing = Drawing::new(150.0, 150.0);
    let center = Point::new(75.0, 75.0);

    // Create 10 concentric squares, all starting from bottom-left corner
    // This maximizes the chance of seeing drift since all paths share a common point
    for i in 1..=10 {
        let half_size = 10.0 + i as f64 * 5.0; // 15mm to 60mm

        // Square as a closed path starting from bottom-left, going clockwise
        let bl = Point::new(center.x - half_size, center.y + half_size);
        let br = Point::new(center.x + half_size, center.y + half_size);
        let tr = Point::new(center.x + half_size, center.y - half_size);
        let tl = Point::new(center.x - half_size, center.y - half_size);

        // Closed polygon starting from bottom-left, going clockwise
        drawing.add(Element::polygon_from_points(vec![bl, br, tr, tl]));
    }

    // Add alignment cross at center for reference
    drawing.add(Element::line(
        Point::new(center.x - 5.0, center.y),
        Point::new(center.x + 5.0, center.y),
    ));
    drawing.add(Element::line(
        Point::new(center.x, center.y - 5.0),
        Point::new(center.x, center.y + 5.0),
    ));

    // Add a small marker at the common start point (bottom-left of smallest square)
    let marker_pos = Point::new(center.x - 15.0, center.y + 15.0);
    drawing.add(Element::circle(marker_pos, 1.5));

    let path = "drift-test-squares.json";
    drawing.save(path).expect("Failed to save drawing");

    println!("Created: {}", path);
    println!("  10 concentric squares, all starting from bottom-left corner");
    println!("  If drift occurs, the corners won't align");
    println!("  Size: {} x {} mm", drawing.width, drawing.height);
}

/// Zigzag pattern that returns to origin periodically
///
/// Tests rapid X/Y direction changes and return accuracy.
/// The final point should exactly match the start point if no drift.
fn create_zigzag_pattern() {
    let mut drawing = Drawing::new(150.0, 100.0);

    let start = Point::new(20.0, 50.0);
    let amplitude = 30.0; // Height of zigzag
    let step = 10.0; // Horizontal step per segment
    let cycles = 10; // Number of complete zigzags

    let mut points = vec![start];
    let mut x = start.x;

    // Create zigzag pattern going right
    for i in 0..(cycles * 2) {
        x += step;
        let y = if i % 2 == 0 {
            start.y - amplitude
        } else {
            start.y + amplitude
        };
        points.push(Point::new(x, y));
    }

    // Return to start point along a straight diagonal
    // This creates a closed shape - if there's drift, it won't close properly
    points.push(start);

    drawing.add(Element::polyline(points));

    // Mark the start/end point with a small circle
    drawing.add(Element::circle(start, 2.0));

    // Add a reference line at the start position
    drawing.add(Element::line(
        Point::new(start.x, start.y - 5.0),
        Point::new(start.x, start.y + 5.0),
    ));

    let path = "drift-test-zigzag.json";
    drawing.save(path).expect("Failed to save drawing");

    println!("Created: {}", path);
    println!(
        "  Zigzag with {} segments returning to origin",
        cycles * 2 + 1
    );
    println!("  If drift occurs, the return path won't meet the start point");
    println!("  Size: {} x {} mm", drawing.width, drawing.height);
}
