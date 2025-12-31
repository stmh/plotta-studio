---
# plotta-studio-o6fw
title: Show remaining time and motion planning status during plotting
status: completed
type: feature
priority: normal
created_at: 2025-12-29T21:22:20Z
updated_at: 2025-12-31T13:55:10Z
parent: plotta-studio-7wzz
---

Enhance the plotta CLI to show:
1. Remaining time estimate during plotting
2. Whether motion planning is enabled

## Checklist
- [x] Add remaining time display to plot progress
- [x] Show motion planning status at start of plot
- [x] Update DrawingStats or PlotEvent to include timing info

## Implementation Notes

- Added `format_duration()` helper function for human-readable time formatting
- Progress bar now shows remaining time that updates based on actual elapsed time
- Uses weighted average of elapsed-based and estimate-based remaining time
- Shows motion planning status in both `preview` and `plot` commands
- Final message shows total elapsed time
- Time estimation now uses motion planning when enabled:
  - Calculates actual accel/decel profiles for each stroke
  - Accounts for starting/stopping at rest between strokes
  - More accurate than constant velocity estimate (~44s vs ~30s for test pattern)