//! Stroke path optimization for efficient plotting

use drawing_core::{Point, ResolvedStyle, Stroke};

/// A reference to a stroke that may be drawn in reverse direction (internal use only)
#[derive(Debug, Clone, Copy)]
struct OptimizedStroke<'a> {
    /// The original stroke
    stroke: &'a Stroke,
    /// Whether to draw this stroke in reverse
    reversed: bool,
}

impl<'a> OptimizedStroke<'a> {
    /// Create a new optimized stroke reference
    fn new(stroke: &'a Stroke, reversed: bool) -> Self {
        Self { stroke, reversed }
    }

    /// Get the effective start point (considering reversal)
    #[cfg(test)]
    fn start(&self) -> Point {
        if self.reversed {
            self.stroke.points.last().copied().unwrap_or(Point::ZERO)
        } else {
            self.stroke.points.first().copied().unwrap_or(Point::ZERO)
        }
    }

    /// Get the effective end point (considering reversal)
    fn end(&self) -> Point {
        if self.reversed {
            self.stroke.points.first().copied().unwrap_or(Point::ZERO)
        } else {
            self.stroke.points.last().copied().unwrap_or(Point::ZERO)
        }
    }

    /// Iterate over points in the correct order (considering reversal)
    #[cfg(test)]
    fn points(&self) -> impl Iterator<Item = Point> + '_ {
        let len = self.stroke.points.len();
        let reversed = self.reversed;
        let points = &self.stroke.points;
        (0..len).map(move |i| {
            let idx = if reversed { len - 1 - i } else { i };
            points[idx]
        })
    }

    /// Convert to an owned version that can be sent across threads
    fn into_owned(self) -> OwnedOptimizedStroke {
        OwnedOptimizedStroke {
            points: self.stroke.points.clone(),
            style: self.stroke.style,
            closed: self.stroke.closed,
            reversed: self.reversed,
        }
    }
}

/// An owned optimized stroke that can be sent across threads.
///
/// Unlike `OptimizedStroke<'a>` which borrows the stroke data, this struct
/// owns all the data and can be safely moved between threads.
#[derive(Debug, Clone)]
pub struct OwnedOptimizedStroke {
    /// The stroke points
    pub points: Vec<Point>,
    /// The stroke style
    pub style: ResolvedStyle,
    /// Whether the stroke is closed
    pub closed: bool,
    /// Whether to draw this stroke in reverse
    pub reversed: bool,
}

impl OwnedOptimizedStroke {
    /// Get the effective start point (considering reversal)
    pub fn start(&self) -> Point {
        if self.reversed {
            self.points.last().copied().unwrap_or(Point::ZERO)
        } else {
            self.points.first().copied().unwrap_or(Point::ZERO)
        }
    }

    /// Get the effective end point (considering reversal)
    pub fn end(&self) -> Point {
        if self.reversed {
            self.points.first().copied().unwrap_or(Point::ZERO)
        } else {
            self.points.last().copied().unwrap_or(Point::ZERO)
        }
    }

    /// Iterate over points in the correct order
    pub fn points_iter(&self) -> impl Iterator<Item = Point> + '_ {
        let len = self.points.len();
        let reversed = self.reversed;
        (0..len).map(move |i| {
            let idx = if reversed { len - 1 - i } else { i };
            self.points[idx]
        })
    }

    /// Check if this stroke is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Optimize stroke order to minimize pen-up travel distance
