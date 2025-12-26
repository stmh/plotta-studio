---
# plotta-studio-svgf
title: Implement SVG font loader
status: done
type: task
parent: plotta-studio-n2l2
created_at: 2025-12-25T14:30:00Z
updated_at: 2025-12-25T15:00:00Z
---

Implement SVG font file support for drawing-text crate. SVG fonts use standard SVG path syntax and are well-suited for single-line stroke fonts.

## Background

SVG fonts are defined in SVG files using the `<font>` element. While deprecated in browsers, they remain useful for:
- Single-line/stroke fonts for plotters
- Easy creation with vector editors (Inkscape, Illustrator)
- Human-readable XML format with standard SVG path syntax

## SVG Font Format

```xml
<svg xmlns="http://www.w3.org/2000/svg">
  <defs>
    <font id="MyFont" horiz-adv-x="1000">
      <font-face 
        font-family="MyFont"
        units-per-em="1000"
        ascent="800"
        descent="-200"
        cap-height="700"
        x-height="500"
      />
      <missing-glyph horiz-adv-x="500" d="M0 0L500 0L500 700L0 700Z"/>
      <glyph unicode="A" horiz-adv-x="600" d="M0 0L300 700L600 0M100 250L500 250"/>
      <glyph unicode="B" horiz-adv-x="550" d="M50 0L50 700L350 700Q450 700 450 600..."/>
      <!-- kerning pairs -->
      <hkern u1="A" u2="V" k="50"/>
      <hkern u1="T" u2="o" k="40"/>
    </font>
  </defs>
</svg>
```

### Key Elements

- `<font>`: Container with default `horiz-adv-x` (advance width)
- `<font-face>`: Metrics (units-per-em, ascent, descent, etc.)
- `<glyph>`: Individual glyphs with `unicode`, `horiz-adv-x`, and `d` (path data)
- `<hkern>`: Horizontal kerning pairs (`u1`, `u2`, `k`)
- `<missing-glyph>`: Fallback for undefined characters

### Path Data

SVG path `d` attribute uses standard commands:
- `M/m`: Move to
- `L/l`: Line to
- `H/h`: Horizontal line
- `V/v`: Vertical line
- `Q/q`: Quadratic bezier
- `C/c`: Cubic bezier
- `Z/z`: Close path

## Implementation Plan

### 1. Add Dependencies
```toml
# In drawing-text/Cargo.toml
roxmltree = "0.20"  # Lightweight XML parsing
svgtypes = "0.15"   # SVG path parsing
```

### 2. Create SvgFont struct
```rust
pub struct SvgFont {
    name: String,
    glyphs: HashMap<char, Glyph>,
    metrics: FontMetrics,
    kerning: HashMap<(char, char), f64>,
}
```

### 3. Implement SVG Parser
- Parse `<font>` element
- Extract `<font-face>` metrics
- Parse each `<glyph>` element:
  - Get `unicode` attribute (may be entity like `&#65;`)
  - Get `horiz-adv-x` for advance width
  - Parse `d` attribute as SVG path
  - Convert SVG path to our `Contour` type
- Parse `<hkern>` elements for kerning

### 4. SVG Path to Contour Conversion
```rust
fn svg_path_to_contours(d: &str) -> Result<Vec<Contour>, FontError> {
    // Use svgtypes to parse path
    // Convert each subpath to a Contour
    // Handle M, L, H, V, Q, C, Z commands
}
```

### 5. Implement Font Trait
```rust
impl Font for SvgFont {
    fn name(&self) -> &str;
    fn glyph(&self, c: char) -> Option<Glyph>;
    fn kerning(&self, left: char, right: char) -> f64;
    fn metrics(&self) -> FontMetrics;
    fn available_chars(&self) -> Vec<char>;
}
```

### 6. Add Loading Functions
```rust
impl SvgFont {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FontError>;
    pub fn from_str(svg: &str) -> Result<Self, FontError>;
}
```

## Coordinate System Notes

SVG fonts typically use:
- Y-axis pointing up (opposite of screen coordinates)
- Origin at baseline left
- Need to negate Y when converting to our coordinate system

## Test Resources

- Create simple test SVG font with basic characters
- Test with existing open-source SVG fonts
- Verify path parsing with complex glyphs

## Acceptance Criteria

- [x] Parse SVG font files with `<font>` element
- [x] Extract font metrics from `<font-face>`
- [x] Convert glyph paths to Contours
- [x] Support kerning via `<hkern>` elements
- [x] Handle missing glyphs gracefully
- [x] Unit tests for parser
- [x] Integration test with sample SVG font

## References

- [SVG Font specification](https://www.w3.org/TR/SVG11/fonts.html)
- [svgtypes crate](https://docs.rs/svgtypes)
- [roxmltree crate](https://docs.rs/roxmltree)
