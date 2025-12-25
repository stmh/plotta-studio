---
# plotta-studio-n2l2
title: Investigate Single Line Font (VSF) support
status: done
type: epic
created_at: 2025-12-23T19:14:41Z
updated_at: 2025-12-25T00:00:00Z
---

Research and plan Single Line Font support for plotta-studio. Vector Stroke Fonts (VSF) are essential for pen plotters as they draw text with single strokes rather than filled outlines.

## Implementation Complete

The `drawing-text` crate has been implemented with:

### Completed Features
- **Core types**: `Font` trait, `Glyph`, `Contour`, `FontMetrics`, `TextOptions`
- **Hershey font parser**: JHF format support with public domain Simplex font
- **VSF format**: JSON-based format with bezier curve support and kerning
- **TextRenderer**: Layout engine with alignment (left/center/right), letter spacing, word spacing, and multiline support
- **Contour conversion**: `to_path()`, `to_stroke()`, `flatten()` methods
- **Integration helpers**: `Element::from_stroke()`, `Style::with_stroke_width/color()`

### Files Added
- `crates/drawing-text/` - New crate with all font functionality
- `fonts/hershey/simplex.jhf` - Public domain Hershey Simplex Roman font
- `fonts/hershey/LICENSE` - Public domain notice
- `sketches/sketch-003-text/` - Demo sketch showing text rendering

### Font Units
Font sizes are in drawing units (millimeters for A4). Example: `TextOptions::new(12.0)` = 12mm tall text.

### Remaining Work (Future)
- UFO font loader (using `norad` crate)
- Additional Hershey font variants (Gothic, Script, etc.)
- `vsf-convert` CLI tool for font conversion
- FontManager for managing multiple fonts

## Background

Single Line Fonts (also called stroke fonts, stick fonts, engraving fonts, or monoline fonts) are designed for applications where text is drawn as paths rather than filled shapes. This is ideal for:
- Pen plotters (AxiDraw, etc.)
- Laser engraving
- CNC routing
- Any application needing text as vector strokes

## Research Findings

### Single Line Font Formats Overview

| Format | Origin | Encoding | Curves | Pros | Cons |
|--------|--------|----------|--------|------|------|
| Hershey | 1967, Naval Weapons Lab | ASCII offset from 'R' | Lines only | Simple, public domain, many fonts | No bezier curves, dated encoding |
| UFO 3 | Font community | XML/plist | Cubic/Quadratic Bezier | Modern, rich metadata, kurbo compatible | Complex structure, large files |
| SVG Font | W3C | XML | Bezier curves | Standard SVG path syntax | Deprecated in browsers |
| OpenType-SVG | Adobe/Mozilla | TTF + SVG | Bezier curves | Works in Adobe apps | Complex, closed contour workaround |
| JHF | Kamal Mostafa | Text-based | Lines only | Standard Hershey distribution | Same limitations as Hershey |
| minf | Golan Levin | Base64 | Lines only | Ultra-compact (72 bytes) | Very limited, minimal charset |

### Hershey Font Format (Deep Dive)

The Hershey fonts were developed c. 1967 by Dr. Allen Vincent Hershey at the Naval Weapons Laboratory. They remain public domain and are widely used.

**Format Structure:**
```
NNNNN CC L R <coordinate pairs...>
```
- `NNNNN`: 5-digit glyph number (1-4000+)
- `CC`: Character count (number of coordinate pairs)
- `L`: Left margin (x offset from 'R')
- `R`: Right margin (x offset from 'R')
- Coordinate pairs: ASCII characters offset from 'R' (ASCII 82)

**Coordinate Encoding:**
- Each coordinate is a single ASCII character
- Value = `char.charCodeAt(0) - 'R'.charCodeAt(0)`
- Example: 'S' = +1, 'L' = -6, 'T' = +2, 'J' = -8
- 'RR' is the origin (0,0)
- ' R' (space + R) = pen up / move without drawing

**Font Variants:**
- Simplex: Minimal strokes, fastest to draw
- Duplex: Two strokes per line
- Complex: More detail, slower
- Triplex: Maximum detail, slowest

**Character Sets Available:**
- Latin (Roman, Gothic, Script, Italic)
- Greek
- Cyrillic
- Japanese (Kanji, Hiragana, Katakana)
- Mathematical symbols
- Musical notation
- Meteorological symbols

