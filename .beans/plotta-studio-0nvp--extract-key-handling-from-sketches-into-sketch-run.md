---
# plotta-studio-0nvp
title: Extract key handling from sketches into sketch-runner
status: completed
type: task
priority: normal
created_at: 2025-12-31T15:36:57Z
updated_at: 2025-12-31T15:48:24Z
---

## Summary

Extract common key handling patterns from sketches into the sketch-runner crate to reduce code duplication and provide standard key bindings for all sketches.

## Analysis

**Common patterns identified across 9 sketches:**

1. **Export SVG (`E` key)** - 8/9 sketches have identical code
2. **Plot to AxiDraw (`P` key)** - 6/9 sketches with `hardware` feature  
3. **Regenerate (`G` key)** - 6/9 sketches call a `build_drawing`/`generate` method

**Hardware feature investigation:**
- `drawing-plotter` has `default = ["hardware"]` feature
- Sketches disable default features and re-enable via their own `hardware` feature
- This means sketch-runner could include hardware support as an optional feature

## Changes Made

### sketch-runner
- Added `svg` feature (default) with `drawing-svg` dependency
- Added `hardware` feature with `drawing-plotter/hardware` dependency
- Added built-in `E` key handler for SVG export (filename based on window title)
- Added built-in `P` key handler for plotting
- Added plot event handling in `about_to_wait` loop
- Re-exports `drawing_svg` and `drawing_plotter` types for convenience

### All sketches
- Removed duplicate `E` key handling code
- Removed duplicate `P` key handling code  
- Removed `plot_handle` field and update() plot event handling
- Simplified Cargo.toml to use `sketch-runner/hardware` feature
- Removed unnecessary `drawing-svg`, `drawing-plotter` dependencies

## Checklist

- [x] Add `drawing-svg` and `drawing-plotter` as optional dependencies to sketch-runner
- [x] Add `hardware` feature to sketch-runner that enables drawing-plotter/hardware
- [x] Add `svg` feature to sketch-runner that enables drawing-svg
- [x] Add built-in `E` key handling for SVG export in sketch-runner
- [x] Add built-in `P` key handling for plotting (with hardware feature)
- [x] Handle plot events in about_to_wait
- [x] Update sketches to remove duplicate E/P key handling
- [x] Simplify sketch Cargo.toml dependencies (can use sketch-runner features)
- [x] Verify all sketches still build and work correctly
- [x] Run cargo fmt and cargo clippy