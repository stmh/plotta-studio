# Plotta Studio

A Rust workspace for generative drawings and pen plotter output.

## Quick Start

```bash
# Run an example sketch
cargo run -p sketch-001-radial

# Run the text rendering demo
cargo run -p sketch-003-text

# Run the hatched circles demo (shows clipping)
cargo run -p sketch-004-hatched-circles
```

## Controls

| Key/Action | Effect |
|------------|--------|
| Middle mouse drag | Pan |
| Option + left mouse drag | Pan (macOS trackpad alternative) |
| Scroll wheel | Zoom (toward cursor) |
| Space | Fit drawing to window |
| R | Reset view to 1:1 |
| S | Save to drawing.json |
| E | Export to SVG |
| Escape | Quit |

## Architecture

```
plotta-studio/
├── crates/
│   ├── drawing-core/      # Primitives, transforms, scene graph, clipping
│   ├── drawing-text/      # Single-line font support (Hershey, VSF, SVG fonts)
│   ├── drawing-svg/       # SVG export
│   ├── drawing-plotter/   # AxiDraw plotter control & path optimization
│   ├── drawing-utils/     # Hatching, frames, and other utilities
│   └── sketch-runner/     # Window, rendering, input handling
├── fonts/
│   ├── hershey/           # Classic Hershey stroke fonts
│   ├── svg/               # SVG single-line fonts
│   └── vsf/               # Vector Stroke Font files
└── sketches/
    ├── sketch-001-radial/
    ├── sketch-002-dvd-screensaver/
    ├── sketch-003-text/
    └── sketch-004-hatched-circles/
```

## Creating a Sketch

```rust
use sketch_runner::*;

struct MySketch;

impl Sketch for MySketch {
    fn setup(&mut self, ctx: &SketchContext) -> Drawing {
        let mut drawing = Drawing::a4_landscape();
        
        // Add elements
        drawing.add(Element::circle((100.0, 100.0), 50.0));
        drawing.add(
            Element::rect_centered(drawing.center(), 100.0, 80.0)
                .rotate_deg(45.0)
                .stroke_width(2.0)
        );
        
        drawing
    }

    fn update(&mut self, drawing: &mut Drawing, ctx: &UpdateContext) -> bool {
        // Called each frame if animate=true
        // Return true if drawing changed
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

- `Element::line(from, to)` - Line segment
- `Element::circle(center, radius)` - Circle
- `Element::ellipse(center, rx, ry)` - Ellipse
- `Element::rect(x, y, w, h)` - Rectangle
- `Element::rect_centered(center, w, h)` - Centered rectangle
- `Element::arc(center, radius, start, end)` - Arc
- `Element::polygon(center, radius, sides)` - Regular polygon
- `Element::polyline(points)` - Open polyline
- `Element::polygon_from_points(points)` - Closed polygon
- `Element::path(path)` - Bezier path
- `Element::group(group)` - Nested group
- `Element::clip(shape)` - Clip group (clips children to shape)
- `Element::text(text, font)` - Text shape

## Transforms

All transforms are chainable:

```rust
Element::circle(Point::ZERO, 50.0)
    .translate(100.0, 100.0)
    .rotate_deg(45.0)
    .rotate_around(angle, center)  // Rotate around a specific point
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
        .rotate_deg(30.0)
);
```

## Clipping

Clip groups constrain children to a closed shape:

```rust
// Create hatch lines clipped to a circle
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
    .quad_to((75.0, 25.0), (75.0, 50.0))  // Quadratic bezier
    .cubic_to((75.0, 75.0), (50.0, 100.0), (0.0, 100.0))  // Cubic bezier
    .close();

drawing.add(Element::path(path));
```

## Single-Line Fonts

Plotta Studio includes comprehensive support for single-line (stroke) fonts, ideal for pen plotters:

### Font Formats

- **Hershey fonts** - Classic public domain stroke fonts (8 variants included)
- **VSF (Vector Stroke Font)** - Modern JSON format with bezier support
- **SVG fonts** - SVG-based single-line fonts

### Built-in Hershey Fonts

- Simplex, Duplex, Triplex (Roman)
- Script Simplex, Script Complex (Cursive)
- Gothic German, Gothic German Bold, Gothic Italian (Fraktur)

### Text Rendering

```rust
use drawing_text::{FontManager, Hershey, TextRenderer, TextOptions, TextAlign};