**Resources:**
- [p5-hershey-js](https://github.com/LingDong-/p5-hershey-js) - Authoritative p5.js implementation
- [hershey-fonts](https://github.com/kamalmostafa/hershey-fonts) - C library and .jhf files
- [p5-single-line-font-resources](https://github.com/golanlevin/p5-single-line-font-resources) - Comprehensive collection

### UFO 3 Format (Deep Dive)

The Unified Font Object (UFO) is a cross-platform, human-readable font format.

**Directory Structure:**
```
myfont.ufo/
├── metainfo.plist      # Format version
├── fontinfo.plist      # Font metadata (family, weight, metrics)
├── glyphs/             # Glyph files
│   ├── contents.plist  # Glyph name → filename mapping
│   ├── A_.glif         # Glyph 'A'
│   └── ...
├── kerning.plist       # Kerning pairs
├── groups.plist        # Glyph groups
└── lib.plist           # Custom data
```

**GLIF Format (Glyph Interchange Format):**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<glyph name="A" format="2">
  <advance width="600"/>
  <unicode hex="0041"/>
  <outline>
    <contour>
      <point x="300" y="0" type="move"/>
      <point x="50" y="700" type="line"/>
    </contour>
    <contour>
      <point x="300" y="0" type="move"/>
      <point x="550" y="700" type="line"/>
    </contour>
    <contour>
      <point x="150" y="250" type="move"/>
      <point x="450" y="250" type="line"/>
    </contour>
  </outline>
</glyph>
```

**Point Types:**
- `move`: First point of open contour (pen up, move to)
- `line`: Draw line from previous point
- `curve`: Cubic bezier (preceded by 0-2 offcurve points)
- `qcurve`: Quadratic bezier (TrueType style)
- `offcurve`: Control point for curves

**Key for single-line fonts:**
- Open contours start with `type="move"`
- Closed contours do NOT start with move (cyclic)
- UFO supports open contours, OpenType does NOT

**Rust Support:** `norad` crate (v0.17+, from Linebender)

### Processing Community Resources

The creative coding community (Processing, p5.js) has developed extensive single-line font tooling:

**Libraries:**
- [p5-hershey-js](https://github.com/LingDong-/p5-hershey-js) - Full Hershey font support with 20+ character sets
- [hersheytext.js](https://github.com/techninja/hersheytextjs) - SVG path output for Hershey fonts
- [opentype.js](https://opentype.js.org/) - TTF/OTF parsing with glyph path access

**Tools:**
- [Hershey Font Editor](https://lingdong-.github.io/p5-hershey-js/editor/) - Interactive glyph editor
- [cnc-text-tool](https://github.com/jvolker/single-line-font-renderer) - Browser-based SVG export
- [Inkscape Hershey Text Extension](https://wiki.evilmadscientist.com/Hershey_Text) - SVG integration

**Font Collections:**
- [singlelinefonts.com](https://singlelinefonts.com) - Commercial foundry
- [onelinefonts.com](https://onelinefonts.com) - Commercial foundry
- [Relief SingleLine](https://github.com/isdat-type/Relief-SingleLine) - Open-source OpenType-SVG

### Rust Ecosystem

| Crate | Purpose | Notes |
|-------|---------|-------|
| `norad` | UFO 3 parsing | From Linebender, kurbo compatible |
| `fontdue` | TTF/OTF rasterization | Not suitable for stroke fonts (raster output) |
| `font-kit` | Font loading | Cross-platform, outline access |
| `rusttype` | TTF/OTF parsing | Glyph shape access |
| `fonterator` | Path-based rendering | Outputs path ops, not strokes |
| `opencv` | Hershey fonts | Via OpenCV bindings, heavyweight |

**Recommendation:** Use `norad` for UFO fonts + custom Hershey parser

## Proposed Architecture

### Abstract Font Renderer Design

```
┌─────────────────────────────────────────────────────────────────┐
│                        FontManager                              │
│  - load_font(source) → FontHandle                              │
│  - get_font(name) → Option<&Font>                              │
│  - list_fonts() → Vec<FontInfo>                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Font (trait object)                         │
│  - name() → &str                                                │
│  - glyph(char) → Option<Glyph>                                 │
│  - kerning(left, right) → f64                                  │
│  - metrics() → FontMetrics                                     │
│  - has_glyph(char) → bool                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
     ┌────────────┬───────────┼───────────┬────────────┐
     ▼            ▼           ▼           ▼            ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────────┐
│ Hershey │ │   Ufo   │ │   Vsf   │ │   Svg   │ │  Custom/  │
│  Font   │ │  Font   │ │  Font   │ │  Font   │ │ Computed  │
│ (lines) │ │ (bezier)│ │ (JSON)  │ │ (bezier)│ │  (future) │
└─────────┘ └─────────┘ └─────────┘ └─────────┘ └───────────┘
     │            │           │           │            │
     └────────────┴───────────┴───────────┴────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Glyph                                   │
│  - contours: Vec<Contour>                                      │
│  - advance_width: f64                                          │
│  - bounds: Rect                                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Contour                                  │
│  - to_path() → Path           (drawing-core Path)              │
│  - to_bezpath() → BezPath     (kurbo BezPath)                  │
│  - flatten(tolerance) → Vec<Point>                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     TextRenderer                                │
│  - render(text, font, options) → Vec<Element>                  │
│  - layout(text, font, options) → TextLayout                    │
│  - measure(text, font, options) → Rect                         │
└─────────────────────────────────────────────────────────────────┘
```

### Core Types

```rust
/// Font metrics for layout calculations
pub struct FontMetrics {
    pub units_per_em: f64,
    pub ascender: f64,
    pub descender: f64,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
    pub line_gap: f64,
}

/// A single glyph from a font
pub struct Glyph {
    pub unicode: char,
    pub name: Option<String>,
    pub advance_width: f64,
    pub contours: Vec<Contour>,
    pub bounds: Rect,
}

/// A contour (open or closed path) within a glyph
pub struct Contour {
    segments: Vec<ContourSegment>,
    closed: bool,
}

pub enum ContourSegment {
    MoveTo(Point),
    LineTo(Point),
    QuadTo { ctrl: Point, to: Point },
    CubicTo { ctrl1: Point, ctrl2: Point, to: Point },
}

/// Text rendering options
pub struct TextOptions {
    pub size: f64,                    // Font size in drawing units
    pub position: Point,              // Baseline start position
    pub align: TextAlign,             // Left, Center, Right
    pub line_height: Option<f64>,     // Override line spacing
    pub letter_spacing: f64,          // Additional spacing between glyphs
    pub word_spacing: f64,            // Additional spacing between words
}

pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Result of text layout
pub struct TextLayout {
    pub glyphs: Vec<PositionedGlyph>,
    pub bounds: Rect,
    pub line_count: usize,
}

pub struct PositionedGlyph {
    pub glyph: Glyph,
    pub position: Point,
    pub transform: Affine,
}
```

### Font Trait

```rust
pub trait Font: Send + Sync {
    /// Font family name
    fn name(&self) -> &str;

    /// Get a glyph by unicode character
    fn glyph(&self, c: char) -> Option<Glyph>;

    /// Get kerning adjustment between two characters
    fn kerning(&self, left: char, right: char) -> f64;

    /// Get font metrics
    fn metrics(&self) -> FontMetrics;

    /// Check if font has a glyph for character
    fn has_glyph(&self, c: char) -> bool {
        self.glyph(c).is_some()
    }

    /// Get all available characters
    fn available_chars(&self) -> Vec<char>;
}
```

### Font Loaders (Pluggable)

```rust
pub trait FontLoader: Send + Sync {
    /// Check if this loader can handle the given source
    fn can_load(&self, source: &FontSource) -> bool;

    /// Load a font from the source
    fn load(&self, source: &FontSource) -> Result<Box<dyn Font>, FontError>;

    /// Supported format name (for error messages)
    fn format_name(&self) -> &'static str;
}

pub enum FontSource {
    File(PathBuf),
    Bytes { data: Vec<u8>, format: FontFormat },
    Url(String),
}

pub enum FontFormat {
    Hershey,    // .jhf or inline Hershey data
    Ufo,        // .ufo directory
    SvgFont,    // .svg font file
    Vsf,        // .vsf (our custom format)
}
```

### Integration with drawing-core

```rust
// New Shape variant
pub enum Shape {
    // ... existing variants ...
    Text(Text),
}

pub struct Text {
    pub content: String,
    pub font_name: String,
    pub options: TextOptions,
}

impl Text {
    pub fn flatten(&self, font_manager: &FontManager) -> Vec<Stroke> {
        let font = font_manager.get_font(&self.font_name)?;
        let renderer = TextRenderer::new();
        let elements = renderer.render(&self.content, font, &self.options);
        elements.iter().flat_map(|e| e.flatten()).collect()
    }
}
```

## Proposed VSF (Vector Stroke Font) File Format

A simple, JSON-based format optimized for single-line/stroke fonts:

```json
{
  "version": "1.0",
  "name": "MySingleLineFont",
  "metadata": {
    "author": "Font Author",
    "license": "CC0",
    "description": "A simple single-line font"
  },
  "metrics": {
    "units_per_em": 1000,
    "ascender": 800,
    "descender": -200,
    "x_height": 500,
    "cap_height": 700,
    "line_gap": 100
  },
  "glyphs": {
    "A": {
      "unicode": 65,
      "advance": 600,
      "contours": [
        {
          "closed": false,
          "points": [
            {"x": 0, "y": 0, "type": "move"},
            {"x": 300, "y": 700, "type": "line"},
            {"x": 600, "y": 0, "type": "line"}
          ]
        },
        {
          "closed": false,
          "points": [
            {"x": 100, "y": 233, "type": "move"},
            {"x": 500, "y": 233, "type": "line"}
          ]
        }
      ]
    },
    "B": {
      "unicode": 66,
      "advance": 600,
      "contours": [
        {
          "closed": false,
          "points": [
            {"x": 50, "y": 0, "type": "move"},
            {"x": 50, "y": 700, "type": "line"},
            {"x": 350, "y": 700, "type": "line"},
            {"x": 450, "y": 650, "type": "quad", "ctrl": [450, 700]},
            {"x": 450, "y": 550, "type": "quad", "ctrl": [450, 600]},
            {"x": 350, "y": 500, "type": "quad", "ctrl": [450, 500]},
            {"x": 50, "y": 500, "type": "line"}
          ]
        }
      ]
    }
  },
  "kerning": {
    "AV": -50,
    "AW": -30,
    "VA": -50,
    "WA": -30,
    "To": -40
  }
}
```

**Format Features:**
- JSON for human readability and easy parsing
- Supports bezier curves (quad/cubic) unlike Hershey
- Optional kerning pairs
- Full font metrics
- Compact representation
- Easy to create programmatically

**Alternative: Binary Format**
For larger fonts, a binary format could be used:
```
Header (32 bytes):
  - Magic: "VSF1" (4 bytes)
  - Version: u16 (2 bytes)
  - Flags: u16 (2 bytes)
  - Glyph count: u32 (4 bytes)
  - Kerning pair count: u32 (4 bytes)
  - Metrics offset: u32 (4 bytes)
  - Glyphs offset: u32 (4 bytes)
  - Kerning offset: u32 (4 bytes)
  - Reserved (4 bytes)

Metrics (32 bytes):
  - units_per_em: f32
  - ascender: f32
  - descender: f32
  - x_height: f32
  - cap_height: f32
  - line_gap: f32
  - Reserved (8 bytes)

Glyph Table:
  - Per glyph entry (variable):
    - Unicode: u32
    - Advance: f32
    - Contour count: u16
    - Contours (variable)

Contour:
  - Point count: u16
  - Closed: u8
  - Points (variable): x: f32, y: f32, type: u8, [ctrl1, ctrl2]
```

## Implementation Plan

### Phase 1: Core Font Types
1. Create `drawing-text` crate
2. Define `Font`, `Glyph`, `Contour` types
3. Implement `Contour` → `Path` → `Stroke` conversion
4. Add unit tests

### Phase 2: Hershey Font Loader
1. Implement Hershey format parser
2. Create `HersheyFont` implementing `Font` trait
3. Bundle common Hershey fonts (simplex, complex, etc.)
4. Test with p5-hershey-js data

### Phase 3: UFO Font Loader
1. Add `norad` dependency
2. Implement `UfoFont` using norad types
3. Convert `norad::Contour` → our `Contour`
4. Handle kerning from UFO

### Phase 4: VSF Format & Loader
1. Define VSF JSON schema
2. Implement `VsfFont` implementing `Font` trait
3. Add VSF parser (serde deserialization)
4. Document format specification

### Phase 5: vsf-convert CLI Tool
1. Create `vsf-convert` binary crate
2. Implement Hershey → VSF converter
   - Parse Hershey ASCII format
   - Map character sets to Unicode
   - Generate VSF JSON output
3. Implement UFO → VSF converter
   - Use existing `UfoFont` loader
   - Export to VSF format
4. Convert and bundle common Hershey fonts as `.vsf`

### Phase 6: Text Rendering
1. Implement `TextRenderer`
2. Add text layout with kerning
3. Create `Text` shape type
4. Integrate with `Drawing`

### Phase 7: Integration
1. Add to sketch-runner preview
2. SVG export support
3. Plotter optimization for text paths
4. Example sketches

## Dependencies

### drawing-text crate
```toml
[dependencies]
norad = "0.17"              # UFO parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"          # VSF format
thiserror = "1.0"           # Error handling
kurbo = { version = "0.11", features = ["serde"] }  # Already used
```

### vsf-convert CLI tool
```toml
[dependencies]
drawing-text = { path = "../drawing-text" }  # Reuse font loaders
clap = { version = "4.0", features = ["derive"] }  # CLI argument parsing
```

## References

- [Hershey Fonts Wikipedia](https://en.wikipedia.org/wiki/Hershey_fonts)
- [p5-single-line-font-resources](https://github.com/golanlevin/p5-single-line-font-resources) - Comprehensive collection
- [p5-hershey-js](https://github.com/LingDong-/p5-hershey-js) - Reference implementation
- [UFO Specification](https://unifiedfontobject.org/versions/ufo3/)
- [GLIF Specification](https://unifiedfontobject.org/versions/ufo3/glyphs/glif/)
- [norad crate](https://github.com/linebender/norad) - Rust UFO library
- [Relief SingleLine](https://github.com/isdat-type/Relief-SingleLine) - Open-source single-line font
- [hershey-fonts](https://github.com/kamalmostafa/hershey-fonts) - C library and JHF files
- [single-line-font-renderer](https://github.com/jvolker/single-line-font-renderer) - Browser tool
