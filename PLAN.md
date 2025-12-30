# Implementation Plan: Tolerance-Based Curve Subdivision

## Overview

Replace the current hardcoded segment-count-based curve flattening with adaptive tolerance-based subdivision using kurbo's `flatten` function. The error tolerance will be configurable with a default of 0.05mm. Additionally, implement path cleanup to remove redundant collinear points.

## Current State

### Hardcoded Segment Counts

| Shape | Current Segments | Location |
|-------|-----------------|----------|
| Circle | 64 | `flatten.rs:12-24` |
| Ellipse | 64 | `flatten.rs:26-38` |
| Arc | 32 | `flatten.rs:40-53` |
| QuadBez (paths) | 16 | `flatten.rs:100` |
| CubicBez (paths) | 24 | `flatten.rs:110` |

### Existing Tolerance-Based Flattening (fonts only)

In `font_types.rs:172-185`, font contours use kurbo's `BezPath::flatten(tolerance, callback)` which is the pattern we want to adopt globally.

## Implementation Steps

### Step 1: Add Tolerance to RenderContext

**File: `crates/drawing-core/src/context.rs`**

Add a `tolerance` field to `RenderContext` with a sensible default (0.05mm = 0.05 in plotter units).

```rust
pub struct RenderContext {
    font_registry: Arc<FontRegistry>,
    /// Curve flattening tolerance in mm (default: 0.05)
    pub tolerance: f64,
}

impl RenderContext {
    pub fn new(font_registry: Arc<FontRegistry>) -> Self {
        Self {
            font_registry,
            tolerance: 0.05, // 0.05mm default
        }
    }

    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    pub fn empty() -> Self {
        Self {
            font_registry: Arc::new(FontRegistry::new()),
            tolerance: 0.05,
        }
    }
}
```

### Step 2: Refactor Flatten Functions to Use Tolerance

**File: `crates/drawing-core/src/flatten.rs`**

#### 2a. Convert Circle to BezPath and Flatten

Replace manual parametric sampling with kurbo's built-in circle-to-bezpath and flatten:

```rust
use kurbo::{Circle as KurboCircle, Shape};

pub fn flatten_circle(circle: &Circle, transform: &Affine, tolerance: f64) -> Vec<Point> {
    // Create kurbo Circle and get its BezPath representation
    let kurbo_circle = KurboCircle::new(circle.center, circle.radius);
    let bezpath = kurbo_circle.to_path(tolerance);

    // Flatten with tolerance and apply transform
    let mut points = Vec::new();
    kurbo::flatten(bezpath, tolerance, |el| {
        match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => {
                points.push(*transform * p);
            }
            _ => {}
        }
    });

    points
}
```

#### 2b. Convert Ellipse to BezPath and Flatten

```rust
use kurbo::{Ellipse as KurboEllipse, Shape};

pub fn flatten_ellipse(ellipse: &Ellipse, transform: &Affine, tolerance: f64) -> Vec<Point> {
    let kurbo_ellipse = KurboEllipse::new(ellipse.center, (ellipse.rx, ellipse.ry), 0.0);
    let bezpath = kurbo_ellipse.to_path(tolerance);

    let mut points = Vec::new();
    kurbo::flatten(bezpath, tolerance, |el| {
        match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => {
                points.push(*transform * p);
            }
            _ => {}
        }
    });

    points
}
```

#### 2c. Convert Arc to BezPath and Flatten

```rust
use kurbo::Arc as KurboArc;

pub fn flatten_arc(arc: &Arc, transform: &Affine, tolerance: f64) -> Vec<Point> {
    // Create kurbo Arc
    let kurbo_arc = KurboArc::new(
        arc.center,
        (arc.radius, arc.radius),  // radii (x, y)
        arc.start_angle,
        arc.end_angle - arc.start_angle,  // sweep angle
        0.0,  // x_rotation
    );

    let bezpath = kurbo_arc.to_path(tolerance);

    let mut points = Vec::new();
    kurbo::flatten(bezpath, tolerance, |el| {
        match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => {
                points.push(*transform * p);
            }
            _ => {}
        }
    });

    points
}
```

#### 2d. Refactor flatten_path to Use Tolerance

Replace hardcoded step counts with kurbo's `flatten`:

```rust
pub fn flatten_path(path: &Path, transform: &Affine, style: ResolvedStyle, tolerance: f64) -> Vec<Stroke> {
    let bezpath = path.to_bezpath();
    let mut strokes = Vec::new();
    let mut current_points: Vec<Point> = Vec::new();
    let mut is_closed = false;
    let mut start_point = Point::ZERO;

    // Use kurbo::flatten for adaptive subdivision
    kurbo::flatten(bezpath, tolerance, |el| {
        match el {
            PathEl::MoveTo(p) => {
                if current_points.len() > 1 {
                    let mut stroke = Stroke::new(std::mem::take(&mut current_points), style);
                    stroke.closed = is_closed;
                    strokes.push(stroke);
                } else {
                    current_points.clear();
                }
                is_closed = false;
                start_point = p;
                current_points.push(*transform * p);
            }
            PathEl::LineTo(p) => {
                current_points.push(*transform * p);
            }
            PathEl::ClosePath => {
                current_points.push(*transform * start_point);
                is_closed = true;
            }
            _ => {} // QuadTo and CurveTo are flattened to LineTo by kurbo::flatten
        }
    });

    if current_points.len() > 1 {
        let mut stroke = Stroke::new(current_points, style);
        stroke.closed = is_closed;
        strokes.push(stroke);
    }

    strokes
}
```

