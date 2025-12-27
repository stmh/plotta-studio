---
# plotta-studio-tdru
title: Vello renderer stops with too many strokes (known upstream issue)
status: todo
type: bug
created_at: 2025-12-27T20:42:44Z
updated_at: 2025-12-27T20:42:44Z
---

When rendering more than ~75k-100k strokes, vello silently stops rendering without error messages. The app remains responsive but no frames are generated.

## Root Cause

This is a known upstream issue in vello:
- https://github.com/linebender/vello/issues/720 - Renderer stops functioning under stress without generating error messages
- https://github.com/linebender/vello/issues/548 - Full system hang on Apple M1 8GB

The issue is related to GPU memory limits when too many paths are drawn.

## Current Workaround

In sketch-runner, we limit rendering to 75,000 strokes maximum to prevent GPU overload. This allows interactive preview while the full drawing can still be exported to SVG for plotting.

## Affected Use Cases

- Large SVG imports (e.g., hamburg.svg with 135k paths)
- Complex generative drawings with many elements

## Possible Future Solutions

- Wait for upstream vello fix
- Implement progressive/chunked rendering
- Add LOD (level of detail) for preview mode
- Cache complex drawings to texture