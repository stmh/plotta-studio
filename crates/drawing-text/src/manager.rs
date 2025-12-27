//! Font manager for loading and registering fonts
//!
//! The `FontManager` provides font loading functionality and works with
//! a `FontRegistry` from drawing-core for storage.

use std::path::Path;
use std::sync::Arc;

use drawing_core::{FontRef, FontRegistry};

use crate::error::FontError;
use crate::font::FontFormat;
use crate::hershey::Hershey;
use crate::svgfont::SvgFont;
use crate::vsf::VsfFont;

/// Font manager for loading fonts into a registry.
///
/// The FontManager handles font loading from various sources (files, directories,
/// built-in fonts) and registers them with a `FontRegistry`.
///
/// # Example
///
/// ```rust,ignore
/// use drawing_text::{FontManager, Hershey};
///
/// // Create manager with its own registry
/// let manager = FontManager::new();
///
/// // Load a built-in Hershey font
/// manager.load_hershey(Hershey::Simplex)?;
///
/// // Get font from registry using the enum
/// let font = manager.registry().get(Hershey::Simplex);
/// ```
pub struct FontManager {
    registry: Arc<FontRegistry>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self {
            registry: Arc::new(FontRegistry::new()),
        }
    }
}

impl FontManager {
    /// Create a new font manager with a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new font manager with the given registry.
    pub fn with_registry(registry: Arc<FontRegistry>) -> Self {
        Self { registry }
    }

    /// Get the underlying font registry.
    pub fn registry(&self) -> &Arc<FontRegistry> {
        &self.registry
    }

    /// Load a built-in Hershey font variant.
    ///
    /// The font is registered with its canonical name (e.g., "Hershey Simplex")
    /// and returned for immediate use.
    pub fn load_hershey(&self, variant: Hershey) -> Result<FontRef, FontError> {
        let font = variant.load()?;
        let font_ref: FontRef = Arc::new(font);
        self.registry.register(font_ref.clone());
        Ok(font_ref)
    }

    /// Load all built-in Hershey font variants.
    ///
    /// Fonts are registered silently. Use `registry().get()` to retrieve them.
    /// Returns the count of fonts loaded on success.
    /// Fails fast on the first error encountered.
    pub fn load_all_hershey(&self) -> Result<usize, FontError> {
        for variant in Hershey::all() {
            let _ = self.load_hershey(*variant)?;
        }
        Ok(Hershey::all().len())
    }

    /// Load a font from string content.
    ///
    /// The font is registered and returned for immediate use.
    ///
    /// # Arguments
    ///
    /// * `content` - The font content as a string
    /// * `format` - The font format to use for parsing
    pub fn load_from_str(&self, content: &str, format: FontFormat) -> Result<FontRef, FontError> {
        let font: FontRef = match format {
            FontFormat::Hershey => {
                return Err(FontError::UnsupportedFormat(
                    "Use load_hershey() for Hershey fonts".to_string(),
                ));
            }
            FontFormat::Vsf => Arc::new(VsfFont::from_json(content)?),
            FontFormat::SvgFont => Arc::new(SvgFont::parse(content)?),
            FontFormat::Ufo => {
                return Err(FontError::UnsupportedFormat(
                    "UFO format not yet implemented".to_string(),
                ));
            }
        };

        self.registry.register(font.clone());
        Ok(font)
    }

    /// Load a single font file.
    ///
    /// The font is registered and returned for immediate use.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the font file
    /// * `format` - The font format to use for parsing
    pub fn load_file(
        &self,
        path: impl AsRef<Path>,
        format: FontFormat,
    ) -> Result<FontRef, FontError> {
        let path = path.as_ref();

        // Read file contents
        let contents = std::fs::read_to_string(path)
            .map_err(|e| FontError::IoError(path.to_path_buf(), e.to_string()))?;

        self.load_from_str(&contents, format)
    }

