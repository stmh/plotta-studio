---
# plotta-studio-kmfg
title: Implement ClipGroup for scenegraph clipping
status: completed
type: feature
priority: normal
created_at: 2025-12-27T14:03:47Z
updated_at: 2025-12-27T14:12:44Z
---

Add ClipGroup to the scenegraph that clips its children to a closed shape. Enables hatched shapes, clipping to paper bounds, and nested clips.

## Overview

- ClipGroup wraps a clip Element (any closed shape) and children Elements
- At flatten time, children are clipped against the clip region using the geo crate
- Multiple clip shapes in a Group use union semantics
- Nested ClipGroups intersect their clip regions
- Transforms apply to both clip shape and content (consistent with Group)

## Use Cases

1. Hatched shapes (lines clipped to circle/polygon)
2. Clipping drawings to paper/plotter bounds
3. Nested clips for complex masking

## Design

### Architecture

**New types:**

```rust
// In shape.rs - add to Shape enum
pub enum Shape {
    // ... existing variants
    ClipGroup(ClipGroup),
}

// In new file: clip.rs
pub struct ClipGroup {
    /// The clipping shape (must be closed when flattened)
    pub clip: Box<Element>,
    /// Children to be clipped
    pub children: Vec<Element>,
}
```

**Flatten behavior:**

1. Flatten the clip shape to get a closed polygon
2. Flatten all children to get strokes
3. For each stroke, clip it against the polygon using `geo` crate
4. Return only the segments inside the clip region

**Nested clips:** When a ClipGroup contains another ClipGroup, the inner clip is intersected with the outer clip polygon before clipping its children.

### Geo Conversions (using From/Into traits)

```rust
// kurbo Point <-> geo Coord
impl From<Point> for geo::Coord<f64> {
    fn from(p: Point) -> Self {
        geo::Coord { x: p.x, y: p.y }
    }
}

impl From<geo::Coord<f64>> for Point {
    fn from(c: geo::Coord<f64>) -> Self {
        Point::new(c.x, c.y)
    }
}

// Stroke -> geo LineString
impl From<&Stroke> for geo::LineString<f64> {
    fn from(stroke: &Stroke) -> Self {
        geo::LineString::new(stroke.points.iter().map(|p| (*p).into()).collect())
    }
}

// geo LineString -> Vec<Point>
impl From<&geo::LineString<f64>> for Vec<Point> {
    fn from(ls: &geo::LineString<f64>) -> Self {
        ls.coords().map(|c| (*c).into()).collect()
    }
}
```

### Element API

```rust
impl Element {
    /// Create a clip group with a clip shape
    pub fn clip(clip_shape: Element) -> Self {
        Self::new(ClipGroup::new(clip_shape))
    }
}

impl ClipGroup {
    pub fn new(clip: Element) -> Self {
        Self {
            clip: Box::new(clip),
            children: Vec::new(),
        }
    }

    pub fn add(mut self, element: Element) -> Self {
        self.children.push(element);
        self
    }

    pub fn push(&mut self, element: Element) {
        self.children.push(element);
    }
}
```

**Usage:**

```rust
// Hatched circle
let hatched = Element::clip(Element::circle((50.0, 50.0), 40.0))
    .add(horizontal_lines)
    .add(vertical_lines)
    .rotate_deg(45.0);

// Clip to paper bounds
let clipped_drawing = Element::clip(Element::rect(0.0, 0.0, 297.0, 210.0))
    .add(my_artwork);

// Multiple shapes defining clip area (union)
Element::clip(Element::group(
    Group::new()
        .add(Element::circle((30.0, 50.0), 25.0))
        .add(Element::circle((70.0, 50.0), 25.0))
))
    .add(hatch_lines)
```

### Flatten Integration

In `element.rs`, add the ClipGroup match arm:

```rust
Shape::ClipGroup(clip_group) => {
    clip_group.flatten_with_transform(ctx, transform)
}
```

In `clip.rs`:

```rust
impl ClipGroup {
    pub(crate) fn flatten_with_transform(
        &self,
        ctx: &RenderContext,
        parent_transform: Affine,
    ) -> Vec<Stroke> {
        // 1. Flatten clip shape to get clip polygons
        let clip_strokes = self.clip.flatten_with_transform(ctx, parent_transform);
        let clip_polygons: Vec<geo::Polygon<f64>> = clip_strokes
            .iter()
            .filter_map(|s| s.try_into().ok())  // Only closed strokes become polygons
            .collect();

        if clip_polygons.is_empty() {
            return vec![]; // No valid clip region = nothing visible
        }

        // 2. Flatten children
        let child_strokes: Vec<Stroke> = self.children
            .iter()
            .flat_map(|child| child.flatten_with_transform(ctx, parent_transform))
            .collect();

        // 3. Clip each stroke against the union of clip polygons
        child_strokes
            .iter()
            .flat_map(|stroke| clip_stroke(&stroke, &clip_polygons))
            .collect()
    }
}
```

### Clipping Algorithm

```rust
use geo::{BooleanOps, LineString, MultiLineString, MultiPolygon, Polygon};

/// Clip a stroke against a set of clip polygons (union semantics)
fn clip_stroke(stroke: &Stroke, clip_polygons: &[Polygon<f64>]) -> Vec<Stroke> {
    let clip_region = union_polygons(clip_polygons);

    if stroke.closed {
        // For closed strokes, use polygon intersection
        if let Ok(poly) = stroke.try_into() {
            let clipped = clip_region.intersection(&poly);
            return polygons_to_strokes(&clipped, stroke.style);
        }
    }

    // Open strokes: line intersection
    let line: LineString<f64> = stroke.into();
    let clipped = clip_region.clip(&line.into(), false);
    linestrings_to_strokes(&clipped, stroke.style, false)
}

/// Union multiple polygons into a MultiPolygon
fn union_polygons(polygons: &[Polygon<f64>]) -> MultiPolygon<f64> {
    if polygons.is_empty() {
        return MultiPolygon::new(vec![]);
    }

    let mut result = MultiPolygon::new(vec![polygons[0].clone()]);
    for poly in &polygons[1..] {
        result = result.union(&MultiPolygon::new(vec![poly.clone()]));
    }
    result
}
```

### File Structure

```
crates/drawing-core/
├── src/
│   ├── lib.rs          # Add: mod clip; pub use clip::ClipGroup;
│   ├── clip.rs         # NEW: ClipGroup struct, flatten, geo conversions
│   ├── shape.rs        # Add: ClipGroup variant to Shape enum
│   └── element.rs      # Add: Element::clip() constructor, flatten match arm
└── Cargo.toml          # Add: geo = "0.32"
```

## Checklist

- [x] Add `geo` dependency to `drawing-core/Cargo.toml`
- [x] Create `clip.rs` with ClipGroup struct and geo conversions
- [x] Add ClipGroup variant to Shape enum
- [x] Add Element::clip() constructor
- [x] Implement flatten_with_transform for ClipGroup
- [x] Implement clip_stroke and union_polygons
- [x] Add tests for basic clipping
- [x] Add tests for boundary crossing (stroke splitting)
- [x] Add tests for multiple clip shapes (union)
- [x] Add tests for nested clips (intersection)
- [x] Add tests for invalid/empty clip shapes