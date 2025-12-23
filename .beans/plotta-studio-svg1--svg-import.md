---
# plotta-studio-svg1
title: SVG import
status: todo
type: epic
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
---

Add SVG import capability to the drawing-svg crate, allowing users to load existing SVG files and convert them to Drawing structures.

## Investigation

### Current State
- `drawing-svg/src/lib.rs` already has SVG export implemented
- TODO comment at line 107-109 suggests using `usvg` for robust parsing
- The crate exports strokes as SVG paths with M, L, Z commands

### Implementation Approach

#### Recommended Library: `usvg`
The `usvg` crate (part of the resvg project) provides:
- Full SVG 1.1/2.0 parsing
- Simplifies complex SVG to basic primitives
- Handles transforms, styles, gradients (converts to simpler forms)
- Already suggested in the TODO comment

#### What Can Be Imported
Since plotta-studio focuses on pen plotter output, import should handle:
- `<path>` elements with M, L, Q, C, Z commands → `Path` / `Polyline`
- `<line>` elements → `Line`
- `<circle>` elements → `Circle`
- `<ellipse>` elements → `Ellipse`
- `<rect>` elements → `Rect`
- `<polygon>` / `<polyline>` elements → `Polyline`
- `<g>` groups with transforms → `Group` with `Transform`

#### What Will Be Ignored (Lossy Import)
- Text (would need font rasterization)
- Gradients and fills (plotter is stroke-only)
- Filters and effects
- Masks and clip paths
- Embedded images

### API Design

```rust
/// Import an SVG file to a Drawing
pub fn import_svg(path: impl AsRef<Path>) -> Result<Drawing, SvgError>;

/// Import SVG from a string
pub fn import_svg_string(svg: &str) -> Result<Drawing, SvgError>;

/// Options for SVG import
pub struct ImportOptions {
    /// Default stroke width if not specified
    pub default_stroke_width: f64,
    /// Default stroke color if not specified
    pub default_stroke_color: Color,
    /// Tolerance for curve flattening (lower = more points)
    pub curve_tolerance: f64,
}
```

### Dependencies to Add
```toml
[dependencies]
usvg = "0.44"  # or latest
```

### Key Implementation Steps
1. Parse SVG with usvg
2. Extract viewBox/dimensions for Drawing size
3. Walk the usvg tree, converting nodes to Elements
4. Handle transforms at each level
5. Convert path segments to PathSegment enum
6. Return Drawing with all elements
