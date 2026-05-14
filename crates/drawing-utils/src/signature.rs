//! Signature trait for adorning drawings.
//!
//! A [`Signature`] is any rendererable mark that can be placed in the corner
//! of a framed drawing. Implementations supply the actual paths/shapes and
//! report a natural size so the frame can scale them to a configured height.
//!
//! This crate ships a [`PlaceholderSignature`] ("xxx") suitable for demos.
//! Personal signatures live outside the library — implement [`Signature`]
//! in your own crate (e.g. an `art-utils` crate alongside your sketches).

use drawing_core::{Element, Group, Path};

/// A signature mark renderable as a drawing element.
///
/// Implementations should render their content in a coordinate system whose
/// natural extent is reported by [`Signature::natural_size`]. The frame
/// rendering code scales the returned element uniformly so the rendered
/// height matches the configured signature height; the origin of the
/// returned element is expected to be at the top-left of the signature
/// bounding box (i.e. the signature should already be normalized).
pub trait Signature: Send + Sync {
    /// Render the signature with its bounding-box origin at (0, 0).
    ///
    /// Returned coordinates are in the implementation's own natural units;
    /// the frame will apply a uniform scale to convert to drawing units.
    fn render(&self) -> Element;

    /// Natural width and height of the signature, in its own coordinate units.
    ///
    /// Used by the frame layout to compute the scale factor and to position
    /// the signature relative to the right edge of the frame.
    fn natural_size(&self) -> (f64, f64);
}

/// A trivial placeholder signature drawing the letters "xxx" as a row of
/// hand-drawn-style crossed strokes. Used in examples and as a sensible
/// default when no personal signature is provided.
///
/// Natural size: 34 × 10 units.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlaceholderSignature;

impl Signature for PlaceholderSignature {
    fn render(&self) -> Element {
        // Three "x" marks side by side. Each glyph is 10 wide × 10 tall,
        // with 2 units of spacing between them.
        let glyph_w = 10.0;
        let glyph_h = 10.0;
        let spacing = 2.0;

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
        Element::group(group)
    }

    fn natural_size(&self) -> (f64, f64) {
        // 3 glyphs × 10 wide + 2 gaps × 2 = 34
        (34.0, 10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_renders_as_group() {
        let sig = PlaceholderSignature;
        match sig.render().shape {
            drawing_core::Shape::Group(_) => {}
            _ => panic!("Expected Group shape"),
        }
    }

    #[test]
    fn placeholder_has_positive_size() {
        let (w, h) = PlaceholderSignature.natural_size();
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn signature_is_object_safe() {
        // Compile-time check: trait is object-safe.
        let _: Box<dyn Signature> = Box::new(PlaceholderSignature);
    }
}
