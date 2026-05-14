# Plotta Studio

A Rust workspace for generative drawings and pen plotter output.

Plotta Studio provides reusable libraries — primitives & scene graph,
single-line fonts, SVG import/export, hatching, AxiDraw control — plus
a small CLI and a handful of example sketches that exercise each
library.

## Quick Start

```bash
# Hello-world: centered single-line text
cargo run -p example-hello-world

# Bouncing "DVD" logo
cargo run -p example-dvd-screensaver

# Single-line font showcase (Hershey, VSF, SVG fonts)
cargo run -p example-text

# Hatching + clip groups
cargo run -p example-hatched-circles

# Inverted clipping
cargo run -p example-clip-demo

# SVG import / viewer
cargo run -p example-svg-viewer
```

## Controls

| Key/Action               | Effect                              |
|--------------------------|-------------------------------------|
| Middle mouse drag        | Pan                                 |
| Option + left mouse drag | Pan (macOS trackpad alternative)    |
| Scroll wheel             | Zoom (toward cursor)                |
| Space                    | Fit drawing to window               |
| R                        | Reset view to 1:1                   |
| S                        | Save to `drawing.json`              |
| E                        | Export to SVG                       |
| Escape                   | Quit                                |

## Architecture

```
plotta-studio/
├── crates/
│   ├── drawing-core/      # Primitives, transforms, scene graph, clipping
│   ├── drawing-text/      # Single-line font support (Hershey, VSF, SVG fonts)
│   ├── drawing-svg/       # SVG import/export
│   ├── drawing-plotter/   # AxiDraw plotter control & path optimization
│   ├── drawing-utils/     # Hatching, frames, signature trait
│   ├── sketch-runner/     # Window, rendering, input handling
│   ├── plotta-cli/        # Command-line tool for plotting
│   └── vsf-convert/       # VSF font conversion utility
├── fonts/
│   ├── hershey/           # Classic Hershey stroke fonts
│   ├── svg/               # SVG single-line fonts
│   └── vsf/               # Vector Stroke Font files
└── examples/
    ├── hello-world/
    ├── dvd-screensaver/
    ├── text/
    ├── hatched-circles/
    ├── clip-demo/
    └── svg-viewer/
```

All workspace crates carry `publish = false` — Plotta Studio is not
distributed via crates.io.

## Creating a Sketch

```rust
use sketch_runner::*;

struct MySketch;

impl Sketch for MySketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::a4_landscape();

        drawing.add(Element::circle((100.0, 100.0), 50.0));
        drawing.add(
            Element::rect_centered(drawing.center(), 100.0, 80.0)
                .rotate_deg(45.0)
                .stroke_width(2.0),
        );

        drawing
    }

    fn update(&mut self, drawing: &mut Drawing, ctx: &UpdateContext) -> bool {
        // Called each frame if animate=true.
        // Return true if the drawing changed.
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, ctx: &SketchContext) {
        // Handle keyboard input
    }
}

fn main() {
    run(MySketch);
}
```

## Primitives

- `Element::line(from, to)` — line segment
- `Element::circle(center, radius)` — circle
- `Element::ellipse(center, rx, ry)` — ellipse
- `Element::rect(x, y, w, h)` — rectangle
- `Element::rect_centered(center, w, h)` — centered rectangle
- `Element::arc(center, radius, start, end)` — arc
- `Element::polygon(center, radius, sides)` — regular polygon
- `Element::polyline(points)` — open polyline
- `Element::polygon_from_points(points)` — closed polygon
- `Element::path(path)` — Bezier path
- `Element::group(group)` — nested group
- `Element::clip(shape)` — clip group (clips children to shape)
- `Element::text(text, font)` — text shape

## Transforms

All transforms are chainable:

```rust
Element::circle(Point::ZERO, 50.0)
    .translate(100.0, 100.0)
    .rotate_deg(45.0)
    .rotate_around(angle, center)   // rotate around a specific point
    .scale(2.0, 1.5)
    .stroke_width(2.0)
    .stroke_color(Color::RED)
```

## Groups

Groups allow nested transforms:

```rust
let mut group = Group::new();
group.push(Element::circle(Point::ZERO, 20.0));
group.push(Element::rect_centered(Point::ZERO, 30.0, 30.0));

drawing.add(
    Element::group(group)
        .translate(200.0, 200.0)
        .rotate_deg(30.0),
);
```

