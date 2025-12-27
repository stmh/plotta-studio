//! Signature utility for adding a handwritten signature to drawings

use drawing_core::{Element, Group, Path};

/// Returns a handwritten signature as a Group of path elements.
///
/// The signature is returned at its original SVG coordinates (approximately 168x141 units).
/// Use `.scale()` and `.translate()` to position and size it appropriately.
///
/// # Example
///
/// ```ignore
/// use drawing_utils::signature;
/// use drawing_core::Drawing;
///
/// let mut drawing = Drawing::a4_landscape();
/// let sig = signature()
///     .scale(0.1, 0.1)  // Scale down to ~16.8x14.1 units
///     .translate(250.0, 180.0);  // Position in lower right
/// drawing.add(sig);
/// ```
pub fn signature() -> Element {
    let mut group = Group::new();

    // Path 1: First flourish (S-like curve)
    let path1 = Path::new()
        .move_to((191.18, 406.39))
        .cubic_to((186.74, 407.22), (176.95, 411.87), (175.95, 416.19))
        .cubic_to((174.95, 420.51), (170.29, 425.66), (189.08, 439.71))
        .cubic_to((207.87, 453.76), (214.52, 458.42), (211.03, 474.05))
        .cubic_to((207.54, 489.68), (199.95, 499.63), (193.44, 501.79))
        .cubic_to((183.48, 505.10), (169.29, 513.51), (168.17, 475.67));

    // Path 2: Vertical stroke
    let path2 = Path::new()
        .move_to((210.22, 365.88))
        .cubic_to((214.76, 393.22), (217.80, 411.03), (222.40, 433.00))
        .cubic_to((227.16, 455.72), (229.63, 469.08), (231.16, 476.05))
        .cubic_to((233.18, 485.22), (235.14, 486.41), (237.15, 487.58));

    // Path 3: Multiple connected curves (main signature body)
    let path3 = Path::new()
        .move_to((213.21, 445.69))
        .cubic_to((218.52, 439.14), (233.93, 416.64), (236.48, 413.53))
        .cubic_to((239.03, 410.43), (242.35, 405.88), (249.89, 422.84))
        .cubic_to((257.43, 439.80), (261.75, 454.21), (264.41, 470.17))
        .cubic_to((268.51, 445.89), (269.97, 439.01), (271.84, 433.92))
        .cubic_to((275.05, 425.16), (278.62, 419.03), (286.92, 435.25))
        .cubic_to((296.90, 454.76), (299.89, 464.74), (306.43, 441.28))
        .cubic_to((311.13, 424.43), (317.18, 408.20), (326.05, 421.39))
        .cubic_to((334.92, 434.58), (345.23, 451.98), (350.88, 446.77))
        .cubic_to((356.53, 441.56), (361.34, 436.90), (363.31, 433.08));

    // Path 4: Final flourish
    let path4 = Path::new()
        .move_to((340.46, 334.83))
        .cubic_to((339.57, 337.27), (340.24, 340.15), (342.23, 349.46))
        .cubic_to((344.22, 358.77), (356.27, 417.59), (357.97, 425.06))
        .cubic_to((363.31, 448.56), (365.06, 423.73), (370.16, 408.65))
        .cubic_to((375.26, 393.57), (380.80, 394.77), (386.79, 403.81))
        .cubic_to((392.78, 412.86), (405.74, 437.15), (416.94, 421.28))
        .cubic_to((422.26, 413.74), (424.70, 409.66), (426.25, 406.38));

    group.push(Element::path(path1));
    group.push(Element::path(path2));
    group.push(Element::path(path3));
    group.push(Element::path(path4));

    Element::group(group)
}

/// Returns the bounding box of the signature in original coordinates.
///
/// Returns `(x, y, width, height)` where:
/// - x: 168.17 (leftmost point)
/// - y: 334.83 (topmost point)
/// - width: ~258.08 (426.25 - 168.17)
/// - height: ~166.96 (501.79 - 334.83)
pub fn signature_bounds() -> (f64, f64, f64, f64) {
    (168.17, 334.83, 258.08, 166.96)
}

/// Returns a signature normalized to start at origin (0, 0).
///
/// This is a convenience function that translates the signature so its
/// top-left corner is at the origin, making it easier to position.
///
/// # Example
///
/// ```ignore
/// use drawing_utils::signature_normalized;
///
/// let sig = signature_normalized()
///     .scale(0.1, 0.1)  // Scale to ~25.8x16.7 units
///     .translate(250.0, 180.0);  // Position where needed
/// ```
pub fn signature_normalized() -> Element {
    let (x, y, _, _) = signature_bounds();
    signature().translate(-x, -y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_returns_group() {
        let sig = signature();
        match sig.shape {
            drawing_core::Shape::Group(_) => {}
            _ => panic!("Expected Group shape"),
        }
    }

    #[test]
    fn test_signature_normalized_returns_group() {
        let sig = signature_normalized();
        match sig.shape {
            drawing_core::Shape::Group(_) => {}
            _ => panic!("Expected Group shape"),
        }
    }

    #[test]
    fn test_signature_bounds() {
        let (x, y, w, h) = signature_bounds();
        assert!(x > 0.0);
        assert!(y > 0.0);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }
}
