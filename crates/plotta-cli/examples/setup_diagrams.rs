//! Example that renders all 4 plotter setup orientations for debugging.
//!
//! Run with: cargo run -p plotta-cli --example setup_diagrams

use plotta_cli::setup_diagram::{PlotterSetup, SetupDiagram};

fn main() {
    // Sample A4 portrait dimensions
    let drawing_width = 210.0;
    let drawing_height = 297.0;

    println!("Plotter Setup Diagram Examples");
    println!(
        "Drawing size: {} x {} mm (A4 portrait)",
        drawing_width, drawing_height
    );
    println!();

    // Render all 4 orientations
    for setup in [
        PlotterSetup::Top,
        PlotterSetup::Bottom,
        PlotterSetup::Left,
        PlotterSetup::Right,
    ] {
        println!("═══════════════════════════════════════════════════════════════");
        println!("Setup: {:?} - {}", setup, setup.description());
        println!("═══════════════════════════════════════════════════════════════");
        println!();

        let diagram = SetupDiagram::new(setup, drawing_width, drawing_height);
        diagram.render_to_terminal();
        println!();
    }

    // Print legend once at the end (using Top setup as reference)
    println!("═══════════════════════════════════════════════════════════════");
    let diagram = SetupDiagram::new(PlotterSetup::Top, drawing_width, drawing_height);
    diagram.print_legend_for_setup();
    println!();
    println!("Grey: Plotter body and bed");
    println!("White: Drawing area");
    println!("Green: Markers");
}
