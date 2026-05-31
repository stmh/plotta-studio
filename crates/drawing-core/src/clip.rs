//! ClipGroup - clips children to a closed shape

use geo::{BooleanOps, Coord, LineString, MultiLineString, MultiPolygon, Polygon};
use kurbo::Affine;
use serde::{Deserialize, Serialize};

use crate::context::RenderContext;
use crate::stroke::Stroke;
use crate::style::ResolvedStyle;
use crate::Element;
use crate::Point;

/// Tolerance for geometric distance comparisons (in coordinate units, typically mm).
/// This is used to determine if two points are "close enough" to be considered the same.
const DISTANCE_TOLERANCE: f64 = 1e-6;

/// Tolerance for comparing normalized parameters (t values in 0..1 range).
/// Used for deduplicating intersection points along a line segment.
const PARAMETER_TOLERANCE: f64 = 1e-6;

/// A group that clips its children to a closed shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipGroup {
    /// The clipping shape (must be closed when flattened)
    pub clip: Box<Element>,
    /// Children to be clipped
    pub children: Vec<Element>,
    /// If true, keep content outside the clip region instead of inside
    #[serde(default)]
    pub invert: bool,
}

impl ClipGroup {
    pub fn new(clip: Element) -> Self {
        Self {
            clip: Box::new(clip),
            children: Vec::new(),
            invert: false,
        }
    }

    /// Set whether to invert the clipping (keep outside instead of inside)
    pub fn invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }

    // We use `add` for builder pattern, not arithmetic addition
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, element: Element) -> Self {
        self.children.push(element);
        self
    }

    pub fn push(&mut self, element: Element) {
        self.children.push(element);
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub(crate) fn flatten_with_inherited(
        &self,
        ctx: &RenderContext,
        parent_transform: Affine,
        parent_style: &ResolvedStyle,
    ) -> Vec<Stroke> {
        // 1. Flatten clip shape to get clip polygons
        let clip_strokes = self
            .clip
            .flatten_with_inherited(ctx, parent_transform, parent_style);
        let clip_polygons: Vec<Polygon<f64>> =
            clip_strokes.iter().filter_map(stroke_to_polygon).collect();

        if clip_polygons.is_empty() {
            return vec![]; // No valid clip region = nothing visible
        }

        // 2. Flatten children
        let child_strokes: Vec<Stroke> = self
            .children
            .iter()
            .flat_map(|child| child.flatten_with_inherited(ctx, parent_transform, parent_style))
            .collect();

        // 3. Clip each stroke against the union of clip polygons
        child_strokes
            .iter()
            .flat_map(|stroke| clip_stroke(stroke, &clip_polygons, self.invert))
            .collect()
    }
}

// ============================================================================
// Geo Conversions
// ============================================================================

fn point_to_coord(p: Point) -> Coord<f64> {
    Coord { x: p.x, y: p.y }
}

fn coord_to_point(c: Coord<f64>) -> Point {
    Point::new(c.x, c.y)
}

fn stroke_to_linestring(stroke: &Stroke) -> LineString<f64> {
    let mut coords: Vec<_> = stroke.points.iter().map(|p| point_to_coord(*p)).collect();
    // For closed strokes, explicitly add the first point at the end to close the path
    // This ensures the closing edge (last point -> first point) is included in clipping
    if stroke.closed && coords.len() >= 2 {
        coords.push(coords[0]);
    }
    LineString::new(coords)
}

fn linestring_to_points(ls: &LineString<f64>) -> Vec<Point> {
    ls.coords().map(|c| coord_to_point(*c)).collect()
}

/// Convert a closed stroke to a geo Polygon
fn stroke_to_polygon(stroke: &Stroke) -> Option<Polygon<f64>> {
    if !stroke.closed || stroke.points.len() < 3 {
        return None;
    }
    let exterior = stroke_to_linestring(stroke);
    Some(Polygon::new(exterior, vec![]))
}

// ============================================================================
// Clipping Algorithm
// ============================================================================

