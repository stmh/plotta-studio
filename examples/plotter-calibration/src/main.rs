//! Plotter calibration test sheet.
//!
//! Plots a set of patterns that visually expose the kind of positioning
//! errors we have been chasing in `drawing-plotter`:
//!
//! 1. **Absolute positioning grid** — registration crosses at fixed mm
//!    coordinates. Lets you measure mm-accurate offset/scale of the plot.
//!
//! 2. **Concentric closed polygons** — squares, triangles and hexagons of
//!    decreasing size, each drawn as a closed stroke. The end of every
//!    polygon must visually touch its start; a gap means closed-path
//!    endpoint drift.
//!
//! 3. **Joint targets** — pairs of strokes that share an endpoint (pen-up
//!    travel in between). The next stroke must start exactly where the
//!    previous ended; a visible offset between them means SM-based
//!    pen-up travel is losing steps.
//!
//! 4. **Crosshair stress** — many tiny crosshair markers laid out in a
//!    grid. Each marker consists of two short orthogonal strokes that
//!    must meet at the center; misalignment exposes the same small-feature
//!    skew that produced the original "i-dot offset" and "t-crossbar
//!    drift" issues.
//!
//! 5. **Plus-and-dot row** — i-shaped pairs (vertical bar + dot) at
//!    several sizes. The dot must land on the centre line of the bar.
//!
//! 6. **Zigzag chain** — a long zig-zag polyline drawn as a single stroke
//!    of many short segments. Junction-velocity / per-segment rounding
//!    errors accumulate visibly along the chain.
//!
//! 7. **Round-trip return** — strokes that draw a shape, lift, travel to
//!    a far point, lift, travel back and overdraw the same shape. The
//!    second pass must overlay the first.
//!
//! 8. **Small-glyph text ladder** — the original failure mode. The same
//!    sample string is rendered with the default single-line font at
//!    progressively smaller point sizes, plus an overdraw pass at the
//!    smallest size. Watch for i-dots drifting off their stems,
//!    t-crossbars not centered, doubled stems, and the second pass not
//!    overlaying the first.
//!
//! Controls:
//! - E: Export to SVG (built-in)
//! - P: Plot to AxiDraw (built-in, requires `hardware` feature)
//! - Space: Fit to window
//! - Escape: Quit

use std::f64::consts::TAU;

use drawing_text::{TextAlign, TextOptions, TextRenderer, DEFAULT_FONT_NAME};
use drawing_utils::{draw_frame_with_title, FrameOptions, PlaceholderSignature};
use sketch_runner::*;

// Page is A4 portrait so it matches the other examples.
const PAGE_W: f64 = 210.0;
const PAGE_H: f64 = 297.0;

// Frame margins (must match the FrameOptions below). Asymmetric because
// the bottom carries the signature.
const FRAME_LEFT: f64 = 8.0;
const FRAME_TOP: f64 = 8.0;
const FRAME_RIGHT: f64 = 8.0;
const FRAME_BOTTOM: f64 = 16.0;

// Registration-cross inset INSIDE the frame.
const REG_INSET: f64 = 6.0;

// Stroke width used throughout. Keep thin so 0.1 mm offsets are visible.
const STROKE_W: f64 = 0.35;

struct CalibrationSketch;

impl Default for CalibrationSketch {
    fn default() -> Self {
        Self
    }
}

impl Sketch for CalibrationSketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::new(PAGE_W, PAGE_H);
        self.generate(&mut drawing, ctx);
        drawing
    }
}

