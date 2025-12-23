---
# plotta-studio-opt3
title: Add stroke reversal support
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-opt1
---

Allow strokes to be drawn in reverse direction when it reduces travel distance.

## Concept

Many strokes can be drawn in either direction without affecting the result:
- Lines
- Polylines (unless directional markers)
- Paths without special start/end requirements

## Implementation

```rust
/// A stroke reference that may be reversed
#[derive(Clone, Copy)]
pub struct StrokeRef {
    pub index: usize,
    pub reversed: bool,
}

impl StrokeRef {
    pub fn start<'a>(&self, strokes: &'a [Stroke]) -> Point {
        let stroke = &strokes[self.index];
        if self.reversed {
            *stroke.points.last().unwrap_or(&Point::ZERO)
        } else {
            stroke.points.first().copied().unwrap_or(Point::ZERO)
        }
    }

    pub fn end<'a>(&self, strokes: &'a [Stroke]) -> Point {
        let stroke = &strokes[self.index];
        if self.reversed {
            stroke.points.first().copied().unwrap_or(Point::ZERO)
        } else {
            *stroke.points.last().unwrap_or(&Point::ZERO)
        }
    }

    pub fn points<'a>(&self, strokes: &'a [Stroke]) -> impl Iterator<Item = Point> + 'a {
        let stroke = &strokes[self.index];
        let pts: Vec<_> = if self.reversed {
            stroke.points.iter().rev().copied().collect()
        } else {
            stroke.points.iter().copied().collect()
        };
        pts.into_iter()
    }
}

/// Optimize with reversal support
pub fn optimize_with_reversal(strokes: &[Stroke]) -> Vec<StrokeRef> {
    let mut refs: Vec<StrokeRef> = (0..strokes.len())
        .map(|i| StrokeRef { index: i, reversed: false })
        .collect();

    let mut current_pos = Point::ZERO;

    for i in 0..refs.len() {
        // Find best next stroke (considering reversal)
        let mut best_idx = i;
        let mut best_dist = f64::MAX;
        let mut best_reversed = false;

        for j in i..refs.len() {
            let stroke = &strokes[refs[j].index];

            // Try normal direction
            let dist_normal = current_pos.distance(stroke.points[0]);
            if dist_normal < best_dist {
                best_dist = dist_normal;
                best_idx = j;
                best_reversed = false;
            }

            // Try reversed
            if let Some(last) = stroke.points.last() {
                let dist_rev = current_pos.distance(*last);
                if dist_rev < best_dist {
                    best_dist = dist_rev;
                    best_idx = j;
                    best_reversed = true;
                }
            }
        }

        refs.swap(i, best_idx);
        refs[i].reversed = best_reversed;
        current_pos = refs[i].end(strokes);
    }

    refs
}
```

## Files to Modify
- `crates/drawing-plotter/src/lib.rs`