///
/// Uses a greedy nearest-neighbor algorithm to reorder strokes
/// so that each stroke starts near where the previous one ended.
///
/// This version does NOT consider stroke reversal. Use `optimize_strokes_with_reversal`
/// for better optimization that can draw strokes in reverse when beneficial.
pub fn optimize_strokes(strokes: &[Stroke]) -> Vec<&Stroke> {
    if strokes.is_empty() {
        return vec![];
    }

    // Simple greedy nearest-neighbor algorithm
    let mut remaining: Vec<_> = strokes.iter().collect();
    let mut ordered = Vec::with_capacity(strokes.len());
    let mut current_pos = Point::ZERO;

    while !remaining.is_empty() {
        // Find nearest stroke start
        let (idx, _) = remaining
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = current_pos.distance(a.points[0]);
                let dist_b = current_pos.distance(b.points[0]);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap();

        let stroke = remaining.remove(idx);
        if let Some(last) = stroke.points.last() {
            current_pos = *last;
        }
        ordered.push(stroke);
    }

    ordered
}

/// Optimize stroke order with reversal support (returns borrowed references)
///
/// This is an internal function that returns borrowed references. For most use cases,
/// prefer `optimize_strokes_owned` which returns owned data that can be sent across threads.
fn optimize_strokes_internal(strokes: &[Stroke], allow_reversal: bool) -> Vec<OptimizedStroke<'_>> {
    if strokes.is_empty() {
        return vec![];
    }

    let total_strokes = strokes.len();
    log::debug!(
        "Optimizing {} strokes (reversal={})",
        total_strokes,
        allow_reversal
    );

    // Track remaining strokes with their indices
    let mut remaining: Vec<(usize, &Stroke)> = strokes.iter().enumerate().collect();
    let mut ordered = Vec::with_capacity(strokes.len());
    let mut current_pos = Point::ZERO;
    let mut reversed_count = 0;

    // Log progress every 10% for large stroke sets
    let log_interval = (total_strokes / 10).max(1000);
    let mut last_log = 0;

    while !remaining.is_empty() {
        let mut best_idx = 0;
        let mut best_dist = f64::MAX;
        let mut best_reversed = false;

        for (i, (_, stroke)) in remaining.iter().enumerate() {
            if stroke.points.is_empty() {
                continue;
            }

            // Check distance to start point
            let start = stroke.points[0];
            let dist_to_start = current_pos.distance(start);
            if dist_to_start < best_dist {
                best_dist = dist_to_start;
                best_idx = i;
                best_reversed = false;
            }

            // Check distance to end point (if reversal allowed)
            if allow_reversal {
                if let Some(&end) = stroke.points.last() {
                    let dist_to_end = current_pos.distance(end);
                    if dist_to_end < best_dist {
                        best_dist = dist_to_end;
                        best_idx = i;
                        best_reversed = true;
                    }
                }
            }
        }

        let (_, stroke) = remaining.remove(best_idx);
        let optimized = OptimizedStroke::new(stroke, best_reversed);
        current_pos = optimized.end();
        if best_reversed {
            reversed_count += 1;
        }
        ordered.push(optimized);

        // Log progress for large stroke sets
        let processed = ordered.len();
        if processed - last_log >= log_interval {
            log::debug!(
                "Optimization progress: {}/{} strokes ({:.0}%)",
                processed,
                total_strokes,
                (processed as f64 / total_strokes as f64) * 100.0
            );
            last_log = processed;
        }
    }

    log::debug!(
        "Optimization complete: {} strokes, {} reversed ({:.1}%)",
        ordered.len(),
        reversed_count,
        (reversed_count as f64 / ordered.len() as f64) * 100.0
    );

    ordered
}

/// Optimize stroke order with reversal support
///
/// Uses a greedy nearest-neighbor algorithm that considers both the start
/// and end points of each stroke. When the end point is closer to the current
/// position, the stroke will be marked for reverse drawing.
///
/// This typically reduces travel distance by 10-30% compared to start-point-only
/// optimization.
///
/// Returns owned data that can be safely sent across threads.
///
/// # Arguments
/// * `strokes` - The strokes to optimize
/// * `allow_reversal` - If true, strokes can be drawn in reverse. If false,
///   strokes are only reordered but not reversed.
pub fn optimize_strokes_with_reversal(
    strokes: &[Stroke],
    allow_reversal: bool,
) -> Vec<OwnedOptimizedStroke> {
    let optimized = optimize_strokes_internal(strokes, allow_reversal);
    optimized.into_iter().map(|o| o.into_owned()).collect()
}

