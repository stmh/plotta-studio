//! Path simplification - remove redundant collinear points

use kurbo::Point;

/// Remove collinear points from a path within the given tolerance.
///
/// Points are considered collinear if the perpendicular distance from
/// the middle point to the line formed by its neighbors is less than `tolerance`.
pub fn simplify_points(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut result = Vec::with_capacity(points.len());
    result.push(points[0]);

    for i in 1..points.len() - 1 {
        let prev = *result.last().unwrap();
        let curr = points[i];
        let next = points[i + 1];

        // Calculate perpendicular distance from curr to line prev->next
        if !is_collinear(prev, curr, next, tolerance) {
            result.push(curr);
        }
    }

    // Always keep the last point
    result.push(*points.last().unwrap());

    result
}

/// Check if three points are collinear within tolerance.
///
/// Returns true if point `b` lies within `tolerance` distance of the line from `a` to `c`.
fn is_collinear(a: Point, b: Point, c: Point, tolerance: f64) -> bool {
    // Vector from a to c
    let ac = c - a;
    let ac_len = ac.hypot();

    if ac_len < f64::EPSILON {
        // a and c are the same point, check if b is close to them
        return a.distance(b) <= tolerance;
    }

    // Vector from a to b
    let ab = b - a;

    // Perpendicular distance = |cross product| / |ac|
    let cross = ab.x * ac.y - ab.y * ac.x;
    let distance = cross.abs() / ac_len;

    distance <= tolerance
}

/// Remove duplicate consecutive points within tolerance.
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

/// Clean up a path by removing duplicates and collinear points.
///
/// First removes points that are too close together, then removes
/// points that lie on straight lines between their neighbors.
pub fn cleanup_points(points: Vec<Point>, tolerance: f64) -> Vec<Point> {
    // First remove duplicate points (using a tighter tolerance)
    let points = remove_duplicates(&points, tolerance * 0.1);
    // Then remove collinear points
    simplify_points(&points, tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_empty() {
        let points: Vec<Point> = vec![];
        let result = simplify_points(&points, 0.1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_simplify_single_point() {
        let points = vec![Point::new(0.0, 0.0)];
        let result = simplify_points(&points, 0.1);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_simplify_two_points() {
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)];
        let result = simplify_points(&points, 0.1);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_simplify_collinear_points() {
        // Three points on a straight line
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 5.0),
            Point::new(10.0, 10.0),
        ];
        let result = simplify_points(&points, 0.1);
        // Middle point should be removed
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Point::new(0.0, 0.0));
        assert_eq!(result[1], Point::new(10.0, 10.0));
    }

    #[test]
    fn test_simplify_non_collinear_points() {
        // Three points forming a corner
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        ];
        let result = simplify_points(&points, 0.1);
        // All points should be kept
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_simplify_nearly_collinear_within_tolerance() {
        // Three points, middle one slightly off the line
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.05), // 0.05 off the line
            Point::new(10.0, 0.0),
        ];
        let result = simplify_points(&points, 0.1);
        // Middle point should be removed (within tolerance)
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_simplify_nearly_collinear_outside_tolerance() {
        // Three points, middle one clearly off the line
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.5), // 0.5 off the line
            Point::new(10.0, 0.0),
        ];
        let result = simplify_points(&points, 0.1);
        // Middle point should be kept (outside tolerance)
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_remove_duplicates_empty() {
        let points: Vec<Point> = vec![];
        let result = remove_duplicates(&points, 0.1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_remove_duplicates_no_duplicates() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(20.0, 0.0),
        ];
        let result = remove_duplicates(&points, 0.1);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_remove_duplicates_with_duplicates() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(0.05, 0.0), // Too close to previous
            Point::new(10.0, 0.0),
        ];
        let result = remove_duplicates(&points, 0.1);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Point::new(0.0, 0.0));
        assert_eq!(result[1], Point::new(10.0, 0.0));
    }

    #[test]
    fn test_cleanup_points() {
        // Points with duplicates and collinear segments
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(0.001, 0.0), // Duplicate
            Point::new(5.0, 0.0),   // Collinear
            Point::new(10.0, 0.0),
        ];
        let result = cleanup_points(points, 0.1);
        // Should remove duplicate and collinear point
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_is_collinear_exact() {
        assert!(is_collinear(
            Point::new(0.0, 0.0),
            Point::new(5.0, 5.0),
            Point::new(10.0, 10.0),
            0.001
        ));
    }

    #[test]
    fn test_is_collinear_same_start_end() {
        // When start and end are the same, check distance from middle
        assert!(is_collinear(
            Point::new(0.0, 0.0),
            Point::new(0.05, 0.0),
            Point::new(0.0, 0.0),
            0.1
        ));
    }
}
