---
# plotta-studio-t54h
title: Optimize stroke ordering with parallelization and spatial indexing
status: todo
type: feature
priority: high
created_at: 2025-12-30T17:32:30Z
updated_at: 2025-12-30T17:32:30Z
---

Research and implementation plan for speeding up stroke optimization for large drawings (100k+ strokes).

## Problem

The current greedy nearest-neighbor algorithm in `optimize_strokes_with_reversal` has O(n²) complexity. For a 135k stroke drawing, this means ~18 billion distance calculations, which is extremely slow.

## Research Findings

- Large drawings (e.g., Hamburg SVG) have 135,000+ paths
- Current algorithm: O(n²) sequential
- The `remaining.remove(best_idx)` operation is also O(n), adding to the problem

## Proposed Solutions

### Phase 1: Rayon Parallelization (Quick Win)

Parallelize the inner loop that finds the nearest stroke:

```rust
use rayon::prelude::*;

let (best_idx, best_dist, best_reversed) = remaining
    .par_iter()
    .enumerate()
    .map(|(i, (_, stroke))| {
        // calculate distances...
        (i, dist, reversed)
    })
    .reduce(|| (0, f64::MAX, false), |a, b| {
        if a.1 < b.1 { a } else { b }
    });
```

**Pros:**
- Simple to implement
- No algorithmic changes
- ~4-8x speedup on multi-core

**Cons:**
- Still O(n²) algorithm
- Thread overhead for each of n iterations
- The `remaining.remove(best_idx)` is still O(n)

**Expected improvement:** ~4-8x faster

### Phase 2: Spatial Indexing (Major Improvement)

Use a k-d tree (e.g., `kiddo` or `rstar` crate) to find nearest neighbors:

```rust
use kiddo::KdTree;

// Build spatial index once: O(n log n)
let mut tree: KdTree<f64, usize, 2> = KdTree::new();
for (i, stroke) in strokes.iter().enumerate() {
    tree.add(&[stroke.start().x, stroke.start().y], i);
    tree.add(&[stroke.end().x, stroke.end().y], i);  // for reversal
}

// Each lookup is O(log n) instead of O(n)
```

**Pros:**
- O(n log n) total complexity instead of O(n²)
- Much better for large stroke counts
- Can be combined with rayon for tree construction

**Cons:**
- More complex implementation
- Additional dependency
- Need to handle removal from tree (or use visited set)

**Expected improvement:** ~1000x+ faster for 135k strokes

### Potential Dependencies

- `rayon` - for parallel iteration
- `kiddo` or `rstar` - for spatial indexing (k-d tree or R-tree)

## Checklist

- [ ] Add rayon dependency to drawing-plotter
- [ ] Implement parallel inner loop in `optimize_strokes_with_reversal`
- [ ] Add benchmarks to measure improvement
- [ ] Research and choose spatial indexing crate (kiddo vs rstar)
- [ ] Implement k-d tree based optimization
- [ ] Add feature flag to choose optimization strategy
- [ ] Update tests for new implementation
- [ ] Document performance characteristics