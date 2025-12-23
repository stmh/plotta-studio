---
# plotta-studio-fcfc
title: Investigate replacing 2D primitives with kurbo
status: todo
type: task
created_at: 2025-12-23T19:19:35Z
updated_at: 2025-12-23T19:19:35Z
parent: n2l2
---

Evaluate whether we can replace our custom 2D geometry types in drawing-core with kurbo, the 2D geometry library from Linebender.

## Background

kurbo (https://github.com/linebender/kurbo) is a mature 2D geometry library used by:
- Vello (which we already depend on)
- Norad (UFO font library we plan to use for SLF)
- Peniko, Skrifa, and other Linebender projects

## Current Custom Types in drawing-core
- `Point` - 2D point with x, y
- `Transform` - 2D affine transformation matrix
- `Rect` - Rectangle
- `Line` - Line segment
- `Circle`, `Ellipse`, `Arc` - Curved shapes
- `Path`, `PathSegment` - Bezier paths
- `quad_bezier`, `cubic_bezier` - Bezier evaluation functions

## kurbo Equivalents
- `kurbo::Point` - 2D point
- `kurbo::Affine` - 2D affine transform
- `kurbo::Rect` - Rectangle
- `kurbo::Line` - Line segment
- `kurbo::Circle`, `kurbo::Ellipse`, `kurbo::Arc` - Curves
- `kurbo::BezPath` - Bezier path with segments
- `kurbo::CubicBez`, `kurbo::QuadBez` - Bezier curves

## Investigation Checklist
- [ ] Compare API surface of our types vs kurbo
- [ ] Check if kurbo supports all operations we need
- [ ] Evaluate serde support for kurbo types
- [ ] Consider impact on existing API (breaking changes)
- [ ] Check if Vello already re-exports kurbo types we could use
- [ ] Benchmark any performance differences
- [ ] Estimate migration effort

## Potential Benefits
- Less code to maintain
- Battle-tested library used in production
- Consistent types across norad, vello, and our code
- Additional functionality (arc length, bounding boxes, etc.)

## Potential Concerns
- API differences may require adaptation
- Serde serialization format may differ
- Loss of control over implementation details
