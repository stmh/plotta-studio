---
# plotta-studio-dab5
title: Add speed control options to plotta-cli
status: completed
type: feature
priority: normal
created_at: 2025-12-28T17:09:51Z
updated_at: 2025-12-31T13:55:10Z
parent: plotta-studio-7wzz
---

Add CLI options to control plotting speed for the plot and preview commands.

## Background
The PlotConfig already supports speed settings:
- pen_down_speed: Drawing speed (default: 25 mm/s)
- pen_up_speed: Travel speed (default: 75 mm/s)

These need to be exposed via CLI options.

## Checklist
- [x] Add --draw-speed option to Plot command
- [x] Add --travel-speed option to Plot command  
- [x] Add speed options to Preview command (for accurate time estimates)
- [x] Update cmd_plot to use custom PlotConfig
- [x] Update cmd_preview to use custom PlotConfig
- [x] Test with different speed values

## Speed Guidelines (from AxiDraw docs)
| Use Case    | Draw Speed | Travel Speed |
| ----------- | ---------- | ------------ |
| Fine detail | 10-20 mm/s | 50-75 mm/s   |
| Normal      | 25-35 mm/s | 75 mm/s      |
| Fast drafts | 50-75 mm/s | 100 mm/s     |