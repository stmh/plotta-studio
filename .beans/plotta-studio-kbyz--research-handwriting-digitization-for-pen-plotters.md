---
# plotta-studio-kbyz
title: Research handwriting digitization for pen plotters
status: completed
type: task
created_at: 2026-01-01T17:09:52Z
updated_at: 2026-01-01T18:30:00Z
---

Research single-line font formats, conversion tools, glyph variation support, and open source tooling for creating custom plotter handwriting fonts.

## Requirements

1. **Single-line/stroke fonts** (not filled outlines) - essential for plotters
2. **Multiple variants per character** for natural-looking randomization

## Option 1: Extend the VSF Format (Recommended)

The project already has a **VSF (Vector Stroke Font)** format that's perfect for this. It can be extended to support glyph variants:

**Current VSF structure:**
```json
{
  "glyphs": {
    "A": { "unicode": 65, "advance": 600, "contours": [...] }
  }
}
```

**Extended for variants:**
```json
{
  "glyphs": {
    "A": [
      { "unicode": 65, "advance": 600, "contours": [...] },
      { "unicode": 65, "advance": 610, "contours": [...] },
      { "unicode": 65, "advance": 595, "contours": [...] }
    ]
  }
}
```

**Workflow:**
1. Write each letter 3-5 times on paper
2. Scan/photograph
3. Trace in Inkscape using the "Trace Centerline" feature (or manually trace)
4. Export as SVG paths
5. Convert to VSF using `vsf-convert`

## Option 2: OpenType Contextual Alternates

Professional fonts use **OpenType features** for natural variation:
- **`calt`** (Contextual Alternates) - automatic substitution based on context
- **`salt`** (Stylistic Alternates) - user-selectable variants
- **`rand`** (Randomize) - random glyph selection

**Tools:**
- **FontForge** (free) - can create OpenType fonts with alternates
- **Glyphs** (Mac, paid) - professional font editor
- **RoboFont** (Mac, paid) - scriptable font editor

**Challenge:** Most OpenType fonts are outline fonts, not single-line. Would need to create a single-line TTF (rare format).

## Option 3: UFO Format with Variants

The **UFO (Unified Font Object)** format is a directory-based, human-readable format that:
- Supports open contours (single-line strokes)
- Can store multiple layers per glyph
- Uses the `norad` Rust crate for parsing

The project already has a bean for UFO support (`plotta-studio-ah5h`).

**Variant approach:** Store variants in separate UFO layers or as alternate glyphs.

## Option 4: Calligraphr/Handwriting Services

**Calligraphr.com** is a web service that:
1. Provides template sheets you print and fill in
2. Scans and vectorizes your handwriting
3. Generates TTF/OTF fonts

**Limitation:** These are outline fonts, not single-line. Would need to use the centerline/skeleton.

## Option 5: Custom Digitization Workflow

**Manual approach using Inkscape:**

1. **Write samples**: Write each character 3-5 times on graph paper
2. **Scan at high DPI** (300+)
3. **Trace centerlines** in Inkscape:
   - Use "Trace Bitmap" -> Centerline (potrace)
   - Or manually trace with the Bezier tool
4. **Organize as SVG font** or custom JSON format
5. **Convert to VSF** for the project

**Semi-automated approach:**
- Use **Potrace** with centerline mode
- Or **Autotrace** for vector conversion
- Clean up manually in Inkscape

## Option 6: Lingdong's Hershey Font Editor

The [Hershey Font Editor](https://hfedit.glitch.me/) by Lingdong Huang lets you:
- Draw glyphs directly in-browser
- Export as Hershey format (polylines)
- The project already supports Hershey fonts

## Recommended Implementation Plan

### 1. Extend the Font trait

```rust
pub trait Font {
    fn glyph(&self, c: char) -> Option<Glyph>;
    fn glyph_variant(&self, c: char, variant: usize) -> Option<Glyph>;
    fn variant_count(&self, c: char) -> usize;
}
```

### 2. Add randomization to TextOptions

```rust
pub struct TextOptions {
    pub randomize_glyphs: bool,  // Use random variants
    pub seed: Option<u64>,       // For reproducibility
}
```

### 3. Extend VSF format

Support arrays of glyphs per character for variants.

### 4. Update TextRenderer

Randomly select variants during layout when `randomize_glyphs` is enabled.

## Resources

- [p5-single-line-font-resources](https://github.com/golanlevin/p5-single-line-font-resources) - Comprehensive archive of single-line fonts
- [Hershey Font Editor](https://hfedit.glitch.me/) - Browser-based editor
- [Relief SingleLine Font](https://github.com/isdat-type/Relief-SingleLine/) - Open source SVG single-line font
- [Evil Mad Scientist SVG Fonts](https://gitlab.com/oskay/svg-fonts) - Collection of SVG single-line fonts
- [Calligraphr](https://www.calligraphr.com/) - Handwriting to font service
- [Single Line Fonts](https://singlelinefonts.com/) - Commercial single-line fonts

## Next Steps

1. Decide on digitization method (Inkscape manual trace vs Hershey editor)
2. Create variant support in VSF format
3. Update TextRenderer for random variant selection
4. Create a sample handwriting font with 3-5 variants per character
