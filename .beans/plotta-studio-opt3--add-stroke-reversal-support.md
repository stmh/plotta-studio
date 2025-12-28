---
# plotta-studio-opt3
title: Add stroke reversal support
status: completed
type: task
priority: normal
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-28T17:25:17Z
parent: plotta-studio-opt1
---

Allow strokes to be drawn in reverse direction when it reduces travel distance.

## Implementation

Implemented `OptimizedStroke` and `optimize_strokes_with_reversal()` in `crates/drawing-plotter/src/optimize.rs`.

Key features:
- `OptimizedStroke<'a>` holds a stroke reference and a `reversed` flag
- `start()` and `end()` return effective positions considering reversal
- `points()` iterator yields points in correct order
- `optimize_strokes_with_reversal(strokes, allow_reversal)` finds optimal order
- When `allow_reversal=true`, considers both start and end points of each stroke
- Enabled by default in all plotting functions

## Results

For sample-drawing.json:
- Travel distance reduced from 773.4 mm to 401.5 mm (48% reduction)
- 6 out of 18 strokes reversed
- Estimated time improved from ~2m 12s to ~2m 6s

## Files Modified
- `crates/drawing-plotter/src/optimize.rs` - Added `OptimizedStroke`, `optimize_strokes_with_reversal()`, distance functions
- `crates/drawing-plotter/src/axidraw.rs` - Updated to use optimized strokes with reversal
- `crates/drawing-plotter/src/stats.rs` - Updated to calculate stats with reversal
- `crates/drawing-plotter/src/lib.rs` - Exported new types and functions
- `crates/plotta-cli/src/main.rs` - Shows reversed stroke count in preview