### Step 3: Add Path Simplification (Remove Redundant Points)

**File: `crates/drawing-core/src/simplify.rs` (new file)**

Add a module to remove collinear points from flattened paths:

```rust
//! Path simplification - remove redundant collinear points

use kurbo::Point;

/// Remove collinear points from a path
///
/// Points are considered collinear if the perpendicular distance from
/// the middle point to the line formed by the neighbors is less than `tolerance`.
pub fn simplify_points(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut result = Vec::with_capacity(points.len());
    result.push(points[0]);

    for i in 1..points.len() - 1 {
        let prev = result.last().unwrap();
        let curr = points[i];
        let next = points[i + 1];

        // Calculate perpendicular distance from curr to line prev->next
        if !is_collinear(*prev, curr, next, tolerance) {
            result.push(curr);
        }
    }

    // Always keep the last point
    result.push(*points.last().unwrap());

    result
}

/// Check if three points are collinear within tolerance
fn is_collinear(a: Point, b: Point, c: Point, tolerance: f64) -> bool {
    // Vector from a to c
    let ac = c - a;
    let ac_len = ac.hypot();

    if ac_len < f64::EPSILON {
        return true; // a and c are the same point
    }

    // Vector from a to b
    let ab = b - a;

    // Perpendicular distance = |cross product| / |ac|
    let cross = ab.x * ac.y - ab.y * ac.x;
    let distance = cross.abs() / ac_len;

    distance <= tolerance
}

/// Remove duplicate consecutive points
pub fn remove_duplicates(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.is_empty() {
        return vec![];
    }

    let mut result = Vec::with_capacity(points.len());
    result.push(points[0]);

    for &p in &points[1..] {
        if let Some(&last) = result.last() {
            if last.distance(p) > tolerance {
                result.push(p);
            }
        }
    }

    result
}
```

### Step 4: Integrate Simplification into Flatten Pipeline

**File: `crates/drawing-core/src/flatten.rs`**

Add simplification as a post-processing step:

```rust
use crate::simplify::{simplify_points, remove_duplicates};

/// Apply simplification to flattened points
fn cleanup_points(points: Vec<Point>, tolerance: f64) -> Vec<Point> {
    // First remove duplicate points
    let points = remove_duplicates(&points, tolerance * 0.1);
    // Then remove collinear points
    simplify_points(&points, tolerance)
}
```

Apply this to all flatten functions before returning points.

### Step 5: Update Function Signatures Throughout Codebase

**File: `crates/drawing-core/src/element.rs`**

Update `flatten_with_inherited` to pass tolerance from context:

```rust
pub(crate) fn flatten_with_inherited(
    &self,
    ctx: &RenderContext,
    parent_transform: Affine,
    parent_style: &ResolvedStyle,
) -> Vec<Stroke> {
    let tolerance = ctx.tolerance;
    // ...

    match &self.shape {
        Shape::Circle(circle) => {
            let points = flatten_circle(circle, &transform, tolerance);
            // cleanup applied inside flatten_circle
            vec![Stroke { points, style: scaled_style, closed: true }]
        }
        // ... similar for other shapes
    }
}
```

### Step 6: Remove Segment Fields from Primitives (Optional)

**File: `crates/drawing-core/src/primitives.rs`**

Since we're moving to tolerance-based subdivision, the `segments` field becomes unnecessary. However, for backwards compatibility, we can deprecate rather than remove:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
    #[deprecated(note = "Use RenderContext tolerance instead")]
    #[serde(skip_serializing, default = "default_segments")]
    pub segments: usize,
}

fn default_segments() -> usize { 64 }
```

### Step 7: Add Configuration to CLI

**File: `crates/plotta-cli/src/main.rs`**

Add a `--tolerance` flag:

```rust
#[derive(Parser)]
struct Cli {
    // ...

    /// Curve flattening tolerance in mm (default: 0.05)
    #[arg(long, default_value = "0.05")]
    tolerance: f64,
}
```

Update render context creation to use the tolerance.

### Step 8: Update Tests

Update existing tests to account for:
1. Variable point counts (no longer fixed segment counts)
2. New tolerance parameter
3. Point count may be higher or lower depending on curve complexity

## Files to Modify

1. `crates/drawing-core/src/context.rs` - Add tolerance field
2. `crates/drawing-core/src/flatten.rs` - Refactor all flatten functions
3. `crates/drawing-core/src/simplify.rs` - New file for path simplification
4. `crates/drawing-core/src/lib.rs` - Export new module
5. `crates/drawing-core/src/element.rs` - Pass tolerance through
6. `crates/drawing-core/src/primitives.rs` - Deprecate segments field
7. `crates/plotta-cli/src/main.rs` - Add --tolerance flag
8. Various test files - Update assertions

## Benefits

1. **Quality**: Curves are approximated based on actual error, not arbitrary segment counts
2. **Efficiency**: Simple curves use fewer points, complex curves get more detail
3. **Consistency**: Same algorithm for all curve types
4. **Configurable**: Users can adjust tolerance for their needs (pen plotter vs laser cutter)
5. **Cleaner output**: Redundant collinear points are removed

## Notes

- kurbo's `flatten` function uses an optimized algorithm based on the Flattening Quadratic Béziers paper
- The tolerance is the maximum Hausdorff distance between the curve and the polyline approximation
- For plotters, 0.05mm is a good default (imperceptible to the eye, efficient)
- For preview/UI, 0.25mm is recommended by kurbo docs
