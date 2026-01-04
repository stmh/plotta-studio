//! Terminal canvas for drawing ASCII diagrams with color support.

use crossterm::style::{Color as CtColor, SetForegroundColor};
use std::fmt::Write as FmtWrite;

/// ANSI colors for terminal output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    /// Default terminal color
    #[default]
    Default,
    /// Grey color for plotter/bed
    Grey,
    /// White/bright color for drawing
    White,
    /// Green color for markers
    Green,
}

impl Color {
    /// Convert to crossterm color
    fn to_crossterm(self) -> Option<CtColor> {
        match self {
            Color::Default => None,
            Color::Grey => Some(CtColor::DarkGrey),
            Color::White => Some(CtColor::White),
            Color::Green => Some(CtColor::Green),
        }
    }
}

/// A single cell in the canvas
#[derive(Clone)]
struct Cell {
    ch: char,
    color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            color: Color::Default,
        }
    }
}

/// A 2D canvas for terminal rendering with ANSI color support.
///
/// The canvas uses a coordinate system where:
/// - (0, 0) is the top-left corner
/// - X increases to the right
/// - Y increases downward
///
/// Aspect ratio compensation: terminal characters are typically ~2:1 (height:width),
/// so horizontal coordinates are doubled internally to make shapes appear more square.
pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,
    /// If true, double horizontal coordinates for aspect ratio compensation
    aspect_compensate: bool,
}

impl Canvas {
    /// Create a new canvas with the given dimensions.
    ///
    /// Width and height are in "logical" units. If aspect compensation is enabled,
    /// the actual character width will be doubled.
    pub fn new(width: usize, height: usize) -> Self {
        // Double the width for aspect ratio compensation
        let actual_width = width * 2;
        let cells = vec![vec![Cell::default(); actual_width]; height];
        Self {
            width: actual_width,
            height,
            cells,
            aspect_compensate: true,
        }
    }

    /// Create a canvas without aspect ratio compensation (1:1 character mapping)
    #[allow(dead_code)]
    pub fn new_no_aspect(width: usize, height: usize) -> Self {
        let cells = vec![vec![Cell::default(); width]; height];
        Self {
            width,
            height,
            cells,
            aspect_compensate: false,
        }
    }

    /// Get the actual width in characters
    pub fn char_width(&self) -> usize {
        self.width
    }

    /// Get the height in characters
    pub fn char_height(&self) -> usize {
        self.height
    }

    /// Convert logical X coordinate to actual character position
    fn to_char_x(&self, x: i32) -> i32 {
        if self.aspect_compensate {
            x * 2
        } else {
            x
        }
    }

    /// Set a cell at the given position
    fn set_cell(&mut self, x: i32, y: i32, ch: char, color: Color) {
        if x >= 0 && y >= 0 {
            let ux = x as usize;
            let uy = y as usize;
            if uy < self.height && ux < self.width {
                self.cells[uy][ux] = Cell { ch, color };
            }
        }
    }

    /// Draw a single character at the given logical position.
    pub fn draw_char(&mut self, x: i32, y: i32, ch: char, color: Color) {
        let cx = self.to_char_x(x);
        self.set_cell(cx, y, ch, color);
        // For aspect compensation, fill second character too (for wide chars)
        if self.aspect_compensate {
            // Don't overwrite with space - leave second cell for box drawing
        }
    }

    /// Draw text at the given logical position.
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Color) {
        let cx = self.to_char_x(x);
        for (i, ch) in text.chars().enumerate() {
            self.set_cell(cx + i as i32, y, ch, color);
        }
    }

    /// Draw a rectangle outline using box-drawing characters.
    ///
    /// The rectangle is drawn from (x, y) to (x + w - 1, y + h - 1).
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        if w == 0 || h == 0 {
            return;
        }

        let w = w as i32;
        let h = h as i32;

        // Convert to character coordinates
        let x1 = self.to_char_x(x);
        let _x2 = self.to_char_x(x + w - 1);
        let y1 = y;
        let y2 = y + h - 1;

        // For aspect-compensated canvas, we need to handle the width properly
        let actual_w = if self.aspect_compensate { w * 2 } else { w };

        // Corners
        self.set_cell(x1, y1, '┌', color);
        self.set_cell(x1 + actual_w - 1, y1, '┐', color);
        self.set_cell(x1, y2, '└', color);
        self.set_cell(x1 + actual_w - 1, y2, '┘', color);

        // Top and bottom edges
        for dx in 1..(actual_w - 1) {
            self.set_cell(x1 + dx, y1, '─', color);
            self.set_cell(x1 + dx, y2, '─', color);
        }

        // Left and right edges
        for dy in 1..(h - 1) {
            self.set_cell(x1, y1 + dy, '│', color);
            self.set_cell(x1 + actual_w - 1, y1 + dy, '│', color);
        }
    }

    /// Draw a filled rectangle.
    #[allow(dead_code)]
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, ch: char, color: Color) {
        let x1 = self.to_char_x(x);
        let actual_w = if self.aspect_compensate { w * 2 } else { w } as i32;

        for dy in 0..h as i32 {
            for dx in 0..actual_w {
                self.set_cell(x1 + dx, y + dy, ch, color);
            }
        }
    }

    /// Render the canvas to a string with ANSI color codes.
    pub fn render(&self) -> String {
        let mut output = String::new();
        let mut current_color = Color::Default;

        for row in &self.cells {
            for cell in row {
                // Change color if needed
                if cell.color != current_color {
                    if let Some(ct_color) = cell.color.to_crossterm() {
                        write!(output, "{}", SetForegroundColor(ct_color)).ok();
                    } else {
                        write!(output, "{}", SetForegroundColor(CtColor::Reset)).ok();
                    }
                    current_color = cell.color;
                }
                output.push(cell.ch);
            }
            // Reset color at end of line and add newline
            if current_color != Color::Default {
                write!(output, "{}", SetForegroundColor(CtColor::Reset)).ok();
                current_color = Color::Default;
            }
            output.push('\n');
        }

        output
    }

    /// Print the canvas to stdout.
    pub fn print(&self) {
        print!("{}", self.render());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new(10, 5);
        // With aspect compensation, width is doubled
        assert_eq!(canvas.char_width(), 20);
        assert_eq!(canvas.char_height(), 5);
    }

    #[test]
    fn test_draw_rect() {
        let mut canvas = Canvas::new_no_aspect(10, 5);
        canvas.draw_rect(0, 0, 5, 3, Color::Default);

        let rendered = canvas.render();
        let lines: Vec<&str> = rendered.lines().collect();

        assert!(lines[0].starts_with("┌───┐"));
        assert!(lines[1].starts_with("│   │"));
        assert!(lines[2].starts_with("└───┘"));
    }

    #[test]
    fn test_draw_text() {
        let mut canvas = Canvas::new_no_aspect(10, 3);
        canvas.draw_text(2, 1, "Hello", Color::Default);

        let rendered = canvas.render();
        let lines: Vec<&str> = rendered.lines().collect();

        assert!(lines[1].contains("Hello"));
    }
}