impl CalibrationSketch {
    fn generate(&self, drawing: &mut Drawing, ctx: &SketchContext) {
        drawing.clear();

        let style = ResolvedStyle::default().with_stroke_width(STROKE_W);

        // ---------------------------------------------------------------
        // 1. Absolute positioning: registration crosses at the four
        //    inset corners + center, with mm coordinates labelled.
        // ---------------------------------------------------------------
        let reg_size = 6.0;
        let reg_left = FRAME_LEFT + REG_INSET;
        let reg_right = PAGE_W - FRAME_RIGHT - REG_INSET;
        let reg_top = FRAME_TOP + REG_INSET;
        let reg_bottom = PAGE_H - FRAME_BOTTOM - REG_INSET;
        let reg_positions = [
            (reg_left, reg_top, "TL"),
            (reg_right, reg_top, "TR"),
            (reg_left, reg_bottom, "BL"),
            (reg_right, reg_bottom, "BR"),
            (PAGE_W / 2.0, PAGE_H / 2.0, "C"),
        ];
        for (x, y, _label) in reg_positions {
            self.add_registration_cross(drawing, x, y, reg_size, style);
        }

        // ---------------------------------------------------------------
        // Section header helper
        // ---------------------------------------------------------------
        let renderer = TextRenderer::new();
        let font = match ctx.fonts.default_font() {
            Some(f) => f,
            None => {
                log::error!("Default font '{}' missing", DEFAULT_FONT_NAME);
                return;
            }
        };

        let label = |drawing: &mut Drawing, text: &str, x: f64, y: f64, size: f64| {
            let opts = TextOptions::new(size).at((x, y)).align(TextAlign::Left);
            let layout = renderer.layout(text, font.clone(), &opts);
            for stroke in layout.to_strokes(style, 0.5) {
                drawing.add(Element::from_stroke(stroke));
            }
        };

        // Content area inside the registration marks.
        let content_left = reg_left + 8.0;
        let content_right = reg_right - 8.0;
        let content_top = reg_top + 8.0;

        // ---------------------------------------------------------------
        // 2. Concentric closed polygons (joint-closure test)
        // ---------------------------------------------------------------
        let mut y = content_top + 4.0;
        label(
            drawing,
            "2. Closed polygons (joints must close)",
            content_left,
            y,
            3.0,
        );
        y += 6.0;

        let group_w = 50.0;
        let cx_square = content_left + group_w / 2.0;
        let cx_tri = content_left + group_w * 1.5;
        let cx_hex = content_left + group_w * 2.5;
        let cy = y + 22.0;

        for r in [22.0, 16.0, 10.0, 6.0, 3.0, 1.5] {
            self.add_regular_polygon(drawing, cx_square, cy, r, 4, style);
            self.add_regular_polygon(drawing, cx_tri, cy, r, 3, style);
            self.add_regular_polygon(drawing, cx_hex, cy, r, 6, style);
        }

        y = cy + 28.0;

        // ---------------------------------------------------------------
        // 3. Joint targets — pairs of strokes that share an endpoint
        //    with a pen-up in between (forces inter-stroke positioning).
        // ---------------------------------------------------------------
        label(
            drawing,
            "3. Pen-up rendezvous (next stroke must touch previous end)",
            content_left,
            y,
            3.0,
        );
        y += 5.0;

        let row_y = y + 5.0;
        for (i, &len) in [12.0, 8.0, 5.0, 3.0, 1.5, 0.8].iter().enumerate() {
            let x0 = content_left + (i as f64) * 22.0;
            let p0 = Point::new(x0, row_y);
            let p1 = Point::new(x0 + len, row_y);
            let p2 = Point::new(x0 + len, row_y + len);
            // Two separate strokes that should connect at p1.
            drawing.add(Element::from_stroke(Stroke::line(p0, p1, style)));
            drawing.add(Element::from_stroke(Stroke::line(p1, p2, style)));
        }
        y = row_y + 16.0;

        // ---------------------------------------------------------------
        // 4. Crosshair grid (tiny-feature alignment)
        // ---------------------------------------------------------------
        label(
            drawing,
            "4. Crosshair grid (arms must meet at center)",
            content_left,
            y,
            3.0,
        );
        y += 5.0;

        let grid_origin_x = content_left;
        let grid_origin_y = y + 4.0;
        let grid_pitch = 7.0;
        let crosshair_arm = 2.0;
        let cols = ((content_right - grid_origin_x) / grid_pitch).floor() as i32;
        let rows = 6;
        for r in 0..rows {
            for c in 0..cols {
                let cx = grid_origin_x + (c as f64) * grid_pitch;
                let cy = grid_origin_y + (r as f64) * grid_pitch;
                // Two separate orthogonal strokes — relies on pen-up move
                // between them to put down the vertical at the right x.
                drawing.add(Element::from_stroke(Stroke::line(
                    Point::new(cx - crosshair_arm, cy),
                    Point::new(cx + crosshair_arm, cy),
                    style,
                )));
                drawing.add(Element::from_stroke(Stroke::line(
                    Point::new(cx, cy - crosshair_arm),
                    Point::new(cx, cy + crosshair_arm),
                    style,
                )));
            }
        }
        y = grid_origin_y + (rows as f64) * grid_pitch + 6.0;

        // ---------------------------------------------------------------
        // 5. Plus-and-dot row — like an "i" letter at various sizes.
        //    The dot must sit centered above the bar. This is the exact
        //    pattern that exposed the original i-dot drift bug.
        // ---------------------------------------------------------------
        label(
            drawing,
            "5. \"i\" pattern (dot must be centered over stem)",
            content_left,
            y,
            3.0,
        );
        y += 5.0;

        let i_row_y = y + 12.0;
        for (i, &size) in [10.0, 7.0, 5.0, 3.5, 2.5, 1.8, 1.2, 0.8].iter().enumerate() {
            let x = content_left + (i as f64) * 16.0;
            let bar_top = i_row_y - size;
            let bar_bot = i_row_y;
            let dot_y = bar_top - size * 0.35;
            // Stem
            drawing.add(Element::from_stroke(Stroke::line(
                Point::new(x, bar_top),
                Point::new(x, bar_bot),
                style,
            )));
            // Dot — drawn as a very short stroke (the smallest features).
            let dot_half = (size * 0.04).max(0.15);
            drawing.add(Element::from_stroke(Stroke::line(
                Point::new(x - dot_half, dot_y),
                Point::new(x + dot_half, dot_y),
                style,
            )));
        }
        y = i_row_y + 8.0;

        // ---------------------------------------------------------------
        // 6. Zigzag chain — single stroke with many short segments.
        //    Tests within-stroke junction-velocity / cumulative rounding.
        // ---------------------------------------------------------------
        label(
            drawing,
            "6. Zigzag chain (single stroke; must stay straight overall)",
            content_left,
            y,
            3.0,
        );
        y += 5.0;

        let zig_y = y + 4.0;
        let zig_amp = 2.0;
        let zig_pitch = 1.5;
        let zig_count = ((content_right - content_left) / zig_pitch) as usize;
        let mut points = Vec::with_capacity(zig_count + 1);
        for i in 0..=zig_count {
            let x = content_left + (i as f64) * zig_pitch;
            let dy = if i % 2 == 0 { 0.0 } else { zig_amp };
            points.push(Point::new(x, zig_y + dy));
        }
        drawing.add(Element::from_stroke(Stroke::new(points, style)));
        y = zig_y + zig_amp + 6.0;

        // Reference straight line directly below the zigzag — any
        // accumulated horizontal drift in the zigzag is visible against
        // this line (which is one single short stroke).
        drawing.add(Element::from_stroke(Stroke::line(
            Point::new(content_left, y),
            Point::new(content_right, y),
            style,
        )));
        y += 6.0;

        // ---------------------------------------------------------------
        // 7. Round-trip return: draw a small shape, travel far away, draw
        //    a second shape, travel back, overdraw the first shape. If
        //    pen-up travel is lossy, the overdraw won't align.
        // ---------------------------------------------------------------
        label(
            drawing,
            "7. Round-trip overdraw (left and right shapes drawn twice)",
            content_left,
            y,
            3.0,
        );
        y += 5.0;

        let rt_y = y + 10.0;
        let left_c = Point::new(content_left + 8.0, rt_y);
        let right_c = Point::new(content_right - 8.0, rt_y);
        for _pass in 0..2 {
            // Small square on the left
            self.add_regular_polygon(drawing, left_c.x, left_c.y, 5.0, 4, style);
            // Travel right; draw small hexagon
            self.add_regular_polygon(drawing, right_c.x, right_c.y, 5.0, 6, style);
            // Tiny triangle near the centre — third destination
            self.add_regular_polygon(drawing, (left_c.x + right_c.x) / 2.0, rt_y, 3.0, 3, style);
        }
        y = rt_y + 8.0;

        // ---------------------------------------------------------------
        // 8. Small-glyph text ladder — reproduces the original failure
        //    mode ("shifted Perspectives"): a sample string rendered at
        //    decreasing sizes, with a special focus on glyphs that
        //    consist of multiple short sub-strokes (i, j, t, colon,
        //    semicolon, period, exclamation, equals, percent). The
        //    smallest line is plotted twice to check pen-up travel
        //    accuracy back to the same baseline.
        // ---------------------------------------------------------------
        label(
            drawing,
            "8. Small text (i-dots, t-crossbars, : ; . ! must align)",
            content_left,
            y,
            3.0,
        );
        y += 5.0;

        // Sample string deliberately rich in detached glyph parts:
        //   - 'i' / 'j' / colon / semicolon / period / exclamation
        //   - 't' / 'f' crossbars
        //   - '=' double bars
        //   - '%' detached circles
        //   - mixed case + digits for general character coverage
        let sample = "Shifted Perspectives: it's a jiffy! i=1; 25%";

        // Render at decreasing point sizes (mm). The bottom rows are the
        // sizes where pen-down settle / SM step rounding issues used to
        // skew small features.
        let sizes: &[f64] = &[6.0, 4.5, 3.5, 2.8, 2.2, 1.8, 1.5];
        for &size in sizes {
            // Use line spacing slightly larger than the cap height so
            // adjacent rows don't overlap even with descenders.
            let baseline = y + size * 0.9;
            let opts = TextOptions::new(size)
                .at((content_left, baseline))
                .align(TextAlign::Left);
            let layout = renderer.layout(sample, font.clone(), &opts);
            for stroke in layout.to_strokes(style, 0.25) {
                drawing.add(Element::from_stroke(stroke));
            }
            // Reference baseline tick at left/right so any vertical drift
            // between rows is measurable against fixed marks.
            let tick = 0.6;
            drawing.add(Element::from_stroke(Stroke::line(
                Point::new(content_left - 2.0, baseline),
                Point::new(content_left - 2.0 + tick, baseline),
                style,
            )));
            y = baseline + size * 0.35;
        }

        // Overdraw the smallest size once more to test pen-up travel
        // accuracy back to exactly the same baseline. The second pass
        // must lie on top of the first, not next to it.
        {
            let size = *sizes.last().expect("sizes is non-empty");
            let baseline = y + size * 0.9;
            let opts = TextOptions::new(size)
                .at((content_left, baseline))
                .align(TextAlign::Left);
            let layout = renderer.layout(
                &format!("{sample}  (overdraw - must overlay above)"),
                font.clone(),
                &opts,
            );
            // Pass 1
            for stroke in layout.to_strokes(style, 0.25) {
                drawing.add(Element::from_stroke(stroke));
            }
            // Pass 2 — identical geometry. After pass 1 the pen is
            // somewhere arbitrary; the optimizer plus pen-up travel must
            // bring it back to draw exactly the same strokes again.
            for stroke in layout.to_strokes(style, 0.25) {
                drawing.add(Element::from_stroke(stroke));
            }
            y = baseline + size * 0.35;
        }

        // Dense isolated-dot row: hardest case for pen-up positioning.
        // Each glyph is essentially one tiny stroke surrounded by
        // pen-up travel. Drift here is visible as uneven spacing.
        {
            let size = 2.0;
            let baseline = y + size * 1.2;
            let opts = TextOptions::new(size)
                .at((content_left, baseline))
                .align(TextAlign::Left);
            let layout = renderer.layout(
                ". . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .",
                font.clone(),
                &opts,
            );
            for stroke in layout.to_strokes(style, 0.25) {
                drawing.add(Element::from_stroke(stroke));
            }
        }

        // ---------------------------------------------------------------
        // Frame with title and signature
        // ---------------------------------------------------------------
        let frame_options = FrameOptions::with_default_font(ctx.fonts)
            .expect("Default font not loaded")
            .margin_left(FRAME_LEFT)
            .margin_top(FRAME_TOP)
            .margin_right(FRAME_RIGHT)
            .margin_bottom(FRAME_BOTTOM)
            .with_signature(PlaceholderSignature);

        drawing.add(draw_frame_with_title(
            drawing,
            "Plotter Calibration",
            &frame_options,
        ));

        log::info!(
            "Calibration sheet: {} strokes",
            drawing.stroke_count(ctx.render)
        );
    }

