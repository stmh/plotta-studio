---
# plotta-studio-3dbz
title: Fix hidden line removal to clip against all overlapping squares
status: completed
type: task
priority: normal
created_at: 2026-01-03T21:57:53Z
updated_at: 2026-01-03T21:58:35Z
---

Update sketch-012 so each square is clipped against ALL subsequent squares (not just the next one) to properly remove all hidden lines.