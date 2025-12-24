---
# plotta-studio-fcfc
title: Replace 2D primitives with kurbo
status: completed
type: task
created_at: 2025-12-23T19:19:35Z
updated_at: 2025-12-24T00:00:00Z
parent: n2l2
---

Replaced custom 2D geometry types in drawing-core with kurbo, the 2D geometry library from Linebender.

## Background

kurbo (https://github.com/linebender/kurbo) is a mature 2D geometry library used by:
- Vello (which we already depend on)
- Norad (UFO font library we plan to use for SLF)
- Peniko, Skrifa, and other Linebender projects

## Investigation Results

### ✅ Vello Already Re-exports kurbo

Vello re-exports kurbo directly via `pub use peniko::kurbo;`. This means:
- We already have kurbo v0.11.3 in our dependency tree
- Can access via `vello::kurbo::*`
- No additional dependencies needed

### ✅ Serde Support Available

kurbo has optional serde support via feature flag:
```toml
kurbo = { version = "0.11", features = ["serde"] }
```

However, accessing through vello may require adding kurbo as a direct dependency to enable the serde feature.

### API Comparison

| Our Type | kurbo Equivalent | API Differences |
|----------|------------------|-----------------|
| `Point` | `kurbo::Point` | kurbo separates `Point` (location) from `Vec2` (displacement). Point - Point = Vec2. Our Point can be multiplied by f64, kurbo requires conversion to Vec2 first. |
| `Transform` | `kurbo::Affine` | kurbo uses array `[f64; 6]` internally vs our named fields `a,b,c,d,tx,ty`. kurbo has richer API: `then_rotate`, `then_scale`, `pre_rotate`, `pre_scale`, `reflect`, `svd`, etc. Our `then()` maps to kurbo's `Mul`. |
| `Rect` | `kurbo::Rect` | kurbo uses `x0,y0,x1,y1` (min/max corners) vs our `origin,width,height`. kurbo has more methods: `inset`, `union`, `intersect`, `inflate`, etc. |
| `Line` | `kurbo::Line` | Similar API. kurbo uses `p0,p1` vs our `from,to`. |
| `Circle` | `kurbo::Circle` | kurbo doesn't have `segments` field (uses tolerance-based flattening). |
| `Ellipse` | `kurbo::Ellipse` | kurbo represents with center, radii, and rotation. More mathematically complete. |
| `Arc` | `kurbo::Arc` | Similar conceptually. |
| `Path` | `kurbo::BezPath` | kurbo uses `PathEl` enum (MoveTo, LineTo, QuadTo, CurveTo, ClosePath). Our `PathSegment` is similar. kurbo has `flatten()` method built-in. |
| `quad_bezier()` | `kurbo::QuadBez::eval()` | kurbo has dedicated curve types with evaluation via `ParamCurve` trait. |
| `cubic_bezier()` | `kurbo::CubicBez::eval()` | Same as above. |

### Current Conversion Points

`sketch-runner/src/lib.rs` already converts our types to kurbo for vello rendering:
```rust
use vello::kurbo::{Affine, BezPath, Rect as KurboRect, Stroke as VelloStroke};

// Manual Point conversion via tuples:
path.move_to((stroke.points[0].x, stroke.points[0].y));
```

### Additional kurbo Features We'd Gain

- `Shape` trait with `bounding_box()`, `area()`, `perimeter()`
- Path operations: `flatten()`, `segments()`, `path_segments()`
- `ParamCurve` trait: `eval()`, `subsegment()`, `subdivide()`
- `ParamCurveArclen`: arc length calculation
- `ParamCurveNearest`: find nearest point on curve
- Path fitting: `fit_to_bezpath()`, `fit_to_cubic()`
- Path simplification: `simplify` module
- Path offsetting: `offset` module
- RoundedRect, Triangle, Insets, Size types
- SVG path parsing: `BezPath::from_svg()`

## Breaking Changes Assessment

### High Impact (User-Facing API)
1. **Point arithmetic**: `Point + Point` not allowed in kurbo (must use `Point + Vec2`)
2. **Point * f64**: Must convert to Vec2 first
3. **Rect construction**: Would change from `Rect::new(x, y, w, h)` to `Rect::from_origin_size((x,y), (w,h))`
4. **Transform fields**: Named fields `a,b,c,d,tx,ty` → array access or methods

### Medium Impact (Internal)
1. Stroke flattening would use kurbo's tolerance-based approach instead of segment count
2. Circle/Ellipse no longer have `segments` parameter
3. Path building API slightly different

### Low Impact
1. Import paths change
2. Some method names differ

## Recommendation

**Partial adoption recommended** rather than full replacement:

### Phase 1: Use kurbo for Internal Geometry (Recommended Now)
- Use `vello::kurbo::Affine` internally in sketch-runner (already happening)
- Use kurbo's `BezPath` for flattening curves
- Keep our public API types for now

### Phase 2: Consider Full Migration Later
- When implementing SLF support with norad, full kurbo adoption makes more sense
- Would unify types across vello, norad, and drawing-core
- Breaking change - would need major version bump

### Alternative: Conversion Traits
Add `From`/`Into` implementations between our types and kurbo:
```rust
impl From<Point> for kurbo::Point {
    fn from(p: Point) -> Self { kurbo::Point::new(p.x, p.y) }
}
impl From<kurbo::Point> for Point {
    fn from(p: kurbo::Point) -> Self { Point::new(p.x, p.y) }
}
```

## Conclusion

kurbo is an excellent library that would provide significant value, especially:
- Better curve mathematics (arc length, nearest point, subdivision)
- Path flattening with tolerance-based accuracy
- Consistent types with vello and norad ecosystems

However, full migration would be a breaking change affecting user code. The recommended approach is:

1. **Now**: Add conversion traits between our types and kurbo
2. **Now**: Use kurbo internally for complex geometry operations
3. **Later**: Consider full migration when implementing SLF support (natural break point)

## Implementation Summary

The refactor was completed with the following changes:

### Changes Made
1. Added `kurbo = { version = "0.11", features = ["serde"] }` to drawing-core
2. Re-exported kurbo types: `Point`, `Affine`, `Rect`, `Line`, `Vec2`, `BezPath`, `PathEl`
3. Added `Transform` type alias for `Affine` for API clarity
4. Updated flatten logic to use `Affine * Point` multiplication
5. Updated `Path::to_bezpath()` to convert to kurbo's BezPath
6. Used `kurbo::ParamCurve` trait for bezier evaluation

### Results
- **Lines removed**: ~1737
- **Lines added**: ~183
- **All 56 tests pass**
- No changes needed to downstream crates (drawing-svg, drawing-plotter, sketch-runner)

### Files Modified
- `crates/drawing-core/Cargo.toml` - Added kurbo dependency
- `crates/drawing-core/src/lib.rs` - Replaced custom types with kurbo
