---
# plotta-studio-ah5h
title: Enhance single-line font support
status: draft
type: epic
created_at: 2025-12-26T12:33:10Z
updated_at: 2025-12-26T12:33:10Z
---

Implement remaining single-line font features documented in the completed plotta-studio-n2l2 bean.

## Background

The `drawing-text` crate already has working implementations for:
- **HersheyFont**: JHF format parser with public domain Simplex font
- **VsfFont**: JSON-based format with bezier curve support and kerning
- **SvgFont**: SVG font file parser with `<font>` element support
- **TextRenderer**: Layout engine with alignment, spacing, and multiline support
- **Contour conversion**: `to_path()`, `to_stroke()`, `flatten()` methods

## Planned Enhancements

Based on the original bean documentation, the following work remains:

### 1. UFO Font Loader (using `norad` crate)
UFO 3 is the industry-standard human-readable font format. Key features:
- Directory-based format with XML/plist files
- Support for open contours (essential for single-line fonts)
- Rich metadata and kerning support
- Bezier curve support (cubic and quadratic)

Implementation steps:
- [ ] Add `norad` dependency to drawing-text
- [ ] Create `UfoFont` struct implementing `Font` trait
- [ ] Parse font metrics from `fontinfo.plist`
- [ ] Parse glyphs from `.glif` files in `glyphs/` directory
- [ ] Convert UFO contours to our `Contour` type
- [ ] Parse kerning from `kerning.plist`
- [ ] Add unit tests with sample UFO font

### 2. Additional Hershey Font Variants
The Hershey font collection includes many variants beyond Simplex:
- Gothic (Roman, Greek)
- Script
- Italic
- Cyrillic
- Japanese (Kanji, Hiragana, Katakana)
- Mathematical symbols

Implementation steps:
- [ ] Research available .jhf files from hershey-fonts collection
- [ ] Add additional .jhf font files to fonts/hershey/
- [ ] Create loader functions for each font variant
- [ ] Add font preview/demo to sketch-003-text

### 3. `vsf-convert` CLI Tool
A command-line tool for converting between font formats:
- Convert Hershey to VSF
- Convert SVG font to VSF
- Convert UFO to VSF
- Validate VSF files

Implementation steps:
- [ ] Create new binary crate `vsf-convert`
- [ ] Add clap for CLI argument parsing
- [ ] Implement conversion from each source format
- [ ] Add validation and error reporting
- [ ] Add documentation and usage examples

### 4. FontManager for Managing Multiple Fonts
A centralized registry for font management:
- Load fonts from various sources
- Cache loaded fonts
- Font discovery from directories
- Default font fallback

Implementation steps:
- [ ] Design FontManager API in drawing-text
- [ ] Implement font registry with HashMap<String, Box<dyn Font>>
- [ ] Add font discovery from directory
- [ ] Add built-in font loading shortcuts
- [ ] Integrate with RenderContext in drawing-core

## Priority

Recommended implementation order:
1. **FontManager** - Most impactful for user experience
2. **Additional Hershey variants** - Easy win, expands font options
3. **UFO Font Loader** - Enables using professional single-line fonts
4. **vsf-convert CLI** - Nice-to-have for font conversion

## References

- [norad crate](https://docs.rs/norad) - UFO 3 parsing
- [hershey-fonts](https://github.com/kamalmostafa/hershey-fonts) - JHF files
- [UFO 3 spec](https://unifiedfontobject.org/versions/ufo3/)
- [p5-hershey-js](https://github.com/LingDong-/p5-hershey-js) - Reference implementation