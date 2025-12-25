//! DVD Screensaver sketch
//!
//! A bouncing "DVD" logo that changes color when it hits the edges.
//!
//! Controls:
//! - Space: Fit drawing to window
//! - R: Reset view
//! - Escape: Quit

use sketch_runner::*;

/// Style parameters for drawing letters
struct LetterStyle {
    stroke: f64,
    color: Color,
}

struct DvdSketch {
    /// Position of the logo center
    pos: Point,
    /// Velocity (pixels per second)
    vel: Point,
    /// Current color hue (0-360)
    hue: f64,
    /// Logo width
    logo_width: f64,
    /// Letter height (text only)
    letter_height: f64,
    /// Gap between letters and disc
    disc_gap: f64,
    /// Disc height
    disc_height: f64,
}

impl DvdSketch {
    /// Total height of the logo (letters + gap + disc)
    fn total_height(&self) -> f64 {
        self.letter_height + self.disc_gap + self.disc_height
    }
}

impl Default for DvdSketch {
    fn default() -> Self {
        Self {
            pos: Point::new(100.0, 80.0),
            vel: Point::new(80.0, 60.0),
            hue: 0.0,
            logo_width: 60.0,
            letter_height: 30.0,
            disc_gap: 5.0,
            disc_height: 8.0,
        }
    }
}

impl Sketch for DvdSketch {
    fn setup(&mut self) -> Drawing {
        Drawing::a4_landscape().with_background(Color::gray(20))
    }

    fn update(&mut self, drawing: &mut Drawing, ctx: &UpdateContext) -> bool {
        // Update position
        self.pos.x += self.vel.x * ctx.delta;
        self.pos.y += self.vel.y * ctx.delta;

        // Bounce off edges
        let half_w = self.logo_width / 2.0;
        let half_h = self.total_height() / 2.0;
        let mut bounced = false;

        // Left/right bounce
        if self.pos.x - half_w <= 0.0 {
            self.pos.x = half_w;
            self.vel.x = self.vel.x.abs();
            bounced = true;
        } else if self.pos.x + half_w >= drawing.width {
            self.pos.x = drawing.width - half_w;
            self.vel.x = -self.vel.x.abs();
            bounced = true;
        }

        // Top/bottom bounce
        if self.pos.y - half_h <= 0.0 {
            self.pos.y = half_h;
            self.vel.y = self.vel.y.abs();
            bounced = true;
        } else if self.pos.y + half_h >= drawing.height {
            self.pos.y = drawing.height - half_h;
            self.vel.y = -self.vel.y.abs();
            bounced = true;
        }

        // Change color on bounce
        if bounced {
            self.hue = (self.hue + 45.0) % 360.0;
        }

        // Redraw
        self.draw(drawing);
        true
    }
}

impl DvdSketch {
    fn draw(&self, drawing: &mut Drawing) {
        drawing.clear();

        let style = LetterStyle {
            stroke: 2.5,
            color: Color::hsl(self.hue, 1.0, 0.5),
        };

        // Draw "DVD" text as simple geometric shapes
        let x = self.pos.x - self.logo_width / 2.0;
        let y = self.pos.y - self.total_height() / 2.0;
        let letter_width = self.logo_width / 4.0;

        // Letter D (first)
        draw_letter_d(drawing, x, y, letter_width, self.letter_height, &style);

        // Letter V
        draw_letter_v(
            drawing,
            x + letter_width * 1.2,
            y,
            letter_width,
            self.letter_height,
            &style,
        );

        // Letter D (second)
        draw_letter_d(
            drawing,
            x + letter_width * 2.4,
            y,
            letter_width,
            self.letter_height,
            &style,
        );

        // Disc shape under the text
        let disc_y = y + self.letter_height + self.disc_gap;
        let disc_width = self.logo_width * 0.8;

        drawing.add(
            Element::ellipse(
                (self.pos.x, disc_y + self.disc_height / 2.0),
                disc_width / 2.0,
                self.disc_height / 2.0,
            )
            .stroke_width(style.stroke)
            .stroke_color(style.color),
        );
    }
}

fn draw_letter_d(drawing: &mut Drawing, x: f64, y: f64, w: f64, h: f64, style: &LetterStyle) {
    // Vertical line
    drawing.add(
        Element::line((x, y), (x, y + h))
            .stroke_width(style.stroke)
            .stroke_color(style.color),
    );

    // Curved part of D using cubic bezier
    // Approximate a half-ellipse with two cubic bezier curves
    // Magic number for circle approximation: 0.552284749831
    let k = 0.552284749831;
    let rx = w; // horizontal radius
    let ry = h / 2.0; // vertical radius
    let cx = x; // center x (left edge)
    let cy = y + h / 2.0; // center y (middle)

    let path = Path::new()
        // Start at top
        .move_to((cx, cy - ry))
        // Top-right quadrant
        .cubic_to(
            (cx + rx * k, cy - ry), // control point 1
            (cx + rx, cy - ry * k), // control point 2
            (cx + rx, cy),          // end at right middle
        )
        // Bottom-right quadrant
        .cubic_to(
            (cx + rx, cy + ry * k), // control point 1
            (cx + rx * k, cy + ry), // control point 2
            (cx, cy + ry),          // end at bottom
        );

    drawing.add(
        Element::path(path)
            .stroke_width(style.stroke)
            .stroke_color(style.color),
    );
}

fn draw_letter_v(drawing: &mut Drawing, x: f64, y: f64, w: f64, h: f64, style: &LetterStyle) {
    // Left diagonal
    drawing.add(
        Element::line((x, y), (x + w / 2.0, y + h))
            .stroke_width(style.stroke)
            .stroke_color(style.color),
    );

    // Right diagonal
    drawing.add(
        Element::line((x + w, y), (x + w / 2.0, y + h))
            .stroke_width(style.stroke)
            .stroke_color(style.color),
    );
}

fn main() {
    let sketch = DvdSketch::default();

    run_with_config(
        sketch,
        RunnerConfig::new("DVD Screensaver")
            .with_size(1200, 800)
            .with_animation(true),
    );
}
