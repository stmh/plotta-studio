---
# plotta-studio-n2l2
title: Investigate Single Line Font (VSF) support
status: in-progress
type: epic
created_at: 2025-12-23T19:14:41Z
updated_at: 2025-12-24T00:00:00Z
---

Research and plan Single Line Font support for plotta-studio. Vector Stroke Fonts (VSF) are essential for pen plotters as they draw text with single strokes rather than filled outlines.

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

### Converter-Based Approach

Rather than implementing multiple font loaders at runtime, we use a **converter-based architecture**:

1. **Build-time converters** transform source formats (Hershey, UFO) → VSF
2. **Single runtime loader** only needs to parse VSF
3. **Bundle pre-converted fonts** as `.vsf` files

```
┌─────────────────────────────────────────────────────────────────┐
│                    BUILD-TIME / CLI TOOLS                       │
├─────────────────────────────────────────────────────────────────┤
│  Hershey (.jhf) ──→ ┌──────────────┐                           │
│  UFO (.ufo)     ──→ │ vsf-convert  │ ──→ .vsf files            │
│  SVG Font       ──→ └──────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (bundled fonts / user fonts)
┌─────────────────────────────────────────────────────────────────┐
│                         RUNTIME                                 │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐      ┌─────────────────┐                  │
│  │   FontManager   │ ──→  │    VsfFont      │                  │
│  │  - load(.vsf)   │      │  (single impl)  │                  │
│  │  - get_font()   │      └─────────────────┘                  │
│  │  - list_fonts() │              │                            │
│  └─────────────────┘              ▼                            │
│                           ┌─────────────────┐                  │
│                           │      Glyph      │                  │
│                           │  - contours     │                  │
│                           │  - advance      │                  │
│                           └─────────────────┘                  │
│                                   │                            │
│                                   ▼                            │
│                           ┌─────────────────┐                  │
│                           │  TextRenderer   │                  │
│                           │  - render()     │                  │
│                           │  - layout()     │                  │
│                           └─────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

**Benefits:**
- Single font format at runtime (simpler code, fewer dependencies)
- VSF is a superset (supports bezier curves that Hershey lacks)
- Converters are separate CLI tools, not runtime dependencies
- Pre-convert and bundle common Hershey fonts as `.vsf`
- Users can convert their own fonts with the CLI tool
- `norad` dependency only needed in converter, not main library

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

### VsfFont (Single Implementation)

```rust
/// A VSF font loaded from a .vsf file
pub struct VsfFont {
    name: String,
    metadata: FontMetadata,
    metrics: FontMetrics,
    glyphs: HashMap<char, Glyph>,
    kerning: HashMap<(char, char), f64>,
}

impl VsfFont {
    /// Load from a .vsf file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, FontError>;

    /// Load from bytes (embedded fonts)
    pub fn from_bytes(data: &[u8]) -> Result<Self, FontError>;

    /// Font family name
    pub fn name(&self) -> &str;

    /// Get a glyph by unicode character
    pub fn glyph(&self, c: char) -> Option<&Glyph>;

    /// Get kerning adjustment between two characters
    pub fn kerning(&self, left: char, right: char) -> f64;

    /// Get font metrics
    pub fn metrics(&self) -> &FontMetrics;

    /// Check if font has a glyph for character
    pub fn has_glyph(&self, c: char) -> bool;

    /// Get all available characters
    pub fn available_chars(&self) -> Vec<char>;
}
```

### FontManager

```rust
/// Manages loaded fonts and provides access by name
pub struct FontManager {
    fonts: HashMap<String, VsfFont>,
    search_paths: Vec<PathBuf>,
}

impl FontManager {
    pub fn new() -> Self;

    /// Add a directory to search for .vsf files
    pub fn add_search_path(&mut self, path: impl AsRef<Path>);

    /// Load a font from file
    pub fn load_font(&mut self, path: impl AsRef<Path>) -> Result<&VsfFont, FontError>;

    /// Get a loaded font by name
    pub fn get_font(&self, name: &str) -> Option<&VsfFont>;

    /// List all loaded fonts
    pub fn list_fonts(&self) -> Vec<&str>;

    /// Load all .vsf files from search paths
    pub fn load_all(&mut self) -> Result<(), FontError>;
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

### Phase 1: VSF Format & Parser
1. Create `drawing-text` crate
2. Define VSF JSON schema (finalize format spec)
3. Implement `VsfFont` parser (`serde` deserialization)
4. Define `Glyph`, `Contour`, `FontMetrics` types
5. Implement `Contour` → `Path` → `Stroke` conversion
6. Add unit tests

### Phase 2: vsf-convert CLI Tool
1. Create `vsf-convert` binary crate
2. Implement Hershey → VSF converter
   - Parse Hershey ASCII format
   - Map character sets to Unicode
   - Generate VSF JSON output
3. Implement UFO → VSF converter (using `norad`)
   - Parse UFO with norad
   - Convert contours and kerning
   - Generate VSF JSON output
4. Add CLI interface (input format detection, output path)

### Phase 3: Bundle Fonts
1. Convert common Hershey fonts to VSF:
   - Simplex (romans, scripts)
   - Complex (romanc, scriptc)
   - Gothic, Greek, Cyrillic
2. Include as embedded assets or separate font package
3. Document available bundled fonts

### Phase 4: Text Rendering
1. Implement `FontManager` (load, cache, lookup)
2. Implement `TextRenderer`
3. Add text layout with kerning
4. Create `Text` shape type in drawing-core
5. Integrate with `Drawing`

### Phase 5: Integration
1. Add to sketch-runner preview
2. SVG export support for text
3. Plotter optimization for text paths
4. Example sketches with text

## Dependencies

### drawing-text crate (runtime)
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"          # VSF format parsing
thiserror = "1.0"           # Error handling
kurbo = { version = "0.11", features = ["serde"] }  # Already used in drawing-core
```

### vsf-convert CLI tool (build-time only)
```toml
[dependencies]
norad = "0.17"              # UFO parsing (only needed here)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.0", features = ["derive"] }  # CLI argument parsing
thiserror = "1.0"
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
