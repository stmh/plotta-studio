//! Plotter setup diagram renderer.
//!
//! Renders a terminal diagram showing the physical arrangement of:
//! - Plotter body (700mm x 100mm)
//! - Bed area (460mm x 325mm)
//! - Drawing rectangle (user's drawing dimensions)
//! - Orientation markers

use crate::canvas::{Canvas, Color};
use std::str::FromStr;

/// Physical dimensions in mm
const PLOTTER_WIDTH_MM: f64 = 560.0; // 700mm * 0.8
const PLOTTER_HEIGHT_MM: f64 = 100.0;
const BED_WIDTH_MM: f64 = 460.0;
const BED_HEIGHT_MM: f64 = 325.0;

/// Target canvas width range (in logical units before aspect compensation)
const MIN_CANVAS_WIDTH: usize = 14;
const MAX_CANVAS_WIDTH: usize = 20;

/// Plotter position relative to the bed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlotterSetup {
    /// Plotter above bed, up-vector points up (easiest orientation)
    #[default]
    Top,
    /// Plotter below bed, up-vector points down
    Bottom,
    /// Plotter left of bed, up-vector points left
    Left,
    /// Plotter right of bed, up-vector points right
    Right,
}

impl PlotterSetup {
    /// Get the up-vector arrow character for this setup
    pub fn up_arrow(&self) -> char {
        match self {
            PlotterSetup::Top => '↑',
            PlotterSetup::Bottom => '↓',
            PlotterSetup::Left => '←',
            PlotterSetup::Right => '→',
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            PlotterSetup::Top => "plotter on top, bed below",
            PlotterSetup::Bottom => "plotter on bottom, bed above",
            PlotterSetup::Left => "plotter on left, bed to right",
            PlotterSetup::Right => "plotter on right, bed to left",
        }
    }
}

impl FromStr for PlotterSetup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "top" => Ok(PlotterSetup::Top),
            "bottom" => Ok(PlotterSetup::Bottom),
            "left" => Ok(PlotterSetup::Left),
            "right" => Ok(PlotterSetup::Right),
            _ => Err(format!(
                "Invalid plotter setup '{}'. Valid options: top, bottom, left, right",
                s
            )),
        }
    }
}

/// Unicode markers for the diagram
pub const MARKER_PAPER_ORIGIN: char = '⊙';
pub const MARKER_PRINTHEAD_HOME: char = '⌂';

/// Setup diagram renderer
pub struct SetupDiagram {
    setup: PlotterSetup,
    drawing_width: f64,
    drawing_height: f64,
}

impl SetupDiagram {
    /// Create a new setup diagram for the given configuration.
    pub fn new(setup: PlotterSetup, drawing_width: f64, drawing_height: f64) -> Self {
        Self {
            setup,
            drawing_width,
            drawing_height,
        }
    }

    /// Calculate the total bounding box dimensions in mm for the layout.
    fn total_dimensions(&self) -> (f64, f64) {
        match self.setup {
            PlotterSetup::Top | PlotterSetup::Bottom => {
                // Plotter and bed stacked vertically
                // Plotter is wider (700mm) than bed (460mm), so total width = plotter width
                let total_width = PLOTTER_WIDTH_MM.max(BED_WIDTH_MM);
                let total_height = PLOTTER_HEIGHT_MM + BED_HEIGHT_MM;
                (total_width, total_height)
            }
            PlotterSetup::Left | PlotterSetup::Right => {
                // Plotter and bed side by side, both rotated 90 degrees
                // Plotter: 700mm becomes height, 100mm becomes width
                // Bed: 460mm becomes height, 325mm becomes width
                let total_width = PLOTTER_HEIGHT_MM + BED_HEIGHT_MM;
                let total_height = PLOTTER_WIDTH_MM.max(BED_WIDTH_MM);
                (total_width, total_height)
            }
        }
    }

    /// Calculate scaling factor to fit within target canvas width
    fn calculate_scale(&self) -> f64 {
        let (total_w, total_h) = self.total_dimensions();

        // We want the larger dimension to fit within our target width
        let max_dim = total_w.max(total_h);
        let target_width = ((MIN_CANVAS_WIDTH + MAX_CANVAS_WIDTH) / 2) as f64;

        target_width / max_dim
    }

