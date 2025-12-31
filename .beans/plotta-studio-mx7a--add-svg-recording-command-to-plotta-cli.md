---
# plotta-studio-mx7a
title: Add SVG recording command to plotta-cli
status: completed
type: feature
priority: normal
created_at: 2025-12-30T19:40:29Z
updated_at: 2025-12-31T13:55:10Z
parent: plotta-studio-7wzz
---

Add a 'record' command to plotta-cli that simulates plotting and records all optimized strokes into an SVG file. This shows exactly what would be plotted including stroke order optimization and reversal.

## Features

- Records optimized strokes to SVG (exactly what would be plotted)
- Optional travel lines (dashed gray) showing pen-up movements
- Optional direction arrows showing stroke direction
- Unified stroke width for all strokes
- Preserves original stroke colors

## Checklist

- [x] Add `drawing-plotter` dependency to `drawing-svg/Cargo.toml`
- [x] Add `RecordOptions` struct to `drawing-svg`
- [x] Add `record_strokes_to_svg()` function to `drawing-svg`
- [x] Add helper for travel line SVG generation
- [x] Add helper for direction arrow SVG generation
- [x] Add `Record` command variant to CLI
- [x] Implement `cmd_record()` function
- [x] Add tests for SVG recording
- [x] Run clippy and tests