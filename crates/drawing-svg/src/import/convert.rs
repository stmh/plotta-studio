//! SVG to Drawing conversion functions

use drawing_core::{Affine, ClipGroup, Color, Element, Group, Path, Point, Style};
use usvg::{tiny_skia_path::PathSegment as TinyPathSegment, Node};

use super::{FillBehavior, ImportOptions, ImportWarning};

/// Convert a usvg Group to Elements
///
/// The `for_clip_path` parameter indicates whether we're converting geometry for a clip path,
/// in which case we ignore stroke/fill attributes and just extract the geometry.
pub(super) fn convert_group(
    group: &usvg::Group,
    options: &ImportOptions,
    warnings: &mut Vec<ImportWarning>,
    for_clip_path: bool,
) -> Vec<Element> {
    let mut elements = Vec::new();

    for child in group.children() {
        match child {
            Node::Group(g) => {
                let children = convert_group(g, options, warnings, for_clip_path);
                if !children.is_empty() {
                    // Note: We don't apply g.transform() here because child paths
                    // already have their abs_transform() applied, which includes
                    // all parent transforms.
                    let mut group = Group::new();
                    for child in children {
                        group.push(child);
                    }

                    let element = if let Some(cp) = g.clip_path() {
                        if !for_clip_path && options.import_clip_paths {
                            // Handle clip-path
                            if let Some(clip_element) = convert_clip_path(cp, options, warnings) {
                                let mut clip_group = ClipGroup::new(clip_element);
                                clip_group.push(Element::group(group));
                                Element::clip_group(clip_group)
                            } else {
                                Element::group(group)
                            }
                        } else {
                            Element::group(group)
                        }
                    } else {
                        Element::group(group)
                    };

                    elements.push(element);
                }
            }
            Node::Path(path) => {
                if let Some(element) = convert_path(path, options, for_clip_path) {
                    elements.push(element);
                }
            }
            Node::Image(_) => {
                warnings.push(ImportWarning::UnsupportedElement {
                    element: "image".to_string(),
                    reason: "Embedded images are not supported".to_string(),
                });
            }
            Node::Text(_text) => {
                // usvg should convert text to paths, but if we still see a text node
                // it means conversion wasn't possible
                warnings.push(ImportWarning::TextConversionFailed {
                    text: "text element".to_string(),
                    reason: "Text node found - font may not be available for conversion"
                        .to_string(),
                });
            }
        }
    }

    elements
}

/// Convert a usvg ClipPath to an Element
fn convert_clip_path(
    clip_path: &usvg::ClipPath,
    options: &ImportOptions,
    warnings: &mut Vec<ImportWarning>,
) -> Option<Element> {
    // Pass `true` for `for_clip_path` to extract geometry without stroke/fill filtering
    let elements = convert_group(clip_path.root(), options, warnings, true);

    if elements.is_empty() {
        warnings.push(ImportWarning::ClipPathSkipped {
            id: clip_path.id().to_string(),
            reason: "Empty clip path".to_string(),
        });
        return None;
    }

    // Combine all clip elements into a group
    let mut group = Group::new();
    for element in elements {
        group.push(element);
    }

    Some(Element::group(group))
}

/// Convert a usvg Path to an Element
///
/// When `for_clip_path` is true, we ignore stroke/fill attributes and just extract
/// the geometry with a default style. This is used for clip path shapes.
fn convert_path(
    path: &usvg::Path,
    options: &ImportOptions,
    for_clip_path: bool,
) -> Option<Element> {
    // For clip paths, always import the geometry regardless of stroke/fill
    if !for_clip_path {
        let has_stroke = path.stroke().is_some();
        let has_fill = path.fill().is_some();

        // Determine if we should import this path
        if !has_stroke {
            match options.fill_behavior {
                FillBehavior::Ignore => {
                    if has_fill {
                        // Silently skip filled-only shapes when Ignore is set
                        return None;
                    }
                }
                FillBehavior::ConvertToOutline => {
                    // We'll convert the fill to an outline stroke
                }
            }
        }
    }

    // Extract style - for clip paths use default style (the style won't be rendered anyway)
    let style = if for_clip_path {
        Style::new(options.default_stroke_width, options.default_stroke_color)
    } else if let Some(stroke) = path.stroke() {
        let color = extract_paint_color(stroke.paint(), options);
        let width = stroke.width().get() as f64;
        Style::new(width, color)
    } else {
        // Use default style for fill-only shapes converted to outline
        Style::new(options.default_stroke_width, options.default_stroke_color)
    };

    // Convert path data
    let drawing_path = convert_path_data(path.data().segments());

    if drawing_path.is_empty() {
        return None;
    }

    let transform = convert_transform(path.abs_transform());

    Some(
        Element::path(drawing_path)
            .with_style(style)
            .with_transform(transform),
    )
}

/// Convert usvg path segments to our Path type.
///
/// # Coordinate System
///
/// usvg provides path segments in the SVG coordinate system (origin top-left,
/// Y increases downward). All coordinates are absolute - usvg has already
/// resolved any relative path commands (like 'l', 'm') to absolute coordinates.
///
/// Transforms are NOT applied here. The caller applies the path's `abs_transform()`
/// separately via `Element::with_transform()`, which includes all ancestor
/// group transforms accumulated by usvg.
fn convert_path_data(segments: impl Iterator<Item = TinyPathSegment>) -> Path {
    let mut path = Path::new();

    for segment in segments {
        match segment {
            TinyPathSegment::MoveTo(pt) => {
                path.push_move_to(Point::new(pt.x as f64, pt.y as f64));
            }
            TinyPathSegment::LineTo(pt) => {
                path.push_line_to(Point::new(pt.x as f64, pt.y as f64));
            }
            TinyPathSegment::QuadTo(ctrl, end) => {
                path.push_quad_to(
                    Point::new(ctrl.x as f64, ctrl.y as f64),
                    Point::new(end.x as f64, end.y as f64),
                );
            }
            TinyPathSegment::CubicTo(ctrl1, ctrl2, end) => {
                path.push_cubic_to(
                    Point::new(ctrl1.x as f64, ctrl1.y as f64),
                    Point::new(ctrl2.x as f64, ctrl2.y as f64),
                    Point::new(end.x as f64, end.y as f64),
                );
            }
            TinyPathSegment::Close => {
                path.push_close();
            }
        }
    }

    path
}

/// Extract color from usvg Paint
fn extract_paint_color(paint: &usvg::Paint, options: &ImportOptions) -> Color {
    match paint {
        usvg::Paint::Color(c) => Color::rgb(c.red, c.green, c.blue),
        usvg::Paint::LinearGradient(_) | usvg::Paint::RadialGradient(_) => {
            // Use default color for gradients
            options.default_stroke_color
        }
        usvg::Paint::Pattern(_) => options.default_stroke_color,
    }
}

/// Convert usvg Transform to our Affine
fn convert_transform(t: usvg::Transform) -> Affine {
    Affine::new([
        t.sx as f64,
        t.ky as f64,
        t.kx as f64,
        t.sy as f64,
        t.tx as f64,
        t.ty as f64,
    ])
}