    /// Scale mm to canvas units
    fn scale(&self, mm: f64) -> i32 {
        let scale = self.calculate_scale();
        (mm * scale).round() as i32
    }

    /// Render the diagram to a canvas and return it
    pub fn render(&self) -> Canvas {
        let (total_w, total_h) = self.total_dimensions();
        let scale = self.calculate_scale();

        // Canvas dimensions (add padding)
        let canvas_w = (total_w * scale).ceil() as usize + 4;
        let canvas_h = (total_h * scale).ceil() as usize + 4;

        let mut canvas = Canvas::new(canvas_w, canvas_h);

        // Calculate positions based on setup
        let (plotter_x, plotter_y, plotter_w, plotter_h) = self.plotter_rect();
        let (bed_x, bed_y, bed_w, bed_h) = self.bed_rect();
        let (drawing_x, drawing_y, drawing_w, drawing_h) = self.drawing_rect();

        // Draw plotter (grey)
        canvas.draw_rect(
            plotter_x + 1,
            plotter_y + 1,
            plotter_w as u32,
            plotter_h as u32,
            Color::Grey,
        );

        // Draw bed (grey)
        canvas.draw_rect(
            bed_x + 1,
            bed_y + 1,
            bed_w as u32,
            bed_h as u32,
            Color::Grey,
        );

        // Draw drawing area (white)
        if drawing_w >= 2 && drawing_h >= 2 {
            canvas.draw_rect(
                drawing_x + 1,
                drawing_y + 1,
                drawing_w as u32,
                drawing_h as u32,
                Color::White,
            );
        }

        // Draw markers (green)
        // Paper origin marker - at the corner of the drawing closest to printhead home
        let (origin_x, origin_y) = self.paper_origin_pos();
        canvas.draw_char(
            origin_x + 1,
            origin_y + 1,
            MARKER_PAPER_ORIGIN,
            Color::Green,
        );

        // Printhead home marker - at the plotter's home position
        let (home_x, home_y) = self.printhead_home_pos();
        canvas.draw_char(home_x + 1, home_y + 1, MARKER_PRINTHEAD_HOME, Color::Green);

        // Up-vector arrow - inside the drawing area pointing toward plotter
        let (arrow_x, arrow_y) = self.up_arrow_pos();
        canvas.draw_char(
            arrow_x + 1,
            arrow_y + 1,
            self.setup.up_arrow(),
            Color::Green,
        );

        canvas
    }

    /// Get plotter rectangle (x, y, w, h) in canvas units
    fn plotter_rect(&self) -> (i32, i32, i32, i32) {
        match self.setup {
            PlotterSetup::Top => {
                // Plotter on top, centered horizontally
                let w = self.scale(PLOTTER_WIDTH_MM);
                let h = self.scale(PLOTTER_HEIGHT_MM);
                (0, 0, w, h)
            }
            PlotterSetup::Bottom => {
                // Plotter on bottom
                let w = self.scale(PLOTTER_WIDTH_MM);
                let h = self.scale(PLOTTER_HEIGHT_MM);
                let bed_h = self.scale(BED_HEIGHT_MM);
                (0, bed_h, w, h)
            }
            PlotterSetup::Left => {
                // Plotter on left (rotated, so width/height swapped)
                let w = self.scale(PLOTTER_HEIGHT_MM);
                let h = self.scale(PLOTTER_WIDTH_MM);
                (0, 0, w, h)
            }
            PlotterSetup::Right => {
                // Plotter on right, adjacent to bed
                // Bed is rotated, so its width is BED_HEIGHT_MM
                let w = self.scale(PLOTTER_HEIGHT_MM);
                let h = self.scale(PLOTTER_WIDTH_MM);
                let bed_w = self.scale(BED_HEIGHT_MM);
                (bed_w, 0, w, h)
            }
        }
    }