/// Clip a stroke against a set of clip polygons (union semantics)
///
/// For pen plotting, all strokes are treated as lines (not filled shapes),
/// so we always use line clipping even for closed paths.
///
/// If `invert` is true, keeps content outside the clip region instead of inside.
fn clip_stroke(stroke: &Stroke, clip_polygons: &[Polygon<f64>], invert: bool) -> Vec<Stroke> {
    let clip_region = union_polygons(clip_polygons);

    // Always use line clipping for pen plotting
    // Even closed strokes are just outlines that need to be clipped as lines
    let line = stroke_to_linestring(stroke);
    let clipped = clip_linestring_to_region(&line, &clip_region, invert);

    // Preserve the closed flag if the original stroke was closed and we got a single result
    let mut result = linestrings_to_strokes(&clipped, stroke.style);

    // If original was closed and result has exactly one segment that forms a complete loop,
    // keep it closed. Otherwise, the clipping broke it into segments.
    if stroke.closed && result.len() == 1 {
        if let Some(first) = result.first_mut() {
            // Check if it's still a closed loop (first and last point are close)
            if first.points.len() >= 3 {
                let first_pt = first.points.first().unwrap();
                let last_pt = first.points.last().unwrap();
                if first_pt.distance(*last_pt) < DISTANCE_TOLERANCE {
                    first.closed = true;
                }
            }
        }
    }

    result
}

/// Union multiple polygons into a MultiPolygon
///
/// Note: O(n²) complexity for n polygons due to repeated union operations.
/// This is acceptable for typical use (1-3 clip shapes) but could be slow with many shapes.
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

/// Test whether a point lies within the (closed) clip region.
///
/// The clip region is treated as a *closed* set: points lying exactly on the
/// polygon boundary count as inside. `geo`'s `Contains` is strict (boundary
/// points return `false`), which causes strokes whose endpoints sit on the clip
/// edge to be silently dropped (see issue #19). `Intersects` includes the
/// boundary, giving the closed-region semantics we want.
fn point_in_closed_region(clip_region: &MultiPolygon<f64>, point: &geo::Point<f64>) -> bool {
    use geo::Intersects;
    clip_region.intersects(point)
}

