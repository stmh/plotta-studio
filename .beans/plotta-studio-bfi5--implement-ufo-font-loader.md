---
# plotta-studio-bfi5
title: Implement UFO font loader
status: draft
type: feature
created_at: 2025-12-26T12:34:07Z
updated_at: 2025-12-26T12:34:07Z
parent: plotta-studio-ah5h
---

Implement UFO 3 (Unified Font Object) font loading using the norad crate.

## Background

UFO is the industry-standard human-readable font format, widely used by font designers. It supports open contours (essential for single-line fonts) and bezier curves.

## UFO 3 Format Structure

```
myfont.ufo/
├── metainfo.plist      # Format version
├── fontinfo.plist      # Font metadata (family, weight, metrics)
├── glyphs/             # Glyph files
│   ├── contents.plist  # Glyph name → filename mapping
│   ├── A_.glif         # Glyph 'A'
│   └── ...
├── kerning.plist       # Kerning pairs
├── groups.plist        # Glyph groups (for kerning classes)
└── lib.plist           # Custom data
```

## GLIF Format

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
  </outline>
</glyph>
```

Point types:
- `move`: First point of open contour
- `line`: Draw line from previous point
- `curve`: Cubic bezier (preceded by offcurve points)
- `qcurve`: Quadratic bezier
- (no type): offcurve control point

## Implementation

1. Add norad dependency
2. Create UfoFont struct
3. Parse font using norad
4. Convert norad types to our types
5. Handle open vs closed contours

## API

```rust
pub struct UfoFont {
    name: String,
    glyphs: HashMap<char, Glyph>,
    metrics: FontMetrics,
    kerning: HashMap<(char, char), f64>,
}

impl UfoFont {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, FontError>;
}

impl Font for UfoFont { ... }
```

## Dependencies

```toml
norad = "0.17"  # UFO parsing from Linebender
```

## Checklist

- [ ] Add norad dependency to Cargo.toml
- [ ] Create ufo.rs module
- [ ] Implement UfoFont struct
- [ ] Parse fontinfo.plist for metrics
- [ ] Parse glyphs directory
- [ ] Convert norad Contour to our Contour type
- [ ] Handle open contours correctly
- [ ] Parse kerning from kerning.plist
- [ ] Add unit tests with sample UFO font
- [ ] Export UfoFont from lib.rs