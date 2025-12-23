---
# plotta-studio-opt1
title: Path optimization for plotting
status: todo
type: epic
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
---

Improve path optimization algorithms to minimize pen-up travel time when plotting, making plotter output faster and more efficient.

## Investigation

### Current State
- `drawing-plotter/src/lib.rs` has basic greedy nearest-neighbor at line 80-110
- `optimize_strokes()` function finds nearest stroke start from current position
- `total_travel_distance()` and `pen_down_distance()` metrics exist

### Problem: Traveling Salesman Problem (TSP)
Optimizing stroke order is essentially a TSP variant:
- Each stroke has a start and end point
- Strokes can potentially be reversed (if not directional)
- Goal: minimize total pen-up travel distance

### Optimization Algorithms to Implement

#### 1. Greedy Nearest Neighbor (Current)
- O(n²) complexity
- Quick but often suboptimal (10-25% longer than optimal)
- Good baseline

#### 2. 2-opt Improvement
- Take greedy solution and improve it
- Swap pairs of edges if it reduces distance
- O(n²) per iteration, typically converges quickly
- Can improve greedy by 5-15%

#### 3. Reversible Strokes
- Some strokes can be drawn in either direction
- Double the search space but can significantly reduce travel
- Need to track which strokes are reversible

#### 4. Simulated Annealing (Optional)
- Probabilistic optimization
- Can escape local minima
- Good for large stroke counts (>1000)

### API Design

```rust
/// Optimization strategy
pub enum OptimizationLevel {
    /// No optimization - plot in original order
    None,
    /// Greedy nearest neighbor (fast)
    Greedy,
    /// Greedy + 2-opt improvement (recommended)
    TwoOpt,
    /// Full optimization with annealing (slow but best)
    Full,
}

pub struct OptimizationResult {
    pub strokes: Vec<StrokeRef>,
    pub total_distance: f64,
    pub pen_up_distance: f64,
    pub pen_down_distance: f64,
    pub improvement_percent: f64,
}

/// Reference to a stroke, potentially reversed
pub struct StrokeRef {
    pub index: usize,
    pub reversed: bool,
}

pub fn optimize(strokes: &[Stroke], level: OptimizationLevel) -> OptimizationResult;
```

### Metrics to Report
- Total travel distance
- Pen-up travel distance
- Pen-down travel distance
- Number of strokes
- Estimated plot time
- Improvement over unoptimized
