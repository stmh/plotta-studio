---
# plotta-studio-hjm9
title: Investigate clipping bug with rotated rectangles
status: completed
type: bug
priority: normal
created_at: 2026-01-03T21:51:11Z
updated_at: 2026-01-03T21:55:41Z
---

When clipping a rectangle with an inverted clip of another rotated rectangle, one edge is incorrectly removed. Need to investigate the clip implementation in drawing-core.