    /// "+" with a small square around the centre, drawn as 3 separate
    /// strokes so that pen-up moves between them are exercised.
    fn add_registration_cross(
        &self,
        drawing: &mut Drawing,
        cx: f64,
        cy: f64,
        size: f64,
        style: ResolvedStyle,
    ) {
        let half = size / 2.0;
        drawing.add(Element::from_stroke(Stroke::line(
            Point::new(cx - half, cy),
            Point::new(cx + half, cy),
            style,
        )));
        drawing.add(Element::from_stroke(Stroke::line(
            Point::new(cx, cy - half),
            Point::new(cx, cy + half),
            style,
        )));
        let q = size * 0.25;
        let pts = vec![
            Point::new(cx - q, cy - q),
            Point::new(cx + q, cy - q),
            Point::new(cx + q, cy + q),
            Point::new(cx - q, cy + q),
        ];
        drawing.add(Element::from_stroke(Stroke::new(pts, style).closed()));
    }

    /// Regular polygon as a closed stroke, vertex 0 at angle -PI/2 so
    /// squares and hexagons sit "flat-side up".
    fn add_regular_polygon(
        &self,
        drawing: &mut Drawing,
        cx: f64,
        cy: f64,
        r: f64,
        sides: usize,
        style: ResolvedStyle,
    ) {
        if sides < 3 || r <= 0.0 {
            return;
        }
        let mut pts = Vec::with_capacity(sides);
        let phase = -TAU / 4.0;
        for i in 0..sides {
            let a = phase + TAU * (i as f64) / (sides as f64);
            pts.push(Point::new(cx + r * a.cos(), cy + r * a.sin()));
        }
        drawing.add(Element::from_stroke(Stroke::new(pts, style).closed()));
    }
}

fn main() {
    let sketch = CalibrationSketch;

    run_with_config(
        sketch,
        RunnerConfig::new("Plotter Calibration")
            .with_size(900, 1200)
            .with_animation(false),
    );
}