/// Calculate total travel distance for a set of strokes
///
/// Includes both pen-up travel (moving between strokes) and
/// pen-down travel (drawing the strokes).
pub fn total_travel_distance(strokes: &[&Stroke]) -> f64 {
    let mut total = 0.0;
    let mut pos = Point::ZERO;

    for stroke in strokes {
        if stroke.points.is_empty() {
            continue;
        }

        // Pen-up travel to start
        total += pos.distance(stroke.points[0]);

        // Pen-down travel along stroke
        for pts in stroke.points.windows(2) {
            total += pts[0].distance(pts[1]);
        }

        if let Some(last) = stroke.points.last() {
            pos = *last;
        }
    }

    total
}

/// Calculate pen-down distance only
///
/// This is the total length of all strokes, ignoring travel between them.
pub fn pen_down_distance(strokes: &[&Stroke]) -> f64 {
    strokes
        .iter()
        .map(|s| {
            s.points
                .windows(2)
                .map(|w| w[0].distance(w[1]))
                .sum::<f64>()
        })
        .sum()
}

/// Calculate total travel distance for optimized strokes (with reversal support)
///
/// This correctly accounts for stroke direction when calculating pen-up travel.
pub fn total_travel_distance_optimized(strokes: &[OwnedOptimizedStroke]) -> f64 {
    let mut total = 0.0;
    let mut pos = Point::ZERO;

    for opt_stroke in strokes {
        if opt_stroke.points.is_empty() {
            continue;
        }

        // Pen-up travel to start (considering reversal)
        total += pos.distance(opt_stroke.start());

        // Pen-down travel along stroke (same distance regardless of direction)
        for pts in opt_stroke.points.windows(2) {
            total += pts[0].distance(pts[1]);
        }

        pos = opt_stroke.end();
    }

    total
}

/// Calculate pen-down distance for optimized strokes
///
/// Same as pen_down_distance since stroke length is direction-independent.
pub fn pen_down_distance_optimized(strokes: &[OwnedOptimizedStroke]) -> f64 {
    strokes
        .iter()
        .map(|s| {
            s.points
                .windows(2)
                .map(|w| w[0].distance(w[1]))
                .sum::<f64>()
        })
        .sum()
}

