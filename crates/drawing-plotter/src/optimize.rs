//! Stroke path optimization for efficient plotting

use drawing_core::{Point, Stroke};

/// Optimize stroke order to minimize pen-up travel distance
///
/// Uses a greedy nearest-neighbor algorithm to reorder strokes
/// so that each stroke starts near where the previous one ended.
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

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_core::Style;

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
            Style::default(),
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
                Style::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(50.0, 50.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(50.0, 50.0),
                Point::new(100.0, 100.0),
                Style::default(),
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
                Style::default(),
            ),
            Stroke::line(
                Point::new(10.0, 10.0),
                Point::new(20.0, 20.0),
                Style::default(),
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
                Style::default(),
            ),
            Stroke::line(
                Point::new(50.0, 0.0),
                Point::new(60.0, 0.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Style::default(),
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
                Style::default(),
            ),
            Stroke::line(
                Point::new(20.0, 20.0),
                Point::new(30.0, 30.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(40.0, 40.0),
                Point::new(50.0, 50.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(60.0, 60.0),
                Point::new(70.0, 70.0),
                Style::default(),
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
        let stroke = Stroke::line(Point::new(0.0, 0.0), Point::new(3.0, 4.0), Style::default());
        let strokes = vec![&stroke];
        // Pen-up from origin (0) + pen-down distance (5) = 5
        assert!(approx_eq(total_travel_distance(&strokes), 5.0));
    }

    #[test]
    fn test_total_travel_distance_includes_pen_up() {
        let stroke = Stroke::line(
            Point::new(10.0, 0.0), // 10 units from origin
            Point::new(13.0, 4.0), // 5 unit stroke (3-4-5)
            Style::default(),
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
            Style::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(10.0, 0.0), // No pen-up travel from stroke1 end
            Point::new(20.0, 0.0),
            Style::default(),
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
            Style::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(20.0, 0.0), // 10 units gap
            Point::new(30.0, 0.0),
            Style::default(),
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
            Style::default(),
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
            Style::default(),
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
            Style::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(100.0, 100.0), // Position doesn't matter
            Point::new(100.0, 120.0), // 20 units
            Style::default(),
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
            Style::default(),
        );
        let strokes = vec![&stroke];
        assert!(approx_eq(pen_down_distance(&strokes), 30.0));
    }

    #[test]
    fn test_pen_down_distance_ignores_pen_up_travel() {
        // Two strokes far apart
        let stroke1 = Stroke::line(Point::new(0.0, 0.0), Point::new(5.0, 0.0), Style::default());
        let stroke2 = Stroke::line(
            Point::new(1000.0, 1000.0), // Very far away
            Point::new(1005.0, 1000.0), // 5 units
            Style::default(),
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
                Style::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(50.0, 0.0),
                Point::new(60.0, 0.0),
                Style::default(),
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
                Style::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Style::default(),
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
}
