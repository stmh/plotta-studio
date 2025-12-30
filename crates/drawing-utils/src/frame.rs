//! Frame and title utilities for drawings

use drawing_core::{Color, Drawing, Element, FontRef, Group, TextAlign, TextOptions};
use drawing_text::FontManager;

use crate::signature::{signature_bounds, signature_normalized};

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
    /// Whether to include a signature in the bottom right corner
    pub with_signature: bool,
    /// Height of the signature in drawing units (width scales proportionally)
    pub signature_height: f64,
    /// Horizontal offset of signature from right edge of frame
    pub signature_margin: f64,
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
            font_size: 4.5,
            title_offset: 2.0,
            title_margin: 0.0,
            with_signature: false,
            signature_height: 7.0,
            signature_margin: 0.0,
        }
    }

    /// Create new frame options with the default font (ReliefSingleLine).
    ///
    /// Returns None if the default font is not loaded in the FontManager.
    /// Make sure to call `manager.load_relief_single_line()` or
    /// `manager.load_all_builtin()` first.
    pub fn with_default_font(manager: &FontManager) -> Option<Self> {
        manager.default_font().map(Self::new)
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

    /// Enable signature in the bottom right corner
    pub fn with_signature(mut self) -> Self {
        self.with_signature = true;
        self
    }

    /// Set the height of the signature (width scales proportionally)
    pub fn signature_height(mut self, height: f64) -> Self {
        self.signature_height = height;
        self
    }

    /// Set the horizontal margin of the signature from the right edge
    pub fn signature_margin(mut self, margin: f64) -> Self {
        self.signature_margin = margin;
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

    // Add signature if enabled
    if options.with_signature {
        let (_, _, sig_width, sig_height) = signature_bounds();
        let scale = options.signature_height / sig_height;
        let scaled_width = sig_width * scale;

        // Position in bottom right corner, below the frame line
        let sig_x = drawing.width - options.margin_right - scaled_width - options.signature_margin;
        let sig_y = drawing.height - options.margin_bottom + options.title_offset;

        group.push(
            signature_normalized()
                .scale(scale, scale)
                .translate(sig_x, sig_y)
                .stroke_width(options.stroke_width)
                .stroke_color(options.color)
                .scale_stroke(false),
        );
    }

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