/// Calculate pen-up travel distance for optimized strokes
pub fn travel_distance_optimized(strokes: &[OwnedOptimizedStroke]) -> f64 {
    let total = total_travel_distance_optimized(strokes);
    let pen_down = pen_down_distance_optimized(strokes);
    total - pen_down
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_core::ResolvedStyle;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    // ========================================================================
    // optimize_strokes tests
    // ========================================================================

    #[test]
    fn test_optimize_strokes_empty() {
        let strokes: Vec<Stroke> = vec![];
        let optimized = optimize_strokes(&strokes);
        assert!(optimized.is_empty());
    }

    #[test]
    fn test_optimize_strokes_single() {
        let strokes = vec![Stroke::line(
            Point::new(50.0, 50.0),
            Point::new(100.0, 100.0),
            ResolvedStyle::default(),
        )];
        let optimized = optimize_strokes(&strokes);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].points[0], Point::new(50.0, 50.0));
    }

    #[test]
    fn test_optimize_strokes_nearest_neighbor() {
        let strokes = vec![
            Stroke::line(
                Point::new(100.0, 100.0),
                Point::new(150.0, 150.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(50.0, 50.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(50.0, 50.0),
                Point::new(100.0, 100.0),
                ResolvedStyle::default(),
            ),
        ];

        let optimized = optimize_strokes(&strokes);

        // Should start with stroke closest to origin (0,0)
        assert_eq!(optimized[0].points[0], Point::new(0.0, 0.0));
        // Second stroke should start where first ended (50, 50)
        assert_eq!(optimized[1].points[0], Point::new(50.0, 50.0));
        // Third stroke should start where second ended (100, 100)
        assert_eq!(optimized[2].points[0], Point::new(100.0, 100.0));
    }

    #[test]
    fn test_optimize_strokes_already_optimal() {
        let strokes = vec![
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 10.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(10.0, 10.0),
                Point::new(20.0, 20.0),
                ResolvedStyle::default(),
            ),
        ];

        let optimized = optimize_strokes(&strokes);

        // Order should remain the same
        assert_eq!(optimized[0].points[0], Point::new(0.0, 0.0));
        assert_eq!(optimized[1].points[0], Point::new(10.0, 10.0));
    }

    #[test]
    fn test_optimize_strokes_reverse_order() {
        // Strokes in reverse order should be reordered
        let strokes = vec![
            Stroke::line(
                Point::new(100.0, 0.0),
                Point::new(110.0, 0.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(50.0, 0.0),
                Point::new(60.0, 0.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                ResolvedStyle::default(),
            ),
        ];

        let optimized = optimize_strokes(&strokes);

        // Should be reordered to start from origin
        assert_eq!(optimized[0].points[0], Point::new(0.0, 0.0));
    }

    #[test]
    fn test_optimize_strokes_preserves_count() {
        let strokes = vec![
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 10.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(20.0, 20.0),
                Point::new(30.0, 30.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(40.0, 40.0),
                Point::new(50.0, 50.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(60.0, 60.0),
                Point::new(70.0, 70.0),
                ResolvedStyle::default(),
            ),
        ];

        let optimized = optimize_strokes(&strokes);
        assert_eq!(optimized.len(), strokes.len());
    }

    // ========================================================================
    // total_travel_distance tests
    // ========================================================================

    #[test]
    fn test_total_travel_distance_empty() {
        let strokes: Vec<&Stroke> = vec![];
        assert!(approx_eq(total_travel_distance(&strokes), 0.0));
    }

    #[test]
    fn test_total_travel_distance_single_stroke() {
        let stroke = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(3.0, 4.0),
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke];
        // Pen-up from origin (0) + pen-down distance (5) = 5
        assert!(approx_eq(total_travel_distance(&strokes), 5.0));
    }

    #[test]
    fn test_total_travel_distance_includes_pen_up() {
        let stroke = Stroke::line(
            Point::new(10.0, 0.0), // 10 units from origin
            Point::new(13.0, 4.0), // 5 unit stroke (3-4-5)
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke];
        // Pen-up travel (10) + pen-down travel (5) = 15
        assert!(approx_eq(total_travel_distance(&strokes), 15.0));
    }

    #[test]
    fn test_total_travel_distance_multiple_strokes() {
        let stroke1 = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            ResolvedStyle::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(10.0, 0.0), // No pen-up travel from stroke1 end
            Point::new(20.0, 0.0),
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke1, &stroke2];
        // Pen-up from origin (0) + stroke1 (10) + pen-up (0) + stroke2 (10) = 20
        assert!(approx_eq(total_travel_distance(&strokes), 20.0));
    }

    #[test]
    fn test_total_travel_distance_with_gap() {
        let stroke1 = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            ResolvedStyle::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(20.0, 0.0), // 10 units gap
            Point::new(30.0, 0.0),
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke1, &stroke2];
        // Pen-up from origin (0) + stroke1 (10) + pen-up (10) + stroke2 (10) = 30
        assert!(approx_eq(total_travel_distance(&strokes), 30.0));
    }

    #[test]
    fn test_total_travel_distance_multi_point_stroke() {
        let stroke = Stroke::new(
            vec![
                Point::new(0.0, 0.0),
                Point::new(3.0, 4.0), // 5 units
                Point::new(3.0, 0.0), // 4 units
            ],
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke];
        // Pen-up from origin (0) + 5 + 4 = 9
        assert!(approx_eq(total_travel_distance(&strokes), 9.0));
    }

    // ========================================================================
    // pen_down_distance tests
    // ========================================================================

    #[test]
    fn test_pen_down_distance_empty() {
        let strokes: Vec<&Stroke> = vec![];
        assert!(approx_eq(pen_down_distance(&strokes), 0.0));
    }

    #[test]
    fn test_pen_down_distance_single_stroke() {
        let stroke = Stroke::line(
            Point::new(100.0, 100.0), // Far from origin
            Point::new(103.0, 104.0), // 5 unit stroke
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke];
        // Only pen-down distance, ignores position
        assert!(approx_eq(pen_down_distance(&strokes), 5.0));
    }

    #[test]
    fn test_pen_down_distance_multiple_strokes() {
        let stroke1 = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            ResolvedStyle::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(100.0, 100.0), // Position doesn't matter
            Point::new(100.0, 120.0), // 20 units
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke1, &stroke2];
        assert!(approx_eq(pen_down_distance(&strokes), 30.0));
    }

    #[test]
    fn test_pen_down_distance_multi_point_stroke() {
        let stroke = Stroke::new(
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),  // 10 units
                Point::new(10.0, 10.0), // 10 units
                Point::new(0.0, 10.0),  // 10 units
            ],
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke];
        assert!(approx_eq(pen_down_distance(&strokes), 30.0));
    }

    #[test]
    fn test_pen_down_distance_ignores_pen_up_travel() {
        // Two strokes far apart
        let stroke1 = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0),
            ResolvedStyle::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(1000.0, 1000.0), // Very far away
            Point::new(1005.0, 1000.0), // 5 units
            ResolvedStyle::default(),
        );
        let strokes = vec![&stroke1, &stroke2];
        // Should only count pen-down: 5 + 5 = 10
        assert!(approx_eq(pen_down_distance(&strokes), 10.0));
    }

    // ========================================================================
    // Optimization verification tests
    // ========================================================================

    #[test]
    fn test_optimized_has_less_or_equal_travel() {
        let strokes = vec![
            Stroke::line(
                Point::new(100.0, 0.0),
                Point::new(110.0, 0.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(50.0, 0.0),
                Point::new(60.0, 0.0),
                ResolvedStyle::default(),
            ),
        ];

        let unoptimized: Vec<_> = strokes.iter().collect();
        let optimized = optimize_strokes(&strokes);

        let unoptimized_distance = total_travel_distance(&unoptimized);
        let optimized_distance = total_travel_distance(&optimized);

        // Optimized should have less or equal travel distance
        assert!(optimized_distance <= unoptimized_distance);
    }

    #[test]
    fn test_optimization_preserves_pen_down_distance() {
        let strokes = vec![
            Stroke::line(
                Point::new(100.0, 0.0),
                Point::new(110.0, 0.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                ResolvedStyle::default(),
            ),
        ];

        let unoptimized: Vec<_> = strokes.iter().collect();
        let optimized = optimize_strokes(&strokes);

        // Pen-down distance should be the same regardless of order
        assert!(approx_eq(
            pen_down_distance(&unoptimized),
            pen_down_distance(&optimized)
        ));
    }

    // ========================================================================
    // Stroke reversal tests
    // ========================================================================

    #[test]
    fn test_optimized_stroke_start_end() {
        let stroke = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            ResolvedStyle::default(),
        );

        // Normal direction
        let normal = OptimizedStroke::new(&stroke, false);
        assert_eq!(normal.start(), Point::new(0.0, 0.0));
        assert_eq!(normal.end(), Point::new(100.0, 0.0));

        // Reversed direction
        let reversed = OptimizedStroke::new(&stroke, true);
        assert_eq!(reversed.start(), Point::new(100.0, 0.0));
        assert_eq!(reversed.end(), Point::new(0.0, 0.0));
    }

    #[test]
    fn test_optimized_stroke_points_iteration() {
        let stroke = Stroke::new(
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(20.0, 0.0),
            ],
            ResolvedStyle::default(),
        );

        // Normal direction
        let normal = OptimizedStroke::new(&stroke, false);
        let pts: Vec<_> = normal.points().collect();
        assert_eq!(pts[0], Point::new(0.0, 0.0));
        assert_eq!(pts[1], Point::new(10.0, 0.0));
        assert_eq!(pts[2], Point::new(20.0, 0.0));

        // Reversed direction
        let reversed = OptimizedStroke::new(&stroke, true);
        let pts: Vec<_> = reversed.points().collect();
        assert_eq!(pts[0], Point::new(20.0, 0.0));
        assert_eq!(pts[1], Point::new(10.0, 0.0));
        assert_eq!(pts[2], Point::new(0.0, 0.0));
    }

    #[test]
    fn test_optimize_with_reversal_chooses_closer_end() {
        // Stroke that ends closer to origin than it starts
        let strokes = vec![Stroke::line(
            Point::new(100.0, 0.0), // Start far from origin
            Point::new(10.0, 0.0),  // End close to origin
            ResolvedStyle::default(),
        )];

        let optimized = optimize_strokes_with_reversal(&strokes, true);

        // Should be reversed since end (10,0) is closer to origin than start (100,0)
        assert!(optimized[0].reversed);
        assert_eq!(optimized[0].start(), Point::new(10.0, 0.0));
    }

    #[test]
    fn test_optimize_with_reversal_disabled() {
        // Same stroke as above
        let strokes = vec![Stroke::line(
            Point::new(100.0, 0.0),
            Point::new(10.0, 0.0),
            ResolvedStyle::default(),
        )];

        let optimized = optimize_strokes_with_reversal(&strokes, false);

        // Should NOT be reversed when reversal is disabled
        assert!(!optimized[0].reversed);
        assert_eq!(optimized[0].start(), Point::new(100.0, 0.0));
    }

    #[test]
    fn test_optimize_with_reversal_reduces_travel() {
        // Two strokes where reversal helps
        // First stroke ends at (100, 0), second stroke ends at (100, 0) but starts at (200, 0)
        // Without reversal: travel from (100,0) to (200,0) = 100
        // With reversal: travel from (100,0) to (100,0) = 0
        let strokes = vec![
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(100.0, 0.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(200.0, 0.0), // Far start
                Point::new(100.0, 0.0), // Close end (to previous stroke's end)
                ResolvedStyle::default(),
            ),
        ];

        let without_reversal = optimize_strokes_with_reversal(&strokes, false);
        let with_reversal = optimize_strokes_with_reversal(&strokes, true);

        let travel_without = travel_distance_optimized(&without_reversal);
        let travel_with = travel_distance_optimized(&with_reversal);

        // With reversal should have less travel
        assert!(travel_with < travel_without);
        // Second stroke should be reversed
        assert!(with_reversal[1].reversed);
    }

    #[test]
    fn test_optimize_with_reversal_preserves_pen_down_distance() {
        let strokes = vec![
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(100.0, 0.0),
                ResolvedStyle::default(),
            ),
            Stroke::line(
                Point::new(200.0, 0.0),
                Point::new(100.0, 0.0),
                ResolvedStyle::default(),
            ),
        ];

        let optimized = optimize_strokes_with_reversal(&strokes, true);

        // Pen-down distance should be the same (reversal doesn't change stroke length)
        let original_distance: f64 = strokes
            .iter()
            .map(|s| {
                s.points
                    .windows(2)
                    .map(|w| w[0].distance(w[1]))
                    .sum::<f64>()
            })
            .sum();

        assert!(approx_eq(
            pen_down_distance_optimized(&optimized),
            original_distance
        ));
    }
}
