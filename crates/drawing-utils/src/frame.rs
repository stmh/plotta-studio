//! Frame and title utilities for drawings

use drawing_core::{Color, Drawing, Element, FontRef, Group, TextAlign, TextOptions};

/// Options for drawing a frame with title
#[derive(Clone)]
pub struct FrameOptions {
    /// Margin from the top edge of the drawing (in drawing units, typically mm)
    pub margin_top: f64,
    /// Margin from the right edge of the drawing
    pub margin_right: f64,
    /// Margin from the bottom edge of the drawing
    pub margin_bottom: f64,
    /// Margin from the left edge of the drawing
    pub margin_left: f64,
    /// Stroke width for the frame
    pub stroke_width: f64,
    /// Stroke color for the frame
    pub color: Color,
    /// Font for the title
    pub font: FontRef,
    /// Font size for the title
    pub font_size: f64,
    /// Vertical offset of title from bottom of frame (below the line)
    pub title_offset: f64,
    /// Horizontal offset of title from left of frame
    pub title_margin: f64,
}

impl FrameOptions {
    /// Create new frame options with the given font
    pub fn new(font: FontRef) -> Self {
        Self {
            margin_top: 5.0,
            margin_right: 5.0,
            margin_bottom: 10.0,
            margin_left: 5.0,
            stroke_width: 0.5,
            color: Color::BLACK,
            font,
            font_size: 3.0,
            title_offset: 2.0,
            title_margin: 0.0,
        }
    }

    /// Set all margins to the same value
    pub fn margin(mut self, margin: f64) -> Self {
        self.margin_top = margin;
        self.margin_right = margin;
        self.margin_bottom = margin;
        self.margin_left = margin;
        self
    }

    pub fn margin_top(mut self, margin: f64) -> Self {
        self.margin_top = margin;
        self
    }

    pub fn margin_right(mut self, margin: f64) -> Self {
        self.margin_right = margin;
        self
    }

    pub fn margin_bottom(mut self, margin: f64) -> Self {
        self.margin_bottom = margin;
        self
    }

    pub fn margin_left(mut self, margin: f64) -> Self {
        self.margin_left = margin;
        self
    }

    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    pub fn title_offset(mut self, offset: f64) -> Self {
        self.title_offset = offset;
        self
    }

    pub fn title_margin(mut self, margin: f64) -> Self {
        self.title_margin = margin;
        self
    }
}

/// Draw a frame (border rectangle) on the drawing
///
/// # Example
///
/// ```ignore
/// use drawing_utils::draw_frame;
/// use drawing_core::Drawing;
///
/// let mut drawing = Drawing::a4_landscape();
/// let frame = draw_frame(&drawing, 10.0, 0.5); // margin, stroke_width
/// drawing.add(frame);
/// ```
pub fn draw_frame(drawing: &Drawing, margin: f64, stroke_width: f64) -> Element {
    Element::rect(
        margin,
        margin,
        drawing.width - margin * 2.0,
        drawing.height - margin * 2.0,
    )
    .stroke_width(stroke_width)
    .stroke_color(Color::BLACK)
}

/// Draw a frame with a title in the lower left corner
///
/// The title is positioned below the frame line, in the lower left corner.
///
/// # Example
///
/// ```ignore
/// use drawing_utils::{draw_frame_with_title, FrameOptions};
/// use drawing_core::Drawing;
///
/// let font = /* load font */;
/// let mut drawing = Drawing::a4_landscape();
/// let frame = draw_frame_with_title(&drawing, "My Sketch", &FrameOptions::new(font));
/// drawing.add(frame);
/// ```
pub fn draw_frame_with_title(drawing: &Drawing, title: &str, options: &FrameOptions) -> Element {
    let mut group = Group::new();

    // Add the frame rectangle
    let frame_x = options.margin_left;
    let frame_y = options.margin_top;
    let frame_width = drawing.width - options.margin_left - options.margin_right;
    let frame_height = drawing.height - options.margin_top - options.margin_bottom;

    group.push(
        Element::rect(frame_x, frame_y, frame_width, frame_height)
            .stroke_width(options.stroke_width)
            .stroke_color(options.color),
    );

    // Add the title text below the frame line in lower left corner
    // Position: below the bottom frame line, offset from left
    let text_x = options.margin_left + options.title_margin;
    let text_y = drawing.height - options.margin_bottom + options.title_offset + options.font_size;

    let text_options = TextOptions {
        size: options.font_size,
        align: TextAlign::Left,
        ..Default::default()
    };

    group.push(
        Element::text(title, options.font.clone())
            .text_options(text_options)
            .translate(text_x, text_y)
            .stroke_width(options.stroke_width)
            .stroke_color(options.color),
    );

    Element::group(group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_frame() {
        let drawing = Drawing::a4_landscape();
        let frame = draw_frame(&drawing, 10.0, 0.5);

        // Should return a rect element
        match frame.shape {
            drawing_core::Shape::Rect(_) => {}
            _ => panic!("Expected Rect shape"),
        }
    }

    // Note: draw_frame_with_title tests require a font, tested in integration tests
}