/// Clip a linestring to a multi-polygon region
///
/// If `invert` is true, keeps content outside the clip region instead of inside.
fn clip_linestring_to_region(
    line: &LineString<f64>,
    clip_region: &MultiPolygon<f64>,
    invert: bool,
) -> MultiLineString<f64> {
    use geo::{Line as GeoLine, LineIntersection};

    let mut result_lines: Vec<LineString<f64>> = vec![];
    let mut current_segment: Vec<Coord<f64>> = vec![];

    let coords: Vec<_> = line.coords().collect();
    if coords.len() < 2 {
        return MultiLineString::new(vec![]);
    }

    // Get all boundary lines from the clip region
    let boundary_lines: Vec<GeoLine<f64>> = clip_region
        .0
        .iter()
        .flat_map(|poly| {
            let exterior_lines = poly.exterior().lines();
            let interior_lines = poly.interiors().iter().flat_map(|ring| ring.lines());
            exterior_lines.chain(interior_lines)
        })
        .collect();

    for i in 0..coords.len() {
        let current = coords[i];
        let current_point = geo::Point::new(current.x, current.y);
        // When inverted, we want to keep what's outside, so we flip the inside check
        let current_inside = point_in_closed_region(clip_region, &current_point) != invert;

        if i == 0 {
            // First point
            if current_inside {
                current_segment.push(*current);
            }
            continue;
        }

        let prev = coords[i - 1];
        let prev_point = geo::Point::new(prev.x, prev.y);
        let prev_inside = point_in_closed_region(clip_region, &prev_point) != invert;

        let segment = GeoLine::new(*prev, *current);

        // Find intersections with boundary
        let mut intersections: Vec<(f64, Coord<f64>)> = vec![];
        for boundary in &boundary_lines {
            if let Some(intersection) =
                geo::algorithm::line_intersection::line_intersection(segment, *boundary)
            {
                match intersection {
                    LineIntersection::SinglePoint { intersection, .. } => {
                        // Calculate t parameter along segment
                        let dx = current.x - prev.x;
                        let dy = current.y - prev.y;
                        let t = if dx.abs() > 1e-10 && dx.abs() > dy.abs() {
                            (intersection.x - prev.x) / dx
                        } else if dy.abs() > 1e-10 {
                            (intersection.y - prev.y) / dy
                        } else {
                            // Degenerate segment - use distance ratio as fallback
                            let segment_len_sq = dx * dx + dy * dy;
                            if segment_len_sq > 1e-20 {
                                let dx_to_int = intersection.x - prev.x;
                                let dy_to_int = intersection.y - prev.y;
                                ((dx_to_int * dx + dy_to_int * dy) / segment_len_sq).clamp(0.0, 1.0)
                            } else {
                                0.5 // Truly degenerate point
                            }
                        };
                        if t > 1e-10 && t < 1.0 - 1e-10 {
                            intersections.push((
                                t,
                                Coord {
                                    x: intersection.x,
                                    y: intersection.y,
                                },
                            ));
                        }
                    }
                    LineIntersection::Collinear { .. } => {
                        // Handle collinear case if needed
                    }
                }
            }
        }

        // Sort intersections by t parameter, then deduplicate nearby values.
        // Sorting first ensures dedup_by (which only removes consecutive duplicates)
        // will find all duplicates, since equal values become adjacent after sorting.
        // This handles the case where a line passes through a polygon corner,
        // producing two intersection points at nearly the same t value.
        intersections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        intersections.dedup_by(|a, b| (a.0 - b.0).abs() < PARAMETER_TOLERANCE);

        if intersections.is_empty() {
            // No intersection - simple case
            if current_inside {
                if current_segment.is_empty() && prev_inside {
                    current_segment.push(*prev);
                }
                current_segment.push(*current);
            } else if !current_segment.is_empty() {
                // Leaving clip region. We're inside-then-outside but no interior
                // crossing was recorded, which means the exit happened within
                // tolerance of `current` (the boundary intersection's `t` was so
                // close to an endpoint that it was filtered as a grazing touch).
                // Close the segment on `current` so the inside portion is kept
                // and ends at the boundary, instead of dropping the whole line
                // when only the start point remains. See issue #19.
                current_segment.push(*current);
                if current_segment.len() >= 2 {
                    result_lines.push(LineString::new(current_segment.clone()));
                }
                current_segment.clear();
            }
        } else {
            // Has intersections - process each segment
            let mut inside = prev_inside;

            for (_t, intersection_coord) in &intersections {
                if inside {
                    // We're inside, add point up to intersection (exiting)
                    if current_segment.is_empty() {
                        // If segment is empty, we should have started from prev point
                        current_segment.push(*prev);
                    }
                    current_segment.push(*intersection_coord);
                    if current_segment.len() >= 2 {
                        result_lines.push(LineString::new(current_segment.clone()));
                    }
                    current_segment.clear();
                } else {
                    // We're outside, start new segment at intersection (entering)
                    current_segment.push(*intersection_coord);
                }
                inside = !inside;
            }

            // Handle final segment after all intersections
            if inside {
                // We ended up inside after all intersections
                if current_inside {
                    // Current point is also inside, continue the segment
                    current_segment.push(*current);
                } else {
                    // Current point is outside but we're inside - shouldn't happen
                    // with proper intersection detection, but handle gracefully
                    if current_segment.len() >= 2 {
                        result_lines.push(LineString::new(current_segment.clone()));
                    }
                    current_segment.clear();
                }
            } else {
                // We ended up outside after all intersections
                if !current_segment.is_empty() {
                    // We have a dangling segment start - shouldn't have content
                    // unless something went wrong
                    if current_segment.len() >= 2 {
                        result_lines.push(LineString::new(current_segment.clone()));
                    }
                    current_segment.clear();
                }
            }
        }
    }

    // Don't forget the last segment
    if current_segment.len() >= 2 {
        result_lines.push(LineString::new(current_segment));
    }

    MultiLineString::new(result_lines)
}

