---
# plotta-studio-jay9
title: Refactor Element::text() to accept FontRef instead of string
status: completed
type: feature
priority: normal
created_at: 2025-12-27T15:02:04Z
updated_at: 2025-12-27T15:12:57Z
---

## Summary

Refactor the text API to accept FontRef directly instead of magic strings, improving type safety and avoiding runtime font lookups.

## Changes

### 1. drawing-text/src/manager.rs
- Change `load_hershey()` to return `Result<FontRef, FontError>`
- Change `load_from_str()` to return `Result<FontRef, FontError>`
- Change `load_file()` to return `Result<FontRef, FontError>`
- Keep `load_all_hershey()` returning `Result<usize, FontError>`

### 2. drawing-core/src/text.rs
- Add `#[serde(skip)] font: Option<FontRef>` field to `Text`
- Change `Text::new()` to accept `FontRef`, extract name from it
- Update `flatten()` to use cached font, fall back to registry lookup

### 3. drawing-core/src/element.rs
- Change `Element::text()` signature to accept `FontRef`

### 4. drawing-utils/src/frame.rs
- Replace `font_name: String` with `font: FontRef`
- Change `FrameOptions::new(font: FontRef)` constructor
- Remove `Default` impl (font is required)

### 5. sketch-runner/src/lib.rs
- Add `SketchContext` struct bundling `RenderContext` and `FontManager`
- Update `Sketch` trait to use `SketchContext`
- Remove embedded VSF/SVG fonts, keep Hershey only
- Re-export `FontRef`

### 6. Update all sketches
- Update trait method signatures
- Load fonts via `ctx.fonts`

## Checklist

- [x] Update FontManager to return FontRef
- [x] Update Text struct and Element::text()
- [x] Update FrameOptions
- [x] Add SketchContext and update Sketch trait
- [x] Remove embedded VSF/SVG fonts from sketch-runner
- [x] Update sketch-001-radial
- [x] Update sketch-002-dvd-screensaver
- [x] Update sketch-003-text
- [x] Update sketch-004-hatched-circles
- [x] Run tests and clippy