## Clipping

Clip groups constrain children to a closed shape:

```rust
let hatch_lines = generate_hatch_lines(center, radius, &HatchOptions::default());
let hatched_circle = Element::clip(Element::circle(center, radius))
    .add(hatch_lines);

drawing.add(hatched_circle);
```

Supports:
- Clipping open strokes (lines, polylines) to polygons
- Clipping closed shapes (polygons intersect with clip region)
- Nested clips (clips compose via intersection)
- Multiple clip shapes (union semantics)

## Bezier Paths

```rust
let path = Path::new()
    .move_to((0.0, 0.0))
    .line_to((50.0, 0.0))
    .quad_to((75.0, 25.0), (75.0, 50.0))                  // quadratic
    .cubic_to((75.0, 75.0), (50.0, 100.0), (0.0, 100.0))  // cubic
    .close();

drawing.add(Element::path(path));
```

Curves are flattened adaptively (default tolerance 0.05 mm) before
rendering or plotting.

## Single-Line Fonts

Plotta Studio includes single-line (stroke) font support — ideal for
pen plotters.

### Formats

- **Hershey** — classic public-domain stroke fonts (8 variants bundled)
- **VSF (Vector Stroke Font)** — JSON format with Bezier support
- **SVG fonts** — SVG-based single-line fonts

### Built-in Hershey fonts

Simplex · Duplex · Triplex (Roman) · Script Simplex · Script Complex
(cursive) · Gothic German · Gothic German Bold · Gothic Italian
(Fraktur).

### Rendering

```rust
use drawing_text::{FontManager, Hershey, TextRenderer, TextOptions, TextAlign};

let manager = FontManager::new();
manager.load_hershey(Hershey::Simplex)?;
// manager.load_from_str(svg_content, FontFormat::SvgFont)?;
// manager.load_file("font.vsf", FontFormat::Vsf)?;

let font = manager.get("Hershey Simplex").unwrap();
let renderer = TextRenderer::new();
let options = TextOptions::new(12.0)            // 12 mm cap height
    .at((100.0, 100.0))
    .align(TextAlign::Center)
    .letter_spacing(0.1);

let layout = renderer.layout("Hello, World!", font, &options);
for stroke in layout.to_strokes(Style::default(), 0.5) {
    drawing.add(Element::from_stroke(stroke));
}
```

### Text in the scene graph

```rust
let text = Text::new("Hello", font.clone())
    .size(24.0)
    .at((100.0, 100.0))
    .align(TextAlign::Center)
    .with_debug(true);   // show baselines, bounding boxes

drawing.add(Element::text(text));
```

## Drawing Utilities

The `drawing-utils` crate provides reusable helpers.

### Hatching

```rust
use drawing_utils::{generate_hatch_lines, HatchOptions};

let options = HatchOptions::new()
    .spacing(2.0)        // 2 mm between lines
    .angle_deg(45.0)
    .stroke_width(0.3);

let hatch = generate_hatch_lines(center, radius, &options);
let hatched = Element::clip(Element::circle(center, radius)).add(hatch);
```

### Frames & Signatures

```rust
use drawing_utils::{draw_frame, draw_frame_with_title, FrameOptions, PlaceholderSignature};

// Plain frame
drawing.add(draw_frame(&drawing, &FrameOptions::default()));

// Frame + title + signature
let opts = FrameOptions::with_default_font(ctx.fonts)
    .expect("default font not loaded")
    .margin_bottom(16.0)
    .with_signature(PlaceholderSignature);   // any Signature impl

drawing.add(draw_frame_with_title(&drawing, "My Drawing", &opts));
```

The signature corner is driven by the `Signature` trait:

```rust
pub trait Signature: Send + Sync {
    /// Render the signature at the requested target height.
    /// Returns the element and its final width in drawing units.
    fn render(&self, target_height: f64) -> (Element, f64);
}
```

Implementing your own signature is a matter of providing a `render`
that emits an `Element` (typically a `Group` of `Path`s) sized to the
requested height. The frame positions the result — no extra scaling
is applied — so the implementation has full control over aspect ratio
and curve fidelity. `PlaceholderSignature` (three small "x" glyphs) is
bundled as a demo.

