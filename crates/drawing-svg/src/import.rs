//! SVG import functionality using usvg

use std::path::Path as FilePath;

use drawing_core::{Affine, ClipGroup, Color, Drawing, Element, Group, Path, Point, Style};
use usvg::{tiny_skia_path::PathSegment as TinyPathSegment, Node, Tree};

use crate::SvgError;

/// Result of an SVG import operation
#[derive(Debug)]
pub struct ImportResult {
    /// The imported drawing
    pub drawing: Drawing,
    /// Warnings about elements that couldn't be fully imported
    pub warnings: Vec<ImportWarning>,
}

/// Warnings generated during SVG import
#[derive(Debug, Clone)]
pub enum ImportWarning {
    /// An element type that isn't supported
    UnsupportedElement { element: String, reason: String },
    /// Text conversion failed (font not found, etc.)
    TextConversionFailed { text: String, reason: String },
    /// A clip-path was skipped
    ClipPathSkipped { id: String, reason: String },
    /// A gradient was ignored
    GradientIgnored { id: String },
}

/// How to handle filled shapes during import
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillBehavior {
    /// Ignore filled shapes that have no stroke
    Ignore,
    /// Convert fill to outline stroke
    #[default]
    ConvertToOutline,
}

/// Options for SVG import
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// How to handle filled shapes (default: ConvertToOutline)
    pub fill_behavior: FillBehavior,

    /// Default stroke width when not specified (default: 1.0)
    pub default_stroke_width: f64,

    /// Default stroke color when not specified (default: BLACK)
    pub default_stroke_color: Color,

    /// Tolerance for curve flattening (default: 0.1)
    pub curve_tolerance: f64,

    /// Import clip-paths as ClipGroups (default: true)
    pub import_clip_paths: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            fill_behavior: FillBehavior::ConvertToOutline,
            default_stroke_width: 1.0,
            default_stroke_color: Color::BLACK,
            curve_tolerance: 0.1,
            import_clip_paths: true,
        }
    }
}

/// Import an SVG file to a Drawing
pub fn import_svg(path: impl AsRef<FilePath>) -> Result<ImportResult, SvgError> {
    import_svg_with_options(path, &ImportOptions::default())
}

/// Import an SVG file to a Drawing with custom options
pub fn import_svg_with_options(
    path: impl AsRef<FilePath>,
    options: &ImportOptions,
) -> Result<ImportResult, SvgError> {
    let svg_data =
        std::fs::read(&path).map_err(|e| SvgError::Io(std::io::Error::other(e.to_string())))?;
    import_svg_data_with_options(&svg_data, options)
}

/// Import SVG from a string
pub fn import_svg_string(svg: &str) -> Result<ImportResult, SvgError> {
    import_svg_string_with_options(svg, &ImportOptions::default())
}

/// Import SVG from a string with custom options
pub fn import_svg_string_with_options(
    svg: &str,
    options: &ImportOptions,
) -> Result<ImportResult, SvgError> {
    import_svg_data_with_options(svg.as_bytes(), options)
}

/// Import SVG from raw bytes with custom options
fn import_svg_data_with_options(
    data: &[u8],
    options: &ImportOptions,
) -> Result<ImportResult, SvgError> {
    let tree = Tree::from_data(data, &usvg::Options::default())
        .map_err(|e| SvgError::Parse(e.to_string()))?;

    let size = tree.size();
    let mut drawing = Drawing::new(size.width() as f64, size.height() as f64);
    let mut warnings = Vec::new();

    // Convert the root group
    let elements = convert_group(tree.root(), options, &mut warnings);
    for element in elements {
        drawing.add(element);
    }

    Ok(ImportResult { drawing, warnings })
}

