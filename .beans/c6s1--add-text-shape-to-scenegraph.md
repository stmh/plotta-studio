---
# c6s1
title: Add Text shape to scenegraph
status: done
type: task
created_at: 2025-12-25T17:30:48Z
updated_at: 2025-12-25T19:45:00Z
parent: plotta-studio-n2l2
---

Add `Shape::Text(Text)` to the scenegraph with font registry support via `RenderContext`.

## Design Decisions

- **Text as Shape variant**: `Shape::Text(Text)` integrates directly into existing scenegraph
- **Font lookup by name**: Text stores font name string, looked up from RenderContext at render time
- **RenderContext**: New struct holding font registry, passed to all flatten operations
- **Debug flag per-node**: Each Text has a `debug: bool` field for debug visualization
- **Breaking API change**: `flatten()` now requires `&RenderContext` parameter
- **Font types in drawing-core**: Moved Font trait and related types to drawing-core to avoid cyclic dependencies

## Core Types

### Text (in drawing-core)

```rust
pub struct Text {
    pub text: String,
    pub font_name: String,
    pub options: TextOptions,
    pub debug: bool,
}
```

### RenderContext (in drawing-core)

```rust
pub struct RenderContext {
    fonts: HashMap<String, Box<dyn Font>>,
}

impl RenderContext {
    pub fn new() -> Self;
    pub fn register_font(&mut self, name: impl Into<String>, font: Box<dyn Font>);
    pub fn font(&self, name: &str) -> Option<&dyn Font>;
}
```

### RenderContextExt (in sketch-runner)

```rust
pub trait RenderContextExt {
    fn with_hershey_simplex(self) -> Self;
}
```

## API Changes

### Flatten signature

```rust
// Before
impl Element {
    pub fn flatten(&self) -> Vec<Stroke>
}

// After
impl Element {
    pub fn flatten(&self, ctx: &RenderContext) -> Vec<Stroke>
}
```

### Sketch trait

```rust
// Before
pub trait Sketch {
    fn setup(&mut self) -> Drawing;
    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing);
    // ...
}

// After
pub trait Sketch {
    fn setup(&mut self, ctx: &RenderContext) -> Drawing;
    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, ctx: &RenderContext);
    // ...
}
```

### Element constructors

```rust
Element::text("Hello, World!", "Hershey Simplex")
    .text_options(TextOptions::new(12.0).align(TextAlign::Center))
    .text_debug(true)
    .translate(100.0, 100.0)
```

## Architecture

To avoid cyclic dependencies between drawing-core and drawing-text:

- **drawing-core** contains: Font trait, FontMetrics, Glyph, Contour, TextRenderer, RenderContext
- **drawing-text** contains: Font loaders (Hershey, SVG, VSF), re-exports types from drawing-core
- **sketch-runner** contains: RenderContextExt trait for font loading convenience

Dependency graph:
```
drawing-core (font types, Font trait, TextRenderer)
     ↑
drawing-text (font loaders: Hershey, SVG, VSF)
     ↑
sketch-runner (RenderContextExt with font loading)
```

## Flattening Behavior

When `Element::flatten()` encounters `Shape::Text`:

1. Look up font from `ctx.font(text.font_name)`
2. Create `TextRenderer` and call `layout()`
3. Convert layout to strokes via `layout.to_strokes()`
4. If `text.debug` is true, add debug geometry (baselines, bounding boxes)
5. Apply element's transform to all strokes
6. Return combined strokes (debug first, then text - so text renders on top)

If font not found: log warning, return empty strokes.

## File Changes

### New files
- `crates/drawing-core/src/text.rs` - Text struct
- `crates/drawing-core/src/context.rs` - RenderContext struct
- `crates/drawing-core/src/font_types.rs` - Font trait and related types

### Modified files
- `crates/drawing-core/src/shape.rs` - Add `Shape::Text(Text)` variant
- `crates/drawing-core/src/element.rs` - Add constructors, update flatten signature
- `crates/drawing-core/src/drawing.rs` - Update flatten/stroke_count signatures
- `crates/drawing-core/src/lib.rs` - Export new types
- `crates/drawing-text/src/types.rs` - Re-export types from drawing-core
- `crates/drawing-text/src/font.rs` - Re-export Font trait from drawing-core
- `crates/drawing-text/src/lib.rs` - Remove TextRenderer (now in drawing-core)
- `crates/sketch-runner/src/lib.rs` - Add RenderContextExt, update Sketch trait
- `crates/sketch-runner/Cargo.toml` - Add drawing-text dependency
- `crates/drawing-svg/src/lib.rs` - Update export functions to take RenderContext
- `crates/drawing-plotter/src/axidraw.rs` - Update plot functions to take RenderContext
- All sketches - Update to new Sketch trait signatures

## Future Work (not in scope)

- Caching of flattened strokes
- Multithreading support
- Font hot-reloading
