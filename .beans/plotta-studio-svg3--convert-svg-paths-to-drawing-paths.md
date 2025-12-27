---
# plotta-studio-svg3
title: Convert SVG paths to drawing paths
status: completed
type: task
priority: normal
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-27T18:05:49Z
parent: plotta-studio-svg1
---

Implement conversion from usvg path data to drawing-core Path/PathSegment types.

## Implementation Details

usvg simplifies SVG paths to only these commands:
- MoveTo (M)
- LineTo (L)
- CurveTo (C) - cubic bezier
- ClosePath (Z)

Map to drawing-core PathSegment:
```rust
fn convert_path_data(path: &usvg::Path) -> Path {
    let mut result = Path::new();

    for segment in path.data().segments() {
        match segment {
            usvg::PathSegment::MoveTo { x, y } => {
                result = result.move_to((x, y));
            }
            usvg::PathSegment::LineTo { x, y } => {
                result = result.line_to((x, y));
            }
            usvg::PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                result = result.cubic_to((x1, y1), (x2, y2), (x, y));
            }
            usvg::PathSegment::ClosePath => {
                result = result.close();
            }
        }
    }

    result
}
```

## Edge Cases to Handle
- Empty paths
- Paths with only MoveTo (no actual strokes)
- Very long paths (may want to split)
- Subpaths (multiple MoveTo commands)

## Files to Modify
- `crates/drawing-svg/src/lib.rs`
