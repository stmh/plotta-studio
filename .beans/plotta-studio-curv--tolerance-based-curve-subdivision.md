---
# plotta-studio-curv
title: Replace hardcoded curve flattening with tolerance-based subdivision
status: completed
type: task
priority: high
created_at: 2025-12-30T00:00:00Z
updated_at: 2025-12-30T00:00:00Z
---

Replace hardcoded segment counts for curve flattening with adaptive tolerance-based subdivision using kurbo's `flatten` function. Default tolerance 0.05mm (configurable). Add path cleanup to remove redundant collinear points.

## Summary

Replaced fixed segment counts with kurbo's tolerance-based flattening algorithm. All curves now use adaptive subdivision based on error tolerance rather than arbitrary segment counts.

## Changes Made

### 1. RenderContext (`context.rs`)
- Added `tolerance: f64` field with default `0.05` (mm)
- Added `DEFAULT_TOLERANCE` constant
- Added `empty()` constructor and `with_tolerance()` builder

### 2. Path Simplification (`simplify.rs` - new)
- `simplify_points()` - removes collinear points within tolerance
- `remove_duplicates()` - removes duplicate consecutive points
- `cleanup_points()` - combines both operations

### 3. Flatten Functions (`flatten.rs`)
All functions now use kurbo's `flatten()` with tolerance:
- `flatten_circle()` - uses `kurbo::Circle::path_elements()`
- `flatten_ellipse()` - uses `kurbo::Ellipse::path_elements()`
- `flatten_arc()` - uses `kurbo::Arc::path_elements()`
- `flatten_path()` - uses `kurbo::flatten()` directly

### 4. Primitives (`primitives.rs`)
- Removed `segments` field from Circle, Ellipse, Arc
- Removed `with_segments()` builder methods

### 5. Element (`element.rs`)
- All flatten calls now pass `ctx.tolerance`

## Before/After

| Shape | Before | After |
|-------|--------|-------|
| Circle | 64 fixed segments | Adaptive based on 0.05mm tolerance |
| Ellipse | 64 fixed segments | Adaptive based on 0.05mm tolerance |
| Arc | 32 fixed segments | Adaptive based on 0.05mm tolerance |
| QuadBez | 16 fixed steps | Adaptive based on 0.05mm tolerance |
| CubicBez | 24 fixed steps | Adaptive based on 0.05mm tolerance |

## Test Results

All 103 tests pass (63 in drawing-core, 40 in drawing-svg).

## Usage

```rust
// Use default tolerance (0.05mm)
let ctx = RenderContext::empty();

// Use custom tolerance
let ctx = RenderContext::empty().with_tolerance(0.1);

// Flatten with adaptive subdivision
let strokes = drawing.flatten(&ctx);
```

## Notes

- CLI flag not added (plotta-cli doesn't exist on this branch)
- Tolerance unit is millimeters (matches plotter coordinates)
- Lower tolerance = more points = higher quality
- Higher tolerance = fewer points = faster rendering
