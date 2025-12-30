---
# plotta-studio-q9lh
title: Replace hardcoded curve segments with tolerance-based flattening
status: todo
type: task
created_at: 2025-12-29T18:04:17Z
updated_at: 2025-12-29T18:04:17Z
---

Currently curve flattening uses hardcoded segment counts (16 for QuadTo, 24 for CurveTo, 64 for circles). This should use a tolerance/epsilon-based approach like kurbo's flatten() method, which adaptively subdivides curves based on a maximum deviation from the true curve. This allows users to control the trade-off between accuracy and segment count.

## Current State
- QuadTo: 16 fixed steps
- CurveTo: 24 fixed steps  
- Circle: 64 fixed segments
- Ellipse: 64 fixed segments
- Arc: 32 fixed segments

## Desired Behavior
- Use kurbo's BezPath::flatten(tolerance, callback) for paths
- Calculate segment count for circles/arcs based on radius and tolerance
- Add a global flattening tolerance setting to RenderContext or DrawingOptions
- Smaller tolerance = more segments = higher accuracy
- Larger tolerance = fewer segments = faster plotting, smoother motor motion

## Implementation Notes
- kurbo already has flatten() methods that use adaptive subdivision
- For circles: segments = ceil(2 * PI / acos(1 - tolerance/radius))
- Consider making tolerance configurable via CLI for plotting