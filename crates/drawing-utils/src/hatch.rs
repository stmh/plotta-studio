//! Hatching utilities for filling shapes with parallel lines

use drawing_core::{Color, Element, Group, Point};
use rand::Rng;

/// Safety margin multiplier to ensure hatch lines cover rotated shapes
const HATCH_EXTENT_MULTIPLIER: f64 = 1.5;

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
    /// Random offset applied to each line's y-position (0.0 = no randomness)
    /// Value is a fraction of spacing (e.g., 0.5 = up to half spacing offset)
    pub position_jitter: f64,
    /// Random offset applied to line endpoints (0.0 = no randomness)
    /// Value is in drawing units (typically mm)
    pub endpoint_jitter: f64,
    /// Random angle variation applied to each line in radians (0.0 = no randomness)
    pub angle_jitter: f64,
    /// Random seed for reproducible randomness (None = use system entropy)
    pub seed: Option<u64>,
}

impl Default for HatchOptions {
    fn default() -> Self {
        Self {
            spacing: 2.0,
            angle: 0.0,
            stroke_width: 0.3,
            color: Color::BLACK,
            position_jitter: 0.0,
            endpoint_jitter: 0.0,
            angle_jitter: 0.0,
            seed: None,
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

    /// Set random offset for line positions as a fraction of spacing
    /// (e.g., 0.3 = up to 30% of spacing offset in either direction)
    pub fn position_jitter(mut self, jitter: f64) -> Self {
        self.position_jitter = jitter;
        self
    }

    /// Set random offset for line endpoints in drawing units
    pub fn endpoint_jitter(mut self, jitter: f64) -> Self {
        self.endpoint_jitter = jitter;
        self
    }

    /// Set random angle variation for each line in radians
    pub fn angle_jitter(mut self, jitter: f64) -> Self {
        self.angle_jitter = jitter;
        self
    }

    /// Set random angle variation for each line in degrees
    pub fn angle_jitter_deg(mut self, degrees: f64) -> Self {
        self.angle_jitter = degrees.to_radians();
        self
    }

    /// Set random seed for reproducible results
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
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
    use rand::SeedableRng;

    let mut group = Group::new();

    // Extend beyond radius to ensure coverage after rotation
    let extent = radius * HATCH_EXTENT_MULTIPLIER;

    // Create RNG based on seed option
    let mut rng: Box<dyn rand::RngCore> = match options.seed {
        Some(seed) => Box::new(rand::rngs::StdRng::seed_from_u64(seed)),
        None => Box::new(rand::thread_rng()),
    };

    let has_jitter = options.position_jitter > 0.0
        || options.endpoint_jitter > 0.0
        || options.angle_jitter > 0.0;

    // Generate horizontal lines from -extent to +extent
    let mut y = -extent;
    while y <= extent {
        let (y_offset, x1_offset, y1_offset, x2_offset, y2_offset, line_angle) = if has_jitter {
            let pos_jitter = options.position_jitter * options.spacing;
            let end_jitter = options.endpoint_jitter;
            let ang_jitter = options.angle_jitter;

            (
                if pos_jitter > 0.0 {
                    rng.gen_range(-pos_jitter..pos_jitter)
                } else {
                    0.0
                },
                if end_jitter > 0.0 {
                    rng.gen_range(-end_jitter..end_jitter)
                } else {
                    0.0
                },
                if end_jitter > 0.0 {
                    rng.gen_range(-end_jitter..end_jitter)
                } else {
                    0.0
                },
                if end_jitter > 0.0 {
                    rng.gen_range(-end_jitter..end_jitter)
                } else {
                    0.0
                },
                if end_jitter > 0.0 {
                    rng.gen_range(-end_jitter..end_jitter)
                } else {
                    0.0
                },
                if ang_jitter > 0.0 {
                    rng.gen_range(-ang_jitter..ang_jitter)
                } else {
                    0.0
                },
            )
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        };

        // Calculate line endpoints with angle jitter applied
        let line_y = center.y + y + y_offset;
        let (p1, p2) = if line_angle.abs() > 1e-10 {
            // Apply per-line angle rotation around the line's center
            let cos_a = line_angle.cos();
            let sin_a = line_angle.sin();

            // Line endpoints before rotation (centered at line_y)
            let dx = extent;
            let p1 = Point::new(
                center.x - dx * cos_a + x1_offset,
                line_y + dx * sin_a + y1_offset,
            );
            let p2 = Point::new(
                center.x + dx * cos_a + x2_offset,
                line_y - dx * sin_a + y2_offset,
            );
            (p1, p2)
        } else {
            (
                Point::new(center.x - extent + x1_offset, line_y + y1_offset),
                Point::new(center.x + extent + x2_offset, line_y + y2_offset),
            )
        };

        let line = Element::line(p1, p2)
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

    #[test]
    fn test_hatch_with_jitter() {
        let opts = HatchOptions::new()
            .position_jitter(0.3)
            .endpoint_jitter(1.0)
            .seed(42);

        let hatch1 = generate_hatch_lines(Point::new(50.0, 50.0), 40.0, &opts);
        let hatch2 = generate_hatch_lines(Point::new(50.0, 50.0), 40.0, &opts);

        // With the same seed, should produce identical results
        match (&hatch1.shape, &hatch2.shape) {
            (drawing_core::Shape::Group(g1), drawing_core::Shape::Group(g2)) => {
                assert_eq!(g1.children.len(), g2.children.len());
            }
            _ => panic!("Expected Group shapes"),
        }
    }

    #[test]
    fn test_hatch_jitter_options_builder() {
        let opts = HatchOptions::new()
            .position_jitter(0.5)
            .endpoint_jitter(2.0)
            .angle_jitter_deg(5.0)
            .seed(123);

        assert!((opts.position_jitter - 0.5).abs() < 1e-10);
        assert!((opts.endpoint_jitter - 2.0).abs() < 1e-10);
        assert!((opts.angle_jitter - 5.0_f64.to_radians()).abs() < 1e-10);
        assert_eq!(opts.seed, Some(123));
    }
}