/// Convert a usvg Group to Elements
fn convert_group(
    group: &usvg::Group,
    options: &ImportOptions,
    warnings: &mut Vec<ImportWarning>,
) -> Vec<Element> {
    let mut elements = Vec::new();

    for child in group.children() {
        match child {
            Node::Group(g) => {
                let children = convert_group(g, options, warnings);
                if !children.is_empty() {
                    let transform = convert_transform(g.transform());
                    let mut group = Group::new();
                    for child in children {
                        group.push(child);
                    }

                    let element = if options.import_clip_paths && g.clip_path().is_some() {
                        // Handle clip-path
                        if let Some(clip_element) =
                            convert_clip_path(g.clip_path().unwrap(), options, warnings)
                        {
                            let mut clip_group = ClipGroup::new(clip_element);
                            clip_group.push(Element::group(group).with_transform(transform));
                            Element::clip_group(clip_group)
                        } else {
                            Element::group(group).with_transform(transform)
                        }
                    } else {
                        Element::group(group).with_transform(transform)
                    };

                    elements.push(element);
                }
            }
            Node::Path(path) => {
                if let Some(element) = convert_path(path, options, warnings) {
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
    let elements = convert_group(clip_path.root(), options, warnings);

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
fn convert_path(
    path: &usvg::Path,
    options: &ImportOptions,
    _warnings: &mut Vec<ImportWarning>,
) -> Option<Element> {
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

    // Extract style
    let style = if let Some(stroke) = path.stroke() {
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

/// Convert usvg path segments to our Path type
fn convert_path_data(segments: impl Iterator<Item = TinyPathSegment>) -> Path {
    let mut path = Path::new();

    for segment in segments {
        match segment {
            TinyPathSegment::MoveTo(pt) => {
                path = path.move_to(Point::new(pt.x as f64, pt.y as f64));
            }
            TinyPathSegment::LineTo(pt) => {
                path = path.line_to(Point::new(pt.x as f64, pt.y as f64));
            }
            TinyPathSegment::QuadTo(ctrl, end) => {
                path = path.quad_to(
                    Point::new(ctrl.x as f64, ctrl.y as f64),
                    Point::new(end.x as f64, end.y as f64),
                );
            }
            TinyPathSegment::CubicTo(ctrl1, ctrl2, end) => {
                path = path.cubic_to(
                    Point::new(ctrl1.x as f64, ctrl1.y as f64),
                    Point::new(ctrl2.x as f64, ctrl2.y as f64),
                    Point::new(end.x as f64, end.y as f64),
                );
            }
            TinyPathSegment::Close => {
                path = path.close();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_simple_line() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <line x1="0" y1="0" x2="100" y2="100" stroke="black"/>
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert_eq!(result.drawing.width, 100.0);
        assert_eq!(result.drawing.height, 100.0);
        assert!(!result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_circle() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <circle cx="50" cy="50" r="25" stroke="red" fill="none"/>
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert!(!result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_path() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M10,10 L90,10 L90,90 L10,90 Z" stroke="black" fill="none"/>
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert!(!result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_with_transform() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <g transform="translate(10, 10)">
                <rect x="0" y="0" width="20" height="20" stroke="black" fill="none"/>
            </g>
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert!(!result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_filled_shape_convert_to_outline() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
        </svg>"#;

        let options = ImportOptions {
            fill_behavior: FillBehavior::ConvertToOutline,
            ..Default::default()
        };

        let result = import_svg_string_with_options(svg, &options).unwrap();
        assert!(!result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_filled_shape_ignore() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
        </svg>"#;

        let options = ImportOptions {
            fill_behavior: FillBehavior::Ignore,
            ..Default::default()
        };

        let result = import_svg_string_with_options(svg, &options).unwrap();
        // The filled rect should be ignored
        assert!(result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_stroke_color() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <line x1="0" y1="0" x2="100" y2="100" stroke="#ff0000" stroke-width="2"/>
        </svg>"##;

        let result = import_svg_string(svg).unwrap();
        assert!(!result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_empty_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert_eq!(result.drawing.width, 100.0);
        assert_eq!(result.drawing.height, 100.0);
        assert!(result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_nested_groups() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <g transform="translate(10, 10)">
                <g transform="rotate(45)">
                    <line x1="0" y1="0" x2="50" y2="0" stroke="black"/>
                </g>
            </g>
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert!(!result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_multiple_shapes() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="30" height="30" stroke="black" fill="none"/>
            <circle cx="70" cy="30" r="15" stroke="blue" fill="none"/>
            <ellipse cx="50" cy="70" rx="30" ry="15" stroke="green" fill="none"/>
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert!(!result.drawing.elements.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_import_bezier_curves() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M10,10 Q50,0 90,10" stroke="black" fill="none"/>
            <path d="M10,50 C30,30 70,70 90,50" stroke="black" fill="none"/>
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert!(!result.drawing.elements.is_empty());
    }

    #[test]
    fn test_import_polyline() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <polyline points="10,10 50,50 90,10" stroke="black" fill="none"/>
            <polygon points="10,60 50,90 90,60" stroke="blue" fill="none"/>
        </svg>"#;

        let result = import_svg_string(svg).unwrap();
        assert!(!result.drawing.elements.is_empty());
    }
}
