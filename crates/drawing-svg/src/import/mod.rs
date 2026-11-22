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
    /// A `<text>` element was dropped (no fonts are loaded for import)
    TextConversionFailed { text: String, reason: String },
    /// A clip-path was skipped
    ///
    /// Not emitted by the current importer: usvg drops content clipped by an
    /// empty clip path before conversion. Kept for API compatibility.
    ClipPathSkipped { id: String, reason: String },
    /// A gradient or pattern stroke paint was replaced by the default stroke color
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
    let parse_err = |e: usvg::Error| SvgError::Parse(e.to_string());

    // Same steps as `usvg::Tree::from_data`, but keeping the XML document so
    // `<text>` can be reported: usvg drops it because no fonts are loaded.
    let decompressed;
    let data = if data.starts_with(&[0x1f, 0x8b]) {
        decompressed = usvg::decompress_svgz(data).map_err(parse_err)?;
        &decompressed[..]
    } else {
        data
    };
    let text = std::str::from_utf8(data).map_err(|_| parse_err(usvg::Error::NotAnUtf8Str))?;
    let xml_opt = usvg::roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let xml = usvg::roxmltree::Document::parse_with_options(text, xml_opt)
        .map_err(|e| parse_err(usvg::Error::ParsingFailed(e)))?;

    let mut warnings = text_warnings(&xml);

    let tree = usvg::Tree::from_xmltree(&xml, &usvg::Options::default()).map_err(parse_err)?;

    let size = tree.size();
    let mut drawing = Drawing::new(size.width() as f64, size.height() as f64);

    // Convert the root group
    let elements = convert_group(tree.root(), options, &mut warnings, false);
    for element in elements {
        drawing.add(element);
    }

    Ok(ImportResult { drawing, warnings })
}

/// One warning per SVG `<text>` element: the import loads no fonts, so usvg
/// drops all text before conversion.
fn text_warnings(xml: &usvg::roxmltree::Document) -> Vec<ImportWarning> {
    const SVG_NS: &str = "http://www.w3.org/2000/svg";
    xml.descendants()
        .filter(|n| n.has_tag_name((SVG_NS, "text")))
        .map(|n| {
            let content: Vec<&str> = n
                .descendants()
                .filter(|d| d.is_text())
                .filter_map(|d| d.text())
                .flat_map(str::split_whitespace)
                .collect();
            ImportWarning::TextConversionFailed {
                text: content.join(" "),
                reason: "Text is not imported: no fonts are loaded for SVG import".to_string(),
            }
        })
        .collect()
}
