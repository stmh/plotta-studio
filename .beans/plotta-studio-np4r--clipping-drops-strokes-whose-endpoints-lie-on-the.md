---
# plotta-studio-np4r
title: Clipping drops strokes whose endpoints lie on the clip boundary
status: completed
type: bug
priority: high
created_at: 2026-05-31T16:25:36Z
updated_at: 2026-05-31T16:27:00Z
---

On-boundary points treated as outside in clip.rs, dropping edge-touching strokes. See GitHub issue #19.

## Summary of Changes

Root cause: `clip_linestring_to_region` used geo's strict `Contains` for the point-in-region test, so points exactly on the clip polygon boundary were treated as outside. Strokes whose endpoints sit on the clip edge (e.g. full-height vertical lines, or a rect outline drawn on the boundary) were silently dropped.

Fix (crates/drawing-core/src/clip.rs):
- Added `point_in_closed_region` helper using geo's `Intersects` (which includes the boundary), giving closed-region semantics. Replaced both `clip_region.contains(...)` calls with it.

Tests added:
- `test_clip_full_height_lines_on_boundary_survive` - four full-height vertical lines clipped to the bounding square survive intact.
- `test_clip_rect_outline_on_boundary_survives` - a closed rect outline on the clip boundary is preserved.

All 74 drawing-core tests pass; no regressions.