/// Convert a MultiLineString to strokes
fn linestrings_to_strokes(mls: &MultiLineString<f64>, style: ResolvedStyle) -> Vec<Stroke> {
    mls.0
        .iter()
        .map(|ls| Stroke {
            points: linestring_to_points(ls),
            style,
            closed: false,
        })
        .filter(|s| s.points.len() >= 2)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FontRegistry, Group};
    use std::sync::Arc;

    fn test_ctx() -> RenderContext {
        RenderContext::new(Arc::new(FontRegistry::new()))
    }

    #[test]
    fn test_clip_lines_inside_circle() {
        let ctx = test_ctx();

        // Create a horizontal line fully inside a circle
        let line = Element::polyline(vec![Point::new(40.0, 50.0), Point::new(60.0, 50.0)]);

        let clipped = Element::clip(Element::circle((50.0, 50.0), 20.0)).add(line);

        let strokes = clipped.flatten(&ctx);

        // Line is fully inside, should have 1 stroke
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].points.len(), 2);
    }

    #[test]
    fn test_clip_lines_crossing_boundary() {
        let ctx = test_ctx();

        // Create a horizontal line crossing through a circle
        let line = Element::polyline(vec![Point::new(0.0, 50.0), Point::new(100.0, 50.0)]);

        let clipped = Element::clip(Element::circle((50.0, 50.0), 20.0)).add(line);

        let strokes = clipped.flatten(&ctx);

        // Line crosses boundary, should have clipped segment(s)
        assert!(!strokes.is_empty());
        // The clipped portion should be shorter than original
        let total_points: usize = strokes.iter().map(|s| s.points.len()).sum();
        assert!(total_points < 100); // Original line would have ~2 points, clipped has fewer spans
    }

    #[test]
    fn test_clip_diagonal_through_center() {
        let ctx = test_ctx();

        // Create a diagonal line from (0,0) to (200,200) through a circle at (100,100) r=70
        // This is the exact case from the clipped.svg that was failing
        let line = Element::polyline(vec![Point::new(0.0, 0.0), Point::new(200.0, 200.0)]);

        let clipped = Element::clip(Element::circle((100.0, 100.0), 70.0)).add(line);

        let strokes = clipped.flatten(&ctx);

        // Line should be clipped - the diagonal passes through the circle
        assert!(
            !strokes.is_empty(),
            "Diagonal line through circle center should produce clipped strokes"
        );

        // Verify points are inside the circle (with tolerance for bezier approximation)
        for stroke in &strokes {
            for p in &stroke.points {
                let dist = ((p.x - 100.0).powi(2) + (p.y - 100.0).powi(2)).sqrt();
                assert!(
                    dist <= 71.0, // Allow small tolerance
                    "Point ({}, {}) outside clip circle (dist={})",
                    p.x,
                    p.y,
                    dist
                );
            }
        }
    }

    #[test]
    fn test_clip_lines_outside() {
        let ctx = test_ctx();

        // Create a line completely outside the clip region
        let line = Element::polyline(vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)]);

        let clipped = Element::clip(Element::circle((50.0, 50.0), 20.0)).add(line);

        let strokes = clipped.flatten(&ctx);

        // Line is fully outside, should have no strokes
        assert!(strokes.is_empty());
    }

    #[test]
    fn test_clip_with_rect() {
        let ctx = test_ctx();

        // Create diagonal lines
        let line = Element::polyline(vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)]);

        let clipped = Element::clip(Element::rect(25.0, 25.0, 50.0, 50.0)).add(line);

        let strokes = clipped.flatten(&ctx);

        // Line should be clipped to rect bounds
        assert!(!strokes.is_empty());
    }

    #[test]
    fn test_clip_multiple_shapes_union() {
        let ctx = test_ctx();

        // Two circles as clip region
        let clip_shape = Element::group(
            Group::new()
                .add(Element::circle((30.0, 50.0), 15.0))
                .add(Element::circle((70.0, 50.0), 15.0)),
        );

        // Horizontal line through both circles
        let line = Element::polyline(vec![Point::new(0.0, 50.0), Point::new(100.0, 50.0)]);

        let clipped = Element::clip(clip_shape).add(line);

        let strokes = clipped.flatten(&ctx);

        // Should have segments in both circles
        assert!(strokes.len() >= 2);
    }

    #[test]
    fn test_clip_closed_shape() {
        let ctx = test_ctx();

        // A square that partially overlaps with clip circle
        let square = Element::rect(40.0, 40.0, 30.0, 30.0);

        let clipped = Element::clip(Element::circle((50.0, 50.0), 20.0)).add(square);

        let strokes = clipped.flatten(&ctx);

        // Should have clipped strokes (the square outline clipped to circle)
        assert!(!strokes.is_empty());
        // For pen plotting, we clip as lines, so result may be segments
        // All points should be within or near the clip circle
        for stroke in &strokes {
            for p in &stroke.points {
                let dist = ((p.x - 50.0).powi(2) + (p.y - 50.0).powi(2)).sqrt();
                assert!(
                    dist <= 21.0, // Allow small tolerance
                    "Point ({}, {}) outside clip circle",
                    p.x,
                    p.y
                );
            }
        }
    }

    #[test]
    fn test_clip_empty_clip_region() {
        let ctx = test_ctx();

        // Use an open line as clip (invalid - not closed)
        let invalid_clip = Element::polyline(vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)]);

        let line = Element::polyline(vec![Point::new(0.0, 50.0), Point::new(100.0, 50.0)]);

        let clipped = Element::clip(invalid_clip).add(line);

        let strokes = clipped.flatten(&ctx);

        // No valid clip region = nothing visible
        assert!(strokes.is_empty());
    }

    #[test]
    fn test_clip_with_transform() {
        let ctx = test_ctx();

        // Create clipped content with rotation
        let line = Element::polyline(vec![Point::new(-20.0, 0.0), Point::new(20.0, 0.0)]);

        let clipped = Element::clip(Element::circle((0.0, 0.0), 15.0))
            .add(line)
            .translate(50.0, 50.0);

        let strokes = clipped.flatten(&ctx);

        // Should have clipped line, translated to (50, 50)
        assert!(!strokes.is_empty());
        // Check that points are around (50, 50)
        for stroke in &strokes {
            for p in &stroke.points {
                assert!(p.x > 30.0 && p.x < 70.0);
                assert!(p.y > 30.0 && p.y < 70.0);
            }
        }
    }

    #[test]
    fn test_nested_clips_intersection() {
        let ctx = test_ctx();

        // Outer clip: large circle
        // Inner clip: overlapping circle
        // Content: horizontal line through both
        let line = Element::polyline(vec![Point::new(0.0, 50.0), Point::new(100.0, 50.0)]);

        // Inner clip intersects with outer clip
        let inner_clipped = Element::clip(Element::circle((60.0, 50.0), 25.0)).add(line);

        // Outer clip
        let outer_clipped = Element::clip(Element::circle((40.0, 50.0), 25.0)).add(inner_clipped);

        let strokes = outer_clipped.flatten(&ctx);

        // Line should only appear in the intersection of both circles
        assert!(!strokes.is_empty());

        // All points should be within BOTH clip regions (intersection area is roughly 40-60 on x-axis)
        for stroke in &strokes {
            for p in &stroke.points {
                // Points should be in the overlapping region
                assert!(
                    p.x >= 35.0 && p.x <= 65.0,
                    "Point x={} outside intersection",
                    p.x
                );
            }
        }
    }

    #[test]
    fn test_clip_rect_diagonal_lines_bottom_to_top() {
        let ctx = test_ctx();

        // Create a rect from (25,25) to (75,75)
        let clip_rect = Element::rect(25.0, 25.0, 50.0, 50.0);

        // Create two diagonal lines that start at the bottom of the rect and end at the top
        // Line 1: from (30, 75) to (30, 25) - vertical, should be fully inside
        let line1 = Element::polyline(vec![Point::new(30.0, 75.0), Point::new(30.0, 25.0)]);

        // Line 2: from (20, 80) to (80, 20) - diagonal crossing through rect
        let line2 = Element::polyline(vec![Point::new(20.0, 80.0), Point::new(80.0, 20.0)]);

        // Line 3: from (40, 100) to (60, 0) - diagonal from bottom to top, crossing rect
        let line3 = Element::polyline(vec![Point::new(40.0, 100.0), Point::new(60.0, 0.0)]);

        let lines = Element::group(
            Group::new()
                .add(line1.clone())
                .add(line2.clone())
                .add(line3.clone()),
        );

        let clipped = Element::clip(clip_rect).add(lines);
        let strokes = clipped.flatten(&ctx);

        // All three lines should produce clipped output
        assert!(
            !strokes.is_empty(),
            "Diagonal lines through rect should produce clipped strokes"
        );

        // All clipped points should be within the rect bounds (with small tolerance)
        for stroke in &strokes {
            for p in &stroke.points {
                assert!(
                    p.x >= 24.9 && p.x <= 75.1,
                    "Point x={} outside rect x bounds",
                    p.x
                );
                assert!(
                    p.y >= 24.9 && p.y <= 75.1,
                    "Point y={} outside rect y bounds",
                    p.y
                );
            }
        }
    }

    #[test]
    fn test_clip_inverted_keeps_outside() {
        let ctx = test_ctx();

        // Create a horizontal line through a circle
        let line = Element::polyline(vec![Point::new(0.0, 50.0), Point::new(100.0, 50.0)]);

        // Normal clip: keeps inside
        let normal_clipped = Element::clip(Element::circle((50.0, 50.0), 20.0)).add(line.clone());
        let normal_strokes = normal_clipped.flatten(&ctx);

        // Inverted clip: keeps outside
        let inverted_clipped = Element::clip(Element::circle((50.0, 50.0), 20.0))
            .invert(true)
            .add(line);
        let inverted_strokes = inverted_clipped.flatten(&ctx);

        // Normal should have strokes inside the circle
        assert!(
            !normal_strokes.is_empty(),
            "Normal clip should produce strokes"
        );
        for stroke in &normal_strokes {
            for p in &stroke.points {
                let dist = ((p.x - 50.0).powi(2) + (p.y - 50.0).powi(2)).sqrt();
                assert!(
                    dist <= 21.0,
                    "Normal clip: point ({}, {}) should be inside circle (dist={})",
                    p.x,
                    p.y,
                    dist
                );
            }
        }

        // Inverted should have strokes outside the circle
        assert!(
            !inverted_strokes.is_empty(),
            "Inverted clip should produce strokes"
        );
        // Should have 2 segments (one on each side of the circle)
        assert_eq!(
            inverted_strokes.len(),
            2,
            "Inverted clip should produce 2 segments (left and right of circle)"
        );

        for stroke in &inverted_strokes {
            for p in &stroke.points {
                let dist = ((p.x - 50.0).powi(2) + (p.y - 50.0).powi(2)).sqrt();
                assert!(
                    dist >= 19.0,
                    "Inverted clip: point ({}, {}) should be outside circle (dist={})",
                    p.x,
                    p.y,
                    dist
                );
            }
        }
    }

    #[test]
    fn test_clip_inverted_line_fully_inside() {
        let ctx = test_ctx();

        // Create a line fully inside a circle
        let line = Element::polyline(vec![Point::new(40.0, 50.0), Point::new(60.0, 50.0)]);

        // Normal clip: should keep the line
        let normal_clipped = Element::clip(Element::circle((50.0, 50.0), 20.0)).add(line.clone());
        let normal_strokes = normal_clipped.flatten(&ctx);
        assert_eq!(
            normal_strokes.len(),
            1,
            "Normal clip should keep line inside"
        );

        // Inverted clip: should remove the line (it's fully inside)
        let inverted_clipped = Element::clip(Element::circle((50.0, 50.0), 20.0))
            .invert(true)
            .add(line);
        let inverted_strokes = inverted_clipped.flatten(&ctx);
        assert!(
            inverted_strokes.is_empty(),
            "Inverted clip should remove line that's fully inside"
        );
    }

    #[test]
    fn test_clip_inverted_line_fully_outside() {
        let ctx = test_ctx();

        // Create a line fully outside a circle
        let line = Element::polyline(vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)]);

        // Normal clip: should remove the line
        let normal_clipped = Element::clip(Element::circle((50.0, 50.0), 20.0)).add(line.clone());
        let normal_strokes = normal_clipped.flatten(&ctx);
        assert!(
            normal_strokes.is_empty(),
            "Normal clip should remove line outside"
        );

        // Inverted clip: should keep the line
        let inverted_clipped = Element::clip(Element::circle((50.0, 50.0), 20.0))
            .invert(true)
            .add(line);
        let inverted_strokes = inverted_clipped.flatten(&ctx);
        assert_eq!(
            inverted_strokes.len(),
            1,
            "Inverted clip should keep line outside"
        );
    }

    #[test]
    fn test_clip_inverted_rotated_rect() {
        let ctx = test_ctx();

        // Simulate the rotated squares sketch case:
        // Two rectangles centered at the same point, with different rotations
        // First rect clipped by second (inverted)
        let cx = 100.0;
        let cy = 100.0;
        let half = 50.0;
        let size = 100.0;

        // First rotation (about 5 degrees)
        let rot1 = 5.0_f64.to_radians();
        // Second rotation (about -3 degrees)
        let rot2 = (-3.0_f64).to_radians();

        // Create the first square (the one being clipped)
        let square1 = Element::rect(-half, -half, size, size)
            .rotate(rot1)
            .translate(cx, cy);

        // Create clip shape from the second square
        let clip_shape = Element::rect(-half, -half, size, size)
            .rotate(rot2)
            .translate(cx, cy);

        // Apply inverted clip
        let clipped = Element::clip(clip_shape).invert(true).add(square1);
        let strokes = clipped.flatten(&ctx);

        // The clipped result should have segments for all 4 edges (plus the closing edge)
        assert!(
            !strokes.is_empty(),
            "Inverted clip of rotated rectangles should produce strokes"
        );

        // We expect visible portions from all 4 edges of the first rectangle
        // With the closing edge fix, we should have 5 segments (4 edges + closing)
        // or at least 4 if some edges are fully inside the clip region
        assert!(
            strokes.len() >= 4,
            "Expected at least 4 segments (one per edge), got {}",
            strokes.len()
        );
    }

    #[test]
    fn test_clip_full_height_lines_on_boundary_survive() {
        // Regression test for issue #19:
        // Vertical lines whose endpoints lie exactly on the clip rect's top/bottom
        // edges must be preserved in full, not silently dropped.
        let ctx = test_ctx();

        // Clip region: the bounding square (0,0)-(100,100).
        let clip_rect = Element::rect(0.0, 0.0, 100.0, 100.0);

        // Four full-height vertical lines spanning the entire clip height.
        // Their endpoints sit exactly on the top (y=0) and bottom (y=100) edges.
        let xs = [20.0, 40.0, 60.0, 80.0];
        let mut group = Group::new();
        for x in xs {
            group = group.add(Element::polyline(vec![
                Point::new(x, 0.0),
                Point::new(x, 100.0),
            ]));
        }

        let clipped = Element::clip(clip_rect).add(Element::group(group));
        let strokes = clipped.flatten(&ctx);

        // Every vertical line should survive intact.
        assert_eq!(
            strokes.len(),
            xs.len(),
            "All {} full-height lines should survive clipping, got {}",
            xs.len(),
            strokes.len()
        );

        // Each surviving stroke must still span the full height of the clip rect.
        for stroke in &strokes {
            let min_y = stroke
                .points
                .iter()
                .map(|p| p.y)
                .fold(f64::INFINITY, f64::min);
            let max_y = stroke
                .points
                .iter()
                .map(|p| p.y)
                .fold(f64::NEG_INFINITY, f64::max);
            assert!(
                min_y <= 0.0 + 1e-6 && max_y >= 100.0 - 1e-6,
                "Clipped line should span full height (got {}..{})",
                min_y,
                max_y
            );
        }
    }

    #[test]
    fn test_clip_rect_outline_on_boundary_survives() {
        // Regression test for issue #19:
        // A closed rectangle outline drawn exactly on the clip boundary must be preserved.
        let ctx = test_ctx();

        let clip_rect = Element::rect(0.0, 0.0, 100.0, 100.0);
        // Same rectangle as a child - its outline lies exactly on the clip boundary.
        let outline = Element::rect(0.0, 0.0, 100.0, 100.0);

        let clipped = Element::clip(clip_rect).add(outline);
        let strokes = clipped.flatten(&ctx);

        assert!(
            !strokes.is_empty(),
            "Rectangle outline on the clip boundary should be preserved, but it vanished"
        );

        // The full perimeter should be present (bounding box matches the rect).
        let min_x = strokes
            .iter()
            .flat_map(|s| s.points.iter())
            .map(|p| p.x)
            .fold(f64::INFINITY, f64::min);
        let max_x = strokes
            .iter()
            .flat_map(|s| s.points.iter())
            .map(|p| p.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = strokes
            .iter()
            .flat_map(|s| s.points.iter())
            .map(|p| p.y)
            .fold(f64::INFINITY, f64::min);
        let max_y = strokes
            .iter()
            .flat_map(|s| s.points.iter())
            .map(|p| p.y)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            min_x <= 1e-6 && min_y <= 1e-6 && max_x >= 100.0 - 1e-6 && max_y >= 100.0 - 1e-6,
            "Outline should cover the full rect perimeter (got x {}..{}, y {}..{})",
            min_x,
            max_x,
            min_y,
            max_y
        );
    }

    #[test]
    fn test_clip_line_endpoint_just_outside_is_clipped_not_dropped() {
        // Regression test for issue #19:
        // A line whose start is inside and whose end is a hair *outside* the clip
        // boundary (here ~3e-14 below it, the kind of error produced by
        // `y0 + n * (size / n)`) must be clipped to the boundary, not dropped.
        // The exit crossing sits within the endpoint-tolerance of the segment,
        // so the naive path discarded it and deleted the whole line.
        let ctx = test_ctx();

        let clip_rect = Element::rect(0.0, 0.0, 100.0, 100.0);

        // End point a tiny bit below the bottom edge (y = 100).
        let just_outside = 100.0 + 3e-14;
        let line = Element::polyline(vec![Point::new(50.0, 75.0), Point::new(50.0, just_outside)]);

        let clipped = Element::clip(clip_rect).add(line);
        let strokes = clipped.flatten(&ctx);

        assert_eq!(
            strokes.len(),
            1,
            "Line straddling the boundary by a rounding error should survive, got {}",
            strokes.len()
        );
        // The surviving stroke should still span from the interior start down to
        // (essentially) the boundary.
        let stroke = &strokes[0];
        let ymin = stroke
            .points
            .iter()
            .map(|p| p.y)
            .fold(f64::INFINITY, f64::min);
        let ymax = stroke
            .points
            .iter()
            .map(|p| p.y)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            ymin <= 75.0 + 1e-6 && ymax >= 100.0 - 1e-6,
            "Clipped line should span interior..boundary (got {ymin}..{ymax})"
        );
    }

    #[test]
    fn test_clip_closed_stroke_includes_closing_edge() {
        let ctx = test_ctx();

        // Create a rectangle centered at origin, then rotate slightly
        // Rect corners are at (-50,-50), (50,-50), (50,50), (-50,50)
        // The closing edge goes from (-50,50) back to (-50,-50) - this is the left edge
        let half = 50.0;
        let size = 100.0;
        let rot = 5.0_f64.to_radians();

        let rect = Element::rect(-half, -half, size, size)
            .rotate(rot)
            .translate(100.0, 100.0);

        // Use inverted clip with a non-rotated rectangle
        // The rotated corners will stick out
        let clip_shape = Element::rect(50.0, 50.0, 100.0, 100.0);

        let clipped = Element::clip(clip_shape).invert(true).add(rect);
        let strokes = clipped.flatten(&ctx);

        // The clipped rectangle should have visible portions (the rotated corners sticking out)
        assert!(
            !strokes.is_empty(),
            "Inverted clipped rotated rectangle should have visible corner portions"
        );

        // With the closing edge fix, we should have at least 4 visible segments
        // (parts of all edges including the closing edge)
        assert!(
            strokes.len() >= 4,
            "Should have at least 4 segments for rotated rect corners, got {}",
            strokes.len()
        );
    }
}
