---
# plotta-studio-opt2
title: Implement 2-opt algorithm
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-opt1
---

Implement the 2-opt local search algorithm to improve upon the greedy solution.

## Algorithm Description

2-opt works by:
1. Start with an initial tour (greedy solution)
2. For each pair of edges (i, i+1) and (j, j+1):
   - Calculate cost of swapping to (i, j) and (i+1, j+1)
   - If improvement, apply swap
3. Repeat until no improvement found

## Implementation

```rust
/// Improve stroke order using 2-opt
pub fn two_opt_improve(order: &mut Vec<usize>, strokes: &[Stroke]) -> f64 {
    let n = order.len();
    if n < 4 {
        return calculate_travel(&order, strokes);
    }

    let mut improved = true;
    while improved {
        improved = false;

        for i in 0..n - 2 {
            for j in i + 2..n {
                if j == n - 1 && i == 0 {
                    continue; // Skip if would just reverse entire tour
                }

                let delta = calculate_swap_delta(order, strokes, i, j);

                if delta < -0.001 {
                    // Apply swap: reverse segment from i+1 to j
                    order[i + 1..=j].reverse();
                    improved = true;
                }
            }
        }
    }

    calculate_travel(&order, strokes)
}

fn calculate_swap_delta(order: &[usize], strokes: &[Stroke], i: usize, j: usize) -> f64 {
    // Current edges: (i -> i+1) and (j -> j+1)
    // New edges: (i -> j) and (i+1 -> j+1)

    let end_i = stroke_end(&strokes[order[i]]);
    let start_i1 = stroke_start(&strokes[order[i + 1]]);
    let end_j = stroke_end(&strokes[order[j]]);
    let start_j1 = if j + 1 < order.len() {
        stroke_start(&strokes[order[j + 1]])
    } else {
        Point::ZERO // Return to origin
    };

    let old_cost = end_i.distance(start_i1) + end_j.distance(start_j1);

    // After reversal, i+1 becomes j's position (but reversed)
    let new_start_i1 = stroke_end(&strokes[order[j]]); // reversed j
    let new_end_j = stroke_start(&strokes[order[i + 1]]); // reversed i+1

    let new_cost = end_i.distance(new_start_i1) + new_end_j.distance(start_j1);

    new_cost - old_cost
}
```

## Complexity
- Worst case: O(n³) if many improvements
- Typical: O(n²) for each pass, few passes needed
- For 1000 strokes: typically < 1 second

## Files to Modify
- `crates/drawing-plotter/src/lib.rs` or new `optimization.rs` module
