---
# plotta-studio-kz15
title: Merge adjacent strokes to reduce pen up/down movements
status: completed
type: feature
priority: normal
created_at: 2025-12-31T16:17:12Z
updated_at: 2026-05-17T15:56:23Z
parent: plotta-studio-opt1
---

## Problem

During plotting, the plotter sometimes does pen up/down between strokes that could be connected - when the end of one stroke is very close to (or identical to) the start of another stroke.

## Solution

Add a stroke merging optimization step that:
1. Detects when the endpoint of stroke A is within a tolerance distance of the startpoint of stroke B
2. Merges such strokes into a single continuous stroke (if styles match)
3. Can be configured via PlotConfig

Also unify sketch-runner plotting with plotta-cli by using the same `PreparedDrawing` + `plot_prepared_in_background()` path.

## Configuration Decisions

- **Tolerance:** 0.05mm (same as curve tolerance)
- **Enabled by default:** Yes
- **PlotConfig required everywhere:** Yes

## Checklist

### Part 1: Core Merge Implementation (optimize.rs)
- [ ] Add `MergeResult` struct with strokes + statistics
- [ ] Add `merge_adjacent_strokes(strokes, tolerance) -> MergeResult`
- [ ] Add `styles_match(a, b) -> bool` helper
- [ ] Export from lib.rs

### Part 2: Configuration (config.rs)
- [ ] Add `merge_strokes: bool` (default: `true`)
- [ ] Add `merge_tolerance: f64` (default: `0.05`)
- [ ] Add builder methods: `with_stroke_merging()`, `without_stroke_merging()`

### Part 3: Statistics (stats.rs)
- [ ] Add `merged_strokes: usize` to `DrawingStats`

### Part 4: PreparedDrawing Integration (prepared.rs)
- [ ] Call `merge_adjacent_strokes()` after optimization when config enables it
- [ ] Update stats with merge count

### Part 5: Update AxiDraw APIs (axidraw.rs)
- [ ] Remove legacy `plot_in_background()` that takes raw Drawing
- [ ] Keep only `plot_prepared_in_background()` as the main API
- [ ] Update `plot_strokes()` to require `PlotConfig`
- [ ] Update `plot_strokes_with_events()` to require `PlotConfig`
- [ ] Update `plot_with_events()` to use `PreparedDrawing` internally

### Part 6: Unify sketch-runner with plotta-cli (sketch-runner/src/lib.rs)
- [ ] Change P key handler to use `PreparedDrawing::new()` + `plot_prepared_in_background()`
- [ ] Show stats before plotting (log estimated time)

### Part 7: Unit Tests
- [ ] Empty/single stroke merge cases
- [ ] Adjacent strokes merge correctly
- [ ] Different styles don't merge
- [ ] Multiple consecutive merges
- [ ] Strokes too far apart
- [ ] Closed strokes don't merge
- [ ] Merge statistics accuracy

## Implementation Notes

The merge should happen AFTER stroke reordering (greedy nearest-neighbor) to maximize merge opportunities. The reordering already brings related strokes close together in the sequence.

Current optimize.rs structure:
- optimize_strokes_with_reversal() at line 116-298
- Uses greedy nearest-neighbor with R*-tree spatial index
- Returns OptimizedStrokes with stroke references and reversal flags

Merge should be called after optimization to combine sequential strokes that ended up adjacent.

### Current Plotting Paths (to be unified)

| Aspect      | sketch-runner (P key)          | plotta-cli                     |
| ----------- | ------------------------------ | ------------------------------ |
| Preparation | Flatten/optimize inside thread | `PreparedDrawing::new()` upfront |
| Function    | `plot_in_background()`           | `plot_prepared_in_background()`  |
| Config      | Always `PlotConfig::default()`   | Configurable via CLI           |

After this change, both will use `PreparedDrawing` + `plot_prepared_in_background()`.

## Summary of Changes

Implemented the core stroke-merging optimization (enabled by default).

### Code
- **optimize.rs**: added `MergeResult` struct and `merge_adjacent_strokes(strokes, tolerance) -> MergeResult`. Merges only when styles match (stroke_width within 1e-6, exact color/scale_stroke match), neither stroke is closed, and endpoints are within tolerance. Reversal is baked into the points order of the merged stroke. Coincident junction points are deduped.
- **config.rs**: added `merge_strokes: bool` (default `true`) and `merge_tolerance: f64` (default `0.05` mm). Builder methods `with_stroke_merging(tol)` and `without_stroke_merging()`.
- **stats.rs**: added `merged_strokes: usize` to `DrawingStats`. `stroke_count` now reflects the post-merge count (which is what gets plotted).
- **prepared.rs**: `PreparedDrawing::new` calls `merge_adjacent_strokes` after the optimizer when enabled; reversed-stroke count is captured before merging (merge bakes reversal into points order).
- **plotta-cli**: passes through the new defaults in `build_config` and surfaces the merged count in the stats banner (`Strokes: N (R reversed, M merged)`).
- **lib.rs**: re-exports `MergeResult` and `merge_adjacent_strokes`.

### Tests
9 new unit tests covering: empty input, single stroke, coincident endpoints, within-tolerance gap, outside-tolerance gap, respect for `reversed` flag, style mismatch, closed-stroke skip, chained merges, and pen-down distance preservation. All 285 workspace tests pass; clippy clean.

### Scope deferred (not in this change)
The original bean also bundled "unify sketch-runner with plotta-cli by going through PreparedDrawing" and the related AxiDraw API cleanup. The user's request was scoped to the merge optimization itself, so the sketch-runner path still flattens/optimizes inline (it now picks up merging automatically via PlotConfig::default()). Worth a follow-up bean if we want to delete `plot_in_background` and require PlotConfig everywhere.