## SVG Import

```rust
use drawing_svg::{import_svg, import_svg_with_options, ImportOptions, FillBehavior};

let result = import_svg("input.svg")?;
let drawing = result.drawing;

let options = ImportOptions {
    fill_behavior: FillBehavior::ConvertToOutline,   // or Ignore
    default_stroke_width: 1.0,
    default_stroke_color: Color::BLACK,
    import_clip_paths: true,
};
let result = import_svg_with_options("input.svg", &options)?;

for warning in result.warnings {
    println!("Warning: {:?}", warning);
}
```

SVG import is lossy: only path / line data is preserved. Gradients,
filters, raster fills and unsupported text are dropped (with warnings).

## Export

```rust
// JSON — preserves the full scene graph
drawing.save("output.json")?;

// SVG — flattened strokes, plotter-ready
drawing_svg::export_svg(&drawing, "output.svg", &ctx.render)?;
```

## Path Optimization

The plotter module includes stroke optimization for efficient plotting:

- **R\*-tree spatial indexing** — O(n log n) nearest-neighbor queries
- **Stroke reversal** — reverses a stroke when it reduces travel
- **Greedy nearest-neighbor** — minimizes pen-up travel between strokes

For drawings with 100 k+ strokes, optimization typically completes in
under a second.

```rust
use drawing_plotter::{optimize_strokes_with_reversal, total_travel_distance_optimized};

let optimized = optimize_strokes_with_reversal(&strokes, true);
let total = total_travel_distance_optimized(&optimized);
```

## AxiDraw Plotter Control

```rust
use drawing_plotter::{AxiDraw, PlotConfig, plot_in_background};

let mut plotter = AxiDraw::auto_connect()?;
let config = PlotConfig::default();

// Synchronous plot
plotter.plot(&drawing, &config)?;

// Background plot with events
let handle = plot_in_background(drawing, config, None)?;
while handle.is_running() {
    for event in handle.drain_events() {
        println!("{:?}", event);
    }
}
handle.join()?;
```

## Paper Sizes

```rust
Drawing::a4_landscape()   // 297 x 210 mm
Drawing::a4_portrait()    // 210 x 297 mm
Drawing::a3_landscape()   // 420 x 297 mm
Drawing::a3_portrait()    // 297 x 420 mm
Drawing::new(w, h)        // Custom
```

## Command-Line Tool

The `plotta-cli` binary controls a connected AxiDraw:

```bash
# Preview a drawing (stats, estimated time)
cargo run -p plotta-cli -- preview drawing.json

# Plot a drawing
cargo run -p plotta-cli -- plot drawing.json

# Move pen to position
cargo run -p plotta-cli -- move 100 50

# Pen up/down
cargo run -p plotta-cli -- pen up
cargo run -p plotta-cli -- pen down

# Home / status
cargo run -p plotta-cli -- home
cargo run -p plotta-cli -- status
```

## Development

### Build & test

```bash
cargo check --all-targets
cargo test
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

### Conventional Commits & Releases

Commits on `main` follow [Conventional Commits](https://www.conventionalcommits.org/):
`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, …

Releases are cut automatically by [release-please](https://github.com/googleapis/release-please):

1. Every push to `main` opens or updates a single release PR that bumps
   `[workspace.package].version` and appends a `CHANGELOG.md` entry.
2. Merging that PR tags the merge commit as `vX.Y.Z` and creates the
   matching GitHub Release.

No `cargo publish` is performed — all workspace crates are private.

## Roadmap

- [x] Core drawing primitives
- [x] Scene graph with transforms
- [x] GPU rendering with Vello
- [x] Pan/zoom navigation
- [x] JSON serialization
- [x] SVG export
- [x] Path optimization with R\*-tree spatial indexing
- [x] Stroke reversal optimization
- [x] Single-line font support (Hershey, VSF, SVG)
- [x] Text rendering with alignment & spacing
- [x] `ClipGroup` for clipping elements to shapes
- [x] Hatching utilities
- [x] AxiDraw plotter control with motion planning
- [x] SVG import
- [x] Command-line tool (`plotta-cli`)
- [x] `Signature` trait for pluggable corner signatures
- [x] Automated releases via release-please
- [ ] 2-opt path optimization
- [ ] GUI for parameters (egui)

## License

MIT
