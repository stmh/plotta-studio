//! Signature trait for adorning drawings.
//!
//! A [`Signature`] is any renderable mark that can be placed in the corner
//! of a framed drawing. The implementation owns both the geometry **and**
//! the sizing logic: given a requested target height in drawing units, it
//! returns a rendered [`Element`] and the resulting width. The frame does
//! not apply any further scaling.
//!
//! This crate ships [`PlaceholderSignature`] ("xxx") for demos. Personal
//! signatures live outside the library — implement [`Signature`] in your
//! own crate (e.g. an `art-utils` crate alongside your sketches).

use drawing_core::{Element, Group, Path};

/// A signature mark renderable as a drawing element.
///
/// The signature controls its own aspect ratio: the frame only specifies a
/// target height; the signature returns whatever width its glyphs need.
/// The returned element must be normalized so its bounding-box origin sits
/// at `(0, 0)` and its height equals the requested `target_height`.
pub trait Signature: Send + Sync {
    /// Render the signature at the given height (in drawing units).
    ///
    /// Returns the element (origin at (0, 0)) and the rendered width.
    fn render(&self, target_height: f64) -> (Element, f64);
}

/// A trivial placeholder signature drawing the letters "xxx" as a row of
/// square X-shaped crossed strokes. Used in examples and as a sensible
/// default when no personal signature is provided.
///
/// Each X glyph is square at half the requested target height (the
/// placeholder is intentionally subtle so it does not visually dominate
/// the corner of a drawing). Overall aspect ratio is roughly 3.4 of the
/// rendered height (≈1.7 of the requested target_height).
#[derive(Debug, Clone, Copy, Default)]
pub struct PlaceholderSignature;

impl Signature for PlaceholderSignature {
    fn render(&self, target_height: f64) -> (Element, f64) {
        // Render at half the requested height so the demo signature is
        // visually unobtrusive. Each X is square; spacing is 20% of the
        // rendered glyph height.
        let effective_h = target_height * 0.5;
        let glyph_w = effective_h;
        let glyph_h = effective_h;
        let spacing = effective_h * 0.2;

        let mut group = Group::new();
        for i in 0..3 {
            let x = i as f64 * (glyph_w + spacing);
            // Diagonal \
            group.push(Element::path(
                Path::new()
                    .move_to((x, 0.0))
                    .line_to((x + glyph_w, glyph_h)),
            ));
            // Diagonal /
            group.push(Element::path(
                Path::new()
                    .move_to((x + glyph_w, 0.0))
                    .line_to((x, glyph_h)),
            ));
        }
        let total_w = 3.0 * glyph_w + 2.0 * spacing;
        (Element::group(group), total_w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_renders_at_half_requested_height() {
        // target_height 10 → effective 5; 3 glyphs × 5 + 2 gaps × 1 = 17
        let (_, w) = PlaceholderSignature.render(10.0);
        assert!((w - 17.0).abs() < 1e-9, "got {w}");
    }

    #[test]
    fn placeholder_renders_as_group() {
        let (el, _) = PlaceholderSignature.render(7.0);
        match el.shape {
            drawing_core::Shape::Group(_) => {}
            _ => panic!("Expected Group shape"),
        }
    }

    #[test]
    fn signature_is_object_safe() {
        // Compile-time check: trait is object-safe.
        let _: Box<dyn Signature> = Box::new(PlaceholderSignature);
    }
}
