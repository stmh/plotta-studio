---
# plotta-studio-xwfm
title: Convert Processing fonts to VSF format
status: in-progress
type: feature
priority: normal
created_at: 2025-12-26T13:23:30Z
updated_at: 2025-12-26T13:24:54Z
parent: plotta-studio-ah5h
---

Create a `vsf-convert` CLI tool to convert vintage/procedural single-line fonts from Golan Levin's p5-single-line-font-resources repository to our VSF (Vector Stroke Font) JSON format. Bundle a curated initial set of 3 converted fonts with plotta-studio.

## Initial Fonts

| Font | Origin | License | Description |
|------|--------|---------|-------------|
| **Asteroids** | 1979 Atari arcade game | Public Domain | Iconic vector game font by Ed Logg, extracted by Trammell Hudson |
| **Apple 410** | 1983 Apple Color Plotter | MIT | Vintage plotter font, reverse-engineered by Adam Mayer (@phooky) |
| **minf** | 2024 Golan Levin | CC0 | Ultra-minimal 72-byte procedural font, 4 points per letter |

## Source Data Analysis

**Asteroids font:**
- Source: JavaScript object with coordinate arrays
- Format: Each glyph is an array of strokes, each stroke is `[x1,y1, x2,y2, ...]`
- Characters: A-Z, 0-9 (uppercase only)
- Grid: ~12 units tall

**Apple 410 font:**
- Source: JSON with byte-encoded coordinates
- Format: High 4 bits = X, low 4 bits = Y (16x16 grid)
- Characters: Full ASCII printable set
- Special: Pen-up encoded as specific byte value

**minf font:**
- Source: 72-byte base64 string
- Format: Each letter = 4 points, 2 bits per coordinate
- Characters: A-Z only (lowercase)
- Grid: 4x4 (2-bit resolution)

All three are polyline-only (no curves), mapping directly to VSF with `"type": "move"` and `"type": "line"` points.

## CLI Design

```
vsf-convert <format> <input> -o <output.vsf>

Formats:
  asteroids   - Asteroids arcade font (JS/JSON)
  apple410    - Apple 410 plotter font (JS/JSON)
  minf        - Ultra-minimal base64 font

Options:
  -o, --output <file>   Output VSF file path
  -n, --name <name>     Font name (default: derived from input)
  --preview             Print glyph preview to terminal
  
Examples:
  vsf-convert asteroids asteroids_font.json -o asteroids.vsf
  vsf-convert minf "base64string..." -o minf.vsf
  vsf-convert apple410 apple_410_font.json -o apple410.vsf
```

## Crate Structure

```
crates/vsf-convert/
├── Cargo.toml          # clap, serde_json, drawing-text
├── src/
│   ├── main.rs         # CLI entry point
│   ├── formats/
│   │   ├── mod.rs
│   │   ├── asteroids.rs
│   │   ├── apple410.rs
│   │   └── minf.rs
│   └── convert.rs      # Common conversion to VsfFont
```

## Bundled Fonts Location

```
fonts/
├── hershey/
│   ├── simplex.jhf      # (existing)
│   └── LICENSE
├── svg/
│   ├── ReliefSingleLine-Regular.svg  # (existing)
│   └── ...
└── vsf/                 # (new directory)
    ├── asteroids.vsf
    ├── apple410.vsf
    ├── minf.vsf
    └── README.md        # Attribution and license info
```

## Metadata Population

The converter will populate VSF metadata fields for proper attribution:

```json
{
  "version": "1.0",
  "name": "Asteroids",
  "metadata": {
    "author": "Ed Logg (Atari), extracted by Trammell Hudson",
    "license": "Public Domain",
    "description": "Vector font from the 1979 Atari Asteroids arcade game"
  },
  "metrics": { ... },
  "glyphs": { ... }
}
```

## Checklist

### Phase 1: Create vsf-convert crate
- [ ] Set up crate with clap for CLI parsing
- [ ] Add drawing-text dependency for VsfFont output
- [ ] Implement common conversion helpers

### Phase 2: Implement format parsers
- [ ] Asteroids parser (JS object -> VsfFont)
- [ ] Apple 410 parser (4-bit packed coords -> VsfFont)
- [ ] minf parser (base64 decode -> VsfFont)

### Phase 3: Convert and bundle fonts
- [ ] Download source data from p5-single-line-font-resources
- [ ] Run conversions, verify output
- [ ] Add converted .vsf files to fonts/vsf/
- [ ] Add README.md with attribution

### Phase 4: Integration
- [ ] Update sketch-003-text to demo a vintage font
- [ ] Add to FontManager built-in fonts (when FontManager is implemented)

## References

- [p5-single-line-font-resources](https://github.com/golanlevin/p5-single-line-font-resources) - Source repository
- [Asteroids font source](https://github.com/golanlevin/p5-single-line-font-resources/tree/main/asteroids_font)
- [Apple 410 font source](https://github.com/golanlevin/p5-single-line-font-resources/tree/main/apple_410_font)
- [minf font source](https://github.com/golanlevin/p5-single-line-font-resources/tree/main/minf)