    /// Get bed rectangle (x, y, w, h) in canvas units
    fn bed_rect(&self) -> (i32, i32, i32, i32) {
        match self.setup {
            PlotterSetup::Top => {
                // Bed below plotter
                let w = self.scale(BED_WIDTH_MM);
                let h = self.scale(BED_HEIGHT_MM);
                let plotter_h = self.scale(PLOTTER_HEIGHT_MM);
                // Center bed under plotter
                let offset_x = (self.scale(PLOTTER_WIDTH_MM) - w) / 2;
                (offset_x, plotter_h, w, h)
            }
            PlotterSetup::Bottom => {
                // Bed above plotter
                let w = self.scale(BED_WIDTH_MM);
                let h = self.scale(BED_HEIGHT_MM);
                let offset_x = (self.scale(PLOTTER_WIDTH_MM) - w) / 2;
                (offset_x, 0, w, h)
            }
            PlotterSetup::Left => {
                // Bed to the right of plotter, rotated 90 degrees
                // Bed dimensions swapped: height becomes width, width becomes height
                let w = self.scale(BED_HEIGHT_MM);
                let h = self.scale(BED_WIDTH_MM);
                let plotter_w = self.scale(PLOTTER_HEIGHT_MM);
                // Center bed vertically relative to plotter
                let plotter_h = self.scale(PLOTTER_WIDTH_MM);
                let offset_y = (plotter_h - h) / 2;
                (plotter_w, offset_y.max(0), w, h)
            }
            PlotterSetup::Right => {
                // Bed to the left of plotter, rotated 90 degrees
                // Bed dimensions swapped: height becomes width, width becomes height
                let w = self.scale(BED_HEIGHT_MM);
                let h = self.scale(BED_WIDTH_MM);
                let plotter_h = self.scale(PLOTTER_WIDTH_MM);
                let offset_y = (plotter_h - h) / 2;
                (0, offset_y.max(0), w, h)
            }
        }
    }

    /// Get drawing rectangle (x, y, w, h) in canvas units
    fn drawing_rect(&self) -> (i32, i32, i32, i32) {
        let (bed_x, bed_y, bed_w, bed_h) = self.bed_rect();

        // For Left/Right setups, drawing is also rotated 90° (swap width/height)
        let (draw_w, draw_h) = match self.setup {
            PlotterSetup::Top | PlotterSetup::Bottom => {
                let w = self.scale(self.drawing_width).min(bed_w - 2).max(1);
                let h = self.scale(self.drawing_height).min(bed_h - 2).max(1);
                (w, h)
            }
            PlotterSetup::Left | PlotterSetup::Right => {
                // Swap dimensions for rotated drawing
                let w = self.scale(self.drawing_height).min(bed_w - 2).max(1);
                let h = self.scale(self.drawing_width).min(bed_h - 2).max(1);
                (w, h)
            }
        };

        // Position drawing near the origin (closest to plotter)
        let margin = 1;
        match self.setup {
            PlotterSetup::Top => {
                // Origin at top-left of bed (closest to plotter)
                (bed_x + margin, bed_y + margin, draw_w, draw_h)
            }
            PlotterSetup::Bottom => {
                // Origin at bottom-right of bed (closest to plotter)
                (
                    bed_x + bed_w - draw_w - margin,
                    bed_y + bed_h - draw_h - margin,
                    draw_w,
                    draw_h,
                )
            }
            PlotterSetup::Left => {
                // Origin at bottom-left of bed (closest to plotter)
                (
                    bed_x + margin,
                    bed_y + bed_h - draw_h - margin,
                    draw_w,
                    draw_h,
                )
            }
            PlotterSetup::Right => {
                // Origin at right side of bed (closest to plotter)
                (
                    bed_x + bed_w - draw_w - margin,
                    bed_y + margin,
                    draw_w,
                    draw_h,
                )
            }
        }
    }

