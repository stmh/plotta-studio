//! Hatching utilities for filling shapes with parallel lines

use drawing_core::{Color, Element, Group, Point};

/// Options for generating hatch lines
#[derive(Debug, Clone)]
pub struct HatchOptions {
    /// Spacing between hatch lines (in drawing units, typically mm)
    pub spacing: f64,
    /// Rotation angle in radians
    pub angle: f64,
    /// Stroke width for hatch lines
    pub stroke_width: f64,
    /// Stroke color for hatch lines
    pub color: Color,
}

impl Default for HatchOptions {
    fn default() -> Self {
        Self {
            spacing: 2.0,
            angle: 0.0,
            stroke_width: 0.3,
            color: Color::BLACK,
        }
    }
}

impl HatchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    pub fn angle_deg(mut self, degrees: f64) -> Self {
        self.angle = degrees.to_radians();
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
}

/// Generate parallel hatch lines covering a circular area
///
/// The lines are generated to cover a circle centered at `center` with the given `radius`.
/// Lines extend beyond the circle to ensure full coverage after rotation.
///
/// # Example
///
/// ```ignore
/// use drawing_utils::{generate_hatch_lines, HatchOptions};
/// use drawing_core::{Element, Point};
///
/// let hatch = generate_hatch_lines(
///     Point::new(50.0, 50.0),
///     40.0,
///     &HatchOptions::default().angle_deg(45.0),
/// );
///
/// // Clip to a circle
/// let hatched_circle = Element::clip(Element::circle((50.0, 50.0), 40.0))
///     .add(hatch);
/// ```
pub fn generate_hatch_lines(center: Point, radius: f64, options: &HatchOptions) -> Element {
    let mut group = Group::new();

    // Extend beyond radius to ensure coverage after rotation
    let extent = radius * 1.5;

    // Generate horizontal lines from -extent to +extent
    let mut y = -extent;
    while y <= extent {
        let line = Element::line(
            Point::new(center.x - extent, center.y + y),
            Point::new(center.x + extent, center.y + y),
        )
        .stroke_width(options.stroke_width)
        .stroke_color(options.color);

        group.push(line);
        y += options.spacing;
    }

    // Rotate the entire group around the center
    if options.angle.abs() > 1e-10 {
        Element::group(group).rotate_around(options.angle, center)
    } else {
        Element::group(group)
    }
}

/// Generate hatch lines covering a rectangular area
///
/// Similar to `generate_hatch_lines` but for rectangular regions.
pub fn generate_hatch_lines_rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    options: &HatchOptions,
) -> Element {
    let center = Point::new(x + width / 2.0, y + height / 2.0);
    let radius = (width.powi(2) + height.powi(2)).sqrt() / 2.0;

    generate_hatch_lines(center, radius, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hatch_options_default() {
        let opts = HatchOptions::default();
        assert!((opts.spacing - 2.0).abs() < 1e-10);
        assert!((opts.angle - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_hatch_options_builder() {
        let opts = HatchOptions::new()
            .spacing(3.0)
            .angle_deg(45.0)
            .stroke_width(0.5)
            .color(Color::RED);

        assert!((opts.spacing - 3.0).abs() < 1e-10);
        assert!((opts.angle - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
        assert!((opts.stroke_width - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_generate_hatch_lines() {
        let hatch = generate_hatch_lines(Point::new(50.0, 50.0), 40.0, &HatchOptions::default());

        // Should return a group element
        match hatch.shape {
            drawing_core::Shape::Group(_) => {}
            _ => panic!("Expected Group shape"),
        }
    }
}
