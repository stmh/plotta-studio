---
# plotta-studio-svg1
title: SVG import
status: completed
type: epic
priority: normal
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-27T18:06:40Z
---

Add SVG import capability to the drawing-svg crate, allowing users to load existing SVG files and convert them to Drawing structures.

## Design Decisions

Brainstormed on 2025-12-27. Key decisions:

1. **Use case**: General-purpose SVG import (roundtrip, external artwork, library integration)
2. **Fill handling**: Configurable via `FillBehavior` enum, default `ConvertToOutline`
3. **Text handling**: Let usvg convert text to paths using system fonts, warn if fonts missing
4. **Transform handling**: Preserve hierarchy with Group transforms (not baked into coordinates)
5. **Clip-paths**: Map to our `ClipGroup` element (closed shapes only, warn on open paths)
6. **Warnings**: Return `ImportResult { drawing, warnings }` to report skipped elements

## API Design

### Public Functions

```rust
/// Import SVG from file
pub fn import_svg(path: impl AsRef<Path>) -> Result<ImportResult, SvgError>;

/// Import SVG from file with options
pub fn import_svg_with_options(
    path: impl AsRef<Path>, 
    options: &ImportOptions
) -> Result<ImportResult, SvgError>;

/// Import SVG from string
pub fn import_svg_string(svg: &str) -> Result<ImportResult, SvgError>;

/// Import SVG from string with options
pub fn import_svg_string_with_options(
    svg: &str, 
    options: &ImportOptions
) -> Result<ImportResult, SvgError>;
```

### Result Type

```rust
pub struct ImportResult {
    pub drawing: Drawing,
    pub warnings: Vec<ImportWarning>,
}

pub enum ImportWarning {
    UnsupportedElement { element: String, reason: String },
    TextConversionFailed { text: String, reason: String },
    ClipPathSkipped { id: String, reason: String },
    GradientIgnored { id: String },
}
```

### Options

```rust
pub struct ImportOptions {
    /// How to handle filled shapes (default: ConvertToOutline)
    pub fill_behavior: FillBehavior,
    
    /// Default stroke width when not specified (default: 1.0)
    pub default_stroke_width: f64,
    
    /// Default stroke color when not specified (default: BLACK)
    pub default_stroke_color: Color,
    
    /// Tolerance for curve flattening (default: 0.1)
    pub curve_tolerance: f64,
    
    /// Import clip-paths as ClipGroups (default: true)
    pub import_clip_paths: bool,
}

pub enum FillBehavior {
    /// Ignore filled shapes that have no stroke
    Ignore,
    /// Convert fill to outline stroke
    ConvertToOutline,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            fill_behavior: FillBehavior::ConvertToOutline,
            default_stroke_width: 1.0,
            default_stroke_color: Color::BLACK,
            curve_tolerance: 0.1,
            import_clip_paths: true,
        }
    }
}
```

## Element Mapping

| usvg Node | drawing-core Element |
|-----------|---------------------|
| `usvg::Path` | `Element::path()` with PathSegments |
| `usvg::Group` | `Element::group()` with transform |
| `usvg::Group` + clip-path | `Element::clip()` wrapping group |
| `usvg::Text` | Converted to paths by usvg |
| Filled shapes (no stroke) | Outline stroke (if `ConvertToOutline`) |

## Implementation Flow

1. Parse SVG with `usvg::Tree::from_str()` / `from_data()`
2. Extract `tree.size()` for Drawing dimensions
3. Recursively walk `tree.root()` children
4. For each node:
   - Extract transform, apply to `Group`
   - If has clip-path → wrap in `ClipGroup`
   - Convert paths/shapes to `Element`s
   - Extract stroke style (color, width)
5. Collect warnings for skipped elements
6. Return `ImportResult { drawing, warnings }`

## File Structure

```
crates/drawing-svg/src/
├── lib.rs         # Re-exports, existing export code
├── import.rs      # NEW: import functions, ImportOptions, ImportResult
└── convert.rs     # NEW: usvg-to-Element conversion helpers
```

## Dependencies

```toml
[dependencies]
usvg = "0.44"
```

## Child Tasks

- `svg2`: Add usvg dependency, implement core parsing
- `svg3`: Path conversion logic (usvg paths to Element paths)
- `svg4`: Style extraction (stroke color, width, opacity)
- `svg5`: Comprehensive tests (roundtrip, external SVGs, edge cases)
