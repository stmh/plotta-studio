---
# n2l2
title: Investigate Single Line Font (SLF) support
status: draft
type: epic
created_at: 2025-12-23T19:14:41Z
updated_at: 2025-12-23T19:14:41Z
---

Research and plan Single Line Font support for plotta-studio. SLFs are essential for pen plotters as they draw text with single strokes rather than filled outlines.

## Background

Single Line Fonts (also called stroke fonts, stick fonts, or engraving fonts) are designed for applications where text is drawn as paths rather than filled shapes. This is ideal for:
- Pen plotters (AxiDraw, etc.)
- Laser engraving
- CNC routing
- Any application needing text as vector strokes

## UFO Format Overview

The Unified Font Object (UFO) format is a cross-platform, human-readable font format:
- Website: https://unifiedfontobject.org/
- Current version: UFO 3 (UFO 4 in development)
- Structure: Directory-based with XML/plist files
- Key files:
  - `metainfo.plist` - Format version info
  - `fontinfo.plist` - Font metadata (family, weight, metrics)
  - `glyphs/` - Directory of `.glif` files (one per glyph)
  - `kerning.plist` - Kerning pairs
  - `groups.plist` - Glyph groups

### UFO Glyph Structure (relevant to SLF)
- Glyphs contain `Contour` objects (bezier paths)
- Each contour has `ContourPoint` objects with types: `move`, `line`, `curve`, `qcurve`, `offcurve`
- **For SLF**: Contours are OPEN (not closed) - they represent stroke paths
- Contours can be cubic or quadratic bezier curves

## Norad Crate

The `norad` crate (https://github.com/linebender/norad) is a mature Rust library for UFO:
- Version: 0.17.0
- License: MIT/Apache-2.0
- Maintainer: Linebender (same org as Vello, which we already use)
- Features:
  - Full UFO 3 read/write support
  - `Font`, `Layer`, `Glyph`, `Contour`, `ContourPoint` types
  - Kerning and groups support
  - Optional `kurbo` integration for geometry

### Key Norad Types for SLF
```rust
// Load a font
let font = norad::Font::load("myfont.ufo")?;

// Access glyphs
let layer = font.default_layer();
let glyph = layer.get_glyph("A")?;

// Glyph contains:
// - glyph.width (advance width)
// - glyph.contours (Vec<Contour>)
// - glyph.components (references to other glyphs)

// Contour contains ContourPoints with:
// - point.x, point.y
// - point.typ (PointType: Move, Line, Curve, QCurve, OffCurve)
```

## Research Areas
- [x] Understand UFO format and its SLF support
- [x] Evaluate norad crate for font parsing
- [ ] Research SLF font sources (Hershey fonts, CamBam stick fonts, EMSAllure, etc.)
- [ ] Find/create test UFO single-line fonts
- [ ] Determine data structures needed to represent single-line glyphs
- [ ] Plan API for text rendering to strokes

## Technical Considerations
- Norad uses `kurbo` for geometry (optional feature) - we might want to use our own Point/Path types
- Need to convert `norad::Contour` to `drawing_core::Stroke` or `Path`
- Handle kerning for proper letter spacing
- Consider caching parsed fonts for performance
- SLF glyphs have open contours - our `Stroke` type already supports non-closed paths

## Potential Implementation Steps
1. Add `norad` dependency with minimal features
2. Create `drawing-text` crate (or add to drawing-core)
3. Implement UFO font loading and caching
4. Create glyph-to-strokes conversion (Contour -> Path/Stroke)
5. Implement basic text layout (string -> positioned glyphs with kerning)
6. Add `Text` element type to drawing-core
7. Test with real SLF fonts

## SLF Font Sources to Investigate
- Hershey fonts (public domain, classic vector fonts)
- CamBam stick fonts
- EMSAllure and other engraving fonts
- Single-line fonts from font foundries
- Converting existing fonts to SLF

## Dependencies
```toml
[dependencies]
norad = "0.17"  # or with features = ["kurbo"]
```
