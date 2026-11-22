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

                    let element = match g.clip_path() {
                        Some(cp) if !for_clip_path && options.import_clip_paths => {
                            let clip_element = convert_clip_path(cp, options, warnings);
                            let mut clip_group = ClipGroup::new(clip_element);
                            clip_group.push(Element::group(group));
                            Element::clip_group(clip_group)
                        }
                        _ => Element::group(group),
                    };

                    elements.push(element);
                }
            }
            Node::Path(path) => {
                if let Some(element) = convert_path(path, options, warnings, for_clip_path) {
                    elements.push(element);
                }
            }
            Node::Image(_) => {
                warnings.push(ImportWarning::UnsupportedElement {
                    element: "image".to_string(),
                    reason: "Embedded images are not supported".to_string(),
                });
            }
            // Unreachable: no fonts are loaded, so usvg drops all text before
            // conversion. Dropped `<text>` is reported from the XML source.
            Node::Text(_) => {}
        }
    }

    elements
}

/// Convert a usvg ClipPath to an Element
///
/// usvg removes clip paths without geometry together with the content they
/// clip, so `clip_path` always has at least one shape here.
fn convert_clip_path(
    clip_path: &usvg::ClipPath,
    options: &ImportOptions,
    warnings: &mut Vec<ImportWarning>,
) -> Element {
    // Pass `true` for `for_clip_path` to extract geometry without stroke/fill filtering
    let mut group = Group::new();
    for element in convert_group(clip_path.root(), options, warnings, true) {
        group.push(element);
    }
    Element::group(group)
}

/// Convert a usvg Path to an Element
///
/// When `for_clip_path` is true, we ignore stroke/fill attributes and just extract
/// the geometry with a default style. This is used for clip path shapes.
fn convert_path(
    path: &usvg::Path,
    options: &ImportOptions,
    warnings: &mut Vec<ImportWarning>,
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
        let color = extract_paint_color(stroke.paint(), options, warnings);
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
///
/// Gradient and pattern paints fall back to the default stroke color and
/// emit a `GradientIgnored` warning.
fn extract_paint_color(
    paint: &usvg::Paint,
    options: &ImportOptions,
    warnings: &mut Vec<ImportWarning>,
) -> Color {
    let id = match paint {
        usvg::Paint::Color(c) => return Color::rgb(c.red, c.green, c.blue),
        usvg::Paint::LinearGradient(g) => g.id(),
        usvg::Paint::RadialGradient(g) => g.id(),
        usvg::Paint::Pattern(p) => p.id(),
    };
    warnings.push(ImportWarning::GradientIgnored { id: id.to_string() });
    options.default_stroke_color
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
