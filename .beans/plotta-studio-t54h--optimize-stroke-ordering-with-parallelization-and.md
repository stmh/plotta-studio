---
# plotta-studio-t54h
title: Optimize stroke ordering with parallelization and spatial indexing
status: completed
type: feature
priority: high
created_at: 2025-12-30T17:32:30Z
updated_at: 2025-12-31T14:31:50Z
parent: plotta-studio-opt1
---

Research and implementation plan for speeding up stroke optimization for large drawings (100k+ strokes).

## Problem

The current greedy nearest-neighbor algorithm in `optimize_strokes_with_reversal` has O(n²) complexity. For a 135k stroke drawing, this means ~18 billion distance calculations, which is extremely slow.

## Research Findings

- Large drawings (e.g., Hamburg SVG) have 135,000+ paths
- Current algorithm: O(n²) sequential
- The `remaining.remove(best_idx)` operation is also O(n), adding to the problem

## Implementation Results

### Phase 1: Rayon Parallelization - ABANDONED

Attempted to parallelize the inner loop using rayon, but it **made things slower**:

| Version          | Wall Time | CPU Usage       |
| ---------------- | --------- | --------------- |
| Sequential       | 31s       | 98% (1 core)    |
| Parallel (rayon) | 61s       | 531% (5+ cores) |

**Why it failed:** The algorithm is O(n²) with 134,699 iterations. Each iteration calls `par_iter()` which has thread pool overhead (~1-5μs). The inner loop work (distance calculations) is too cheap compared to thread overhead.

### Phase 2: R*-tree Spatial Indexing - IMPLEMENTED

Used `rstar` crate (R*-tree) for O(n log n) nearest-neighbor queries.

**Algorithm:**
1. Build R*-tree with all stroke endpoints (start + end): O(n log n)
2. Track visited strokes with HashSet: O(1) lookup
3. For each iteration, query nearest unvisited neighbor: O(log n)
4. Total complexity: O(n log n) vs O(n²)

**Performance Results (134,699 strokes):**

| Metric | Before (O(n²)) | After (O(n log n)) | Improvement |
|--------|----------------|---------------------|-------------|
| Optimization time | ~31 seconds | 374 milliseconds | **83x faster** |
| R*-tree build | N/A | 50ms | - |
| Memory overhead | O(n) | O(n) | Similar |

The R*-tree approach maintains the same optimization quality (travel distance) while providing massive speedups for large drawings.

## Checklist

- [x] Research and choose spatial indexing crate (rstar chosen over kiddo due to dependency issues)
- [x] Implement R*-tree based optimization in `optimize_strokes_internal`
- [x] Update tests for new implementation (all 92 tests pass)
- [x] Document performance characteristics
- [ ] ~~Add rayon dependency~~ (abandoned - made things slower)
- [ ] ~~Implement parallel inner loop~~ (abandoned)
- [ ] ~~Add feature flag to choose optimization strategy~~ (not needed - R*-tree is always better)