    /// Get paper origin marker position in canvas units
    /// The origin is at the corner of the drawing closest to the printhead home
    fn paper_origin_pos(&self) -> (i32, i32) {
        let (x, y, w, h) = self.drawing_rect();
        match self.setup {
            PlotterSetup::Top => {
                // Origin at top-left of drawing
                (x, y)
            }
            PlotterSetup::Bottom => {
                // Origin at bottom-right of drawing (on the corner)
                (x + w - 1, y + h - 1)
            }
            PlotterSetup::Left => {
                // Origin at bottom-left of drawing
                (x, y + h - 1)
            }
            PlotterSetup::Right => {
                // Origin at top-right of drawing
                (x + w - 1, y)
            }
        }
    }

    /// Get printhead home position in canvas units
    fn printhead_home_pos(&self) -> (i32, i32) {
        let (bed_x, bed_y, bed_w, bed_h) = self.bed_rect();

        match self.setup {
            PlotterSetup::Top => {
                // Home is at top-left of bed
                (bed_x, bed_y)
            }
            PlotterSetup::Bottom => {
                // Home is at bottom-right of bed (on the corner)
                (bed_x + bed_w - 1, bed_y + bed_h - 1)
            }
            PlotterSetup::Left => {
                // Home is at bottom-left edge of bed
                (bed_x, bed_y + bed_h - 1)
            }
            PlotterSetup::Right => {
                // Home is at right edge of bed
                (bed_x + bed_w - 1, bed_y)
            }
        }
    }

    /// Get up-arrow position (center of drawing area)
    fn up_arrow_pos(&self) -> (i32, i32) {
        let (x, y, w, h) = self.drawing_rect();
        (x + w / 2, y + h / 2)
    }

    /// Render the diagram to terminal
    pub fn render_to_terminal(&self) {
        let canvas = self.render();
        canvas.print();
    }

    /// Print legend with specific arrow for current setup
    pub fn print_legend_for_setup(&self) {
        use crossterm::style::{Print, SetForegroundColor};
        use std::io::stdout;

        let mut stdout = stdout();

        println!("Legend:");
        let _ = crossterm::execute!(
            stdout,
            Print("  "),
            SetForegroundColor(crossterm::style::Color::Green),
            Print(MARKER_PAPER_ORIGIN),
            SetForegroundColor(crossterm::style::Color::Reset),
            Print("  Paper origin (0,0)\n"),
        );
        let _ = crossterm::execute!(
            stdout,
            Print("  "),
            SetForegroundColor(crossterm::style::Color::Green),
            Print(MARKER_PRINTHEAD_HOME),
            SetForegroundColor(crossterm::style::Color::Reset),
            Print("  Printhead home\n"),
        );
        let _ = crossterm::execute!(
            stdout,
            Print("  "),
            SetForegroundColor(crossterm::style::Color::Green),
            Print(format!("{}", self.setup.up_arrow())),
            SetForegroundColor(crossterm::style::Color::Reset),
            Print("   Up direction\n"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plotter_setup_from_str() {
        assert_eq!(PlotterSetup::from_str("top").unwrap(), PlotterSetup::Top);
        assert_eq!(PlotterSetup::from_str("TOP").unwrap(), PlotterSetup::Top);
        assert_eq!(
            PlotterSetup::from_str("bottom").unwrap(),
            PlotterSetup::Bottom
        );
        assert_eq!(PlotterSetup::from_str("left").unwrap(), PlotterSetup::Left);
        assert_eq!(
            PlotterSetup::from_str("right").unwrap(),
            PlotterSetup::Right
        );
        assert!(PlotterSetup::from_str("invalid").is_err());
    }

    #[test]
    fn test_setup_diagram_renders() {
        let diagram = SetupDiagram::new(PlotterSetup::Top, 297.0, 210.0);
        let canvas = diagram.render();

        // Should produce a non-empty canvas
        assert!(canvas.char_width() > 0);
        assert!(canvas.char_height() > 0);

        // Render should produce output
        let output = canvas.render();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_all_setups_render() {
        for setup in [
            PlotterSetup::Top,
            PlotterSetup::Bottom,
            PlotterSetup::Left,
            PlotterSetup::Right,
        ] {
            let diagram = SetupDiagram::new(setup, 297.0, 210.0);
            let canvas = diagram.render();
            let output = canvas.render();
            assert!(!output.is_empty(), "Setup {:?} should render", setup);
        }
    }
}