    /// Load all fonts from a directory.
    ///
    /// Returns the count of fonts loaded on success.
    /// Fails fast on the first error encountered.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory containing font files
    /// * `format` - The font format to use for parsing all files
    pub fn load_directory(
        &self,
        path: impl AsRef<Path>,
        format: FontFormat,
    ) -> Result<usize, FontError> {
        let path = path.as_ref();

        if !path.is_dir() {
            return Err(FontError::InvalidPath(path.to_path_buf()));
        }

        let extension = match format {
            FontFormat::Hershey => "jhf",
            FontFormat::Vsf => "vsf",
            FontFormat::SvgFont => "svg",
            FontFormat::Ufo => {
                return Err(FontError::UnsupportedFormat(
                    "UFO format not yet implemented".to_string(),
                ));
            }
        };

        let mut count = 0;

        let entries = std::fs::read_dir(path)
            .map_err(|e| FontError::IoError(path.to_path_buf(), e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| FontError::IoError(path.to_path_buf(), e.to_string()))?;
            let file_path = entry.path();

            // Skip if not a file or wrong extension
            if !file_path.is_file() {
                continue;
            }

            let file_ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

            if file_ext != extension {
                continue;
            }

            // Load the font (fail fast on error)
            self.load_file(&file_path, format)?;
            count += 1;
        }

        Ok(count)
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<FontRef> {
        self.registry.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_manager() -> (Arc<FontRegistry>, FontManager) {
        let registry = Arc::new(FontRegistry::new());
        let manager = FontManager::with_registry(registry.clone());
        (registry, manager)
    }

    #[test]
    fn test_new_manager() {
        let (registry, manager) = create_manager();
        assert!(Arc::ptr_eq(manager.registry(), &registry));
    }

    #[test]
    fn test_load_hershey() {
        let (registry, manager) = create_manager();

        let result = manager.load_hershey(Hershey::Simplex);
        assert!(result.is_ok());

        let font = result.unwrap();
        assert_eq!(font.name(), "Hershey Simplex");
        assert!(font.has_glyph('A'));

        // Font should also be in registry
        assert!(registry.has(Hershey::Simplex));
        assert!(registry.has("Hershey Simplex"));
    }

    #[test]
    fn test_load_all_hershey() {
        let (registry, manager) = create_manager();

        let result = manager.load_all_hershey();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8);

        // All variants should be loaded
        for variant in Hershey::all() {
            assert!(registry.has(*variant));
        }
    }

    #[test]
    fn test_load_file_vsf() {
        let (registry, manager) = create_manager();

        let result = manager.load_file("../../fonts/vsf/asteroids.vsf", FontFormat::Vsf);

        assert!(result.is_ok());
        let font = result.unwrap();
        assert!(font.has_glyph('A'));

        // Font should also be in registry
        assert!(registry.has(font.name()));
    }

    #[test]
    fn test_load_directory_vsf() {
        let (_registry, manager) = create_manager();

        let result = manager.load_directory("../../fonts/vsf", FontFormat::Vsf);

        assert!(result.is_ok());
        let count = result.unwrap();
        assert!(count >= 3); // asteroids, apple410, minf
    }

    #[test]
    fn test_load_directory_invalid_path() {
        let (_registry, manager) = create_manager();

        let result = manager.load_directory("/nonexistent/path", FontFormat::Vsf);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_file_hershey_not_supported() {
        let (_registry, manager) = create_manager();

        let result = manager.load_file("some/path.jhf", FontFormat::Hershey);
        assert!(result.is_err());
    }

    #[test]
    fn test_hershey_enum_asref() {
        // Hershey enum implements AsRef<str>
        let name: &str = Hershey::Simplex.as_ref();
        assert_eq!(name, "Hershey Simplex");
    }

    #[test]
    fn test_hershey_enum_all() {
        let all = Hershey::all();
        assert_eq!(all.len(), 8);
    }
}
