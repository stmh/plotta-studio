//! SVG import functionality using usvg

mod convert;

#[cfg(test)]
mod tests;

use std::path::Path as FilePath;

use drawing_core::Drawing;

use crate::SvgError;
use convert::convert_group;

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
    pub default_stroke_color: drawing_core::Color,

    /// Import clip-paths as ClipGroups (default: true)
    pub import_clip_paths: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            fill_behavior: FillBehavior::ConvertToOutline,
            default_stroke_width: 1.0,
            default_stroke_color: drawing_core::Color::BLACK,
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
    let svg_data = std::fs::read(&path).map_err(SvgError::Io)?;
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
    let tree = usvg::Tree::from_data(data, &usvg::Options::default())
        .map_err(|e| SvgError::Parse(e.to_string()))?;

    let size = tree.size();
    let mut drawing = Drawing::new(size.width() as f64, size.height() as f64);
    let mut warnings = Vec::new();

    // Convert the root group
    let elements = convert_group(tree.root(), options, &mut warnings, false);
    for element in elements {
        drawing.add(element);
    }

    Ok(ImportResult { drawing, warnings })
}
