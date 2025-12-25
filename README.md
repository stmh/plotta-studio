# Plotta Studio

A Rust workspace for generative drawings and pen plotter output.

## Quick Start

```bash
# Run the example sketch
cargo run -p sketch-001

# Create a new sketch (copy sketch-001 as template)
cp -r sketches/sketch-001 sketches/my-sketch
# Edit sketches/my-sketch/Cargo.toml to change the name
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
| E | Export to drawing.svg (in sketch) |
| Escape | Quit |

## Architecture

```
plotta-studio/
├── crates/
│   ├── drawing-core/      # Primitives, transforms, scene graph
│   ├── sketch-runner/     # Window, rendering, input handling
│   ├── drawing-svg/       # SVG import/export
│   └── drawing-plotter/   # AxiDraw control (WIP)
└── sketches/
    └── sketch-001/        # Example sketch
```

## Creating a Sketch

```rust
use sketch_runner::*;

struct MySketch;

impl Sketch for MySketch {
    fn setup(&mut self) -> Drawing {
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

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing) {
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

## Transforms

All transforms are chainable:

```rust
Element::circle(Point::ZERO, 50.0)
    .translate(100.0, 100.0)
    .rotate_deg(45.0)
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

## Export

```rust
// JSON (preserves full scene graph)
drawing.save("output.json")?;

// SVG (flattened strokes)
drawing_svg::export_svg(&drawing, "output.svg")?;
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
- [ ] SVG import
- [ ] AxiDraw plotter control
- [ ] Single line font support
- [ ] GUI for parameters (egui?)
- [ ] Sketch templates with cargo-generate

## License

MIT
