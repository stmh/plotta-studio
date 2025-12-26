# VSF (Vector Stroke Font) Files

This directory contains single-line fonts in VSF format, converted from various
vintage and procedural font sources.

## Included Fonts

### Asteroids (`asteroids.vsf`)
- **Origin**: Atari Asteroids arcade game (1979)
- **Author**: Ed Logg (Atari), extracted by Trammell Hudson
- **License**: Public Domain
- **Characters**: A-Z, 0-9, punctuation
- **Source**: https://github.com/golanlevin/p5-single-line-font-resources/tree/main/asteroids_font

### Apple 410 (`apple410.vsf`)
- **Origin**: Apple 410 Color Plotter (1983)
- **Author**: Adam Mayer (@phooky)
- **License**: MIT
- **Characters**: Full ASCII printable set (uppercase, lowercase, numbers, symbols)
- **Source**: https://github.com/golanlevin/p5-single-line-font-resources/tree/main/apple_410_font

### minf (`minf.vsf`)
- **Origin**: Ultra-minimal procedural font (2024)
- **Author**: Golan Levin
- **License**: CC0 (Public Domain)
- **Characters**: a-z, A-Z
- **Notes**: Each letter is exactly 4 points connected by 3 line segments. The entire
  font fits in 72 bytes when base64 encoded!
- **Source**: https://github.com/golanlevin/p5-single-line-font-resources/tree/main/minf

## Using These Fonts

```rust
use drawing_text::VsfFont;

// Load a font
let font = VsfFont::from_file("fonts/vsf/asteroids.vsf")?;

// Use with TextRenderer
let renderer = TextRenderer::new();
let options = TextOptions::new(24.0);
let layout = renderer.layout("HELLO WORLD", &font, &options);
```

## Converting Additional Fonts

Use the `vsf-convert` tool to convert additional fonts:

```bash
# Convert individual fonts
vsf-convert asteroids input.json -o output.vsf
vsf-convert apple410 input.json -o output.vsf
vsf-convert minf "base64data..." -o output.vsf

# Convert all embedded fonts
vsf-convert all -o fonts/vsf/
```

## Credits

Fonts sourced from Golan Levin's excellent collection:
https://github.com/golanlevin/p5-single-line-font-resources