// Load fonts
let manager = FontManager::new();
manager.load_hershey(Hershey::Simplex)?;

// Or load from string/file
manager.load_from_str(svg_content, FontFormat::SvgFont)?;
manager.load_file("font.vsf", FontFormat::Vsf)?;

// Render text
let font = manager.get("Hershey Simplex").unwrap();
let renderer = TextRenderer::new();
let options = TextOptions::new(12.0)  // 12mm height
    .at((100.0, 100.0))
    .align(TextAlign::Center)
    .letter_spacing(0.1);

let layout = renderer.layout("Hello, World!", font, &options);
let strokes = layout.to_strokes(Style::default(), 0.5);

for stroke in strokes {
    drawing.add(Element::from_stroke(stroke));
}
```

### Text Element (Scene Graph)

```rust
// Using Text shape directly in the scene graph
let text = Text::new("Hello", font.clone())
    .size(24.0)
    .at((100.0, 100.0))
    .align(TextAlign::Center)
    .with_debug(true);  // Show baselines, bounding boxes

drawing.add(Element::text(text));
```

## Drawing Utilities

The `drawing-utils` crate provides reusable drawing helpers:

### Hatching

```rust
use drawing_utils::{generate_hatch_lines, HatchOptions};

let options = HatchOptions::new()
    .spacing(2.0)      // 2mm between lines
    .angle_deg(45.0)   // 45 degree rotation
    .stroke_width(0.3);

// Generate hatch lines for a circular area
let hatch = generate_hatch_lines(center, radius, &options);

// Clip to desired shape
let hatched = Element::clip(Element::circle(center, radius))
    .add(hatch);
```

### Frames

```rust
use drawing_utils::{draw_frame, draw_frame_with_title, FrameOptions};

// Simple frame
drawing.add(draw_frame(&drawing, &FrameOptions::default()));

// Frame with title (requires a font)
drawing.add(draw_frame_with_title(
    &drawing,
    "My Drawing",
    &FrameOptions::new(font),
));
```

## Export

```rust
// JSON (preserves full scene graph)
drawing.save("output.json")?;

// SVG (flattened strokes, plotter-ready)
drawing_svg::export_svg(&drawing, "output.svg", &ctx.render)?;
```

## Path Optimization

The plotter module includes stroke optimization for efficient plotting:

```rust
use drawing_plotter::{optimize_strokes, total_travel_distance, pen_down_distance};

// Optimize stroke order to minimize pen-up travel
let optimized = optimize_strokes(&strokes);

// Calculate distances
let total = total_travel_distance(&optimized);
let drawing_dist = pen_down_distance(&optimized);
let travel_dist = total - drawing_dist;
```

## AxiDraw Plotter Control

```rust
use drawing_plotter::{AxiDraw, PlotConfig, plot_in_background};

// Auto-connect to plotter
let mut plotter = AxiDraw::auto_connect()?;

// Configure plotting
let config = PlotConfig::default();

// Plot synchronously
plotter.plot(&drawing, &config)?;

// Or plot in background with events
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
Drawing::a4_landscape()  // 297 x 210 mm
Drawing::a4_portrait()   // 210 x 297 mm
Drawing::a3_landscape()  // 420 x 297 mm
Drawing::a3_portrait()   // 297 x 420 mm
Drawing::new(w, h)       // Custom size
```

## Roadmap

- [x] Core drawing primitives
- [x] Scene graph with transforms
- [x] GPU rendering with Vello
- [x] Pan/zoom navigation
- [x] JSON serialization
- [x] SVG export
- [x] Path optimization for plotting (greedy nearest-neighbor)
- [x] Single-line font support (Hershey, VSF, SVG fonts)
- [x] Text rendering with alignment and spacing
- [x] ClipGroup for clipping elements to shapes
- [x] Hatching utilities
- [x] AxiDraw plotter control
- [ ] SVG import
- [ ] 2-opt path optimization
- [ ] Stroke reversal optimization
- [ ] GUI for parameters (egui)
- [ ] Sketch templates with cargo-generate

## License

MIT
