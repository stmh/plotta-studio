//! Font registry for centralized font storage
//!
//! The `FontRegistry` provides thread-safe storage for fonts, allowing
//! fonts to be registered and looked up by name.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::font_types::Font;

/// Shared reference to a font (thread-safe)
pub type FontRef = Arc<dyn Font + Send + Sync>;

/// Thread-safe font registry for storing and retrieving fonts by name.
///
/// # Example
///
/// ```rust,ignore
/// use drawing_core::{FontRegistry, FontRef};
/// use std::sync::Arc;
///
/// let registry = Arc::new(FontRegistry::new());
///
/// // Register a font (font.name() determines the key)
/// registry.register(my_font);
///
/// // Look up by name
/// let font = registry.get("My Font").unwrap();
/// ```
pub struct FontRegistry {
    fonts: RwLock<HashMap<String, FontRef>>,
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FontRegistry {
    /// Create a new empty font registry.
    pub fn new() -> Self {
        Self {
            fonts: RwLock::new(HashMap::new()),
        }
    }

    /// Get a font by name.
    ///
    /// Returns `None` if the font is not registered.
    pub fn get(&self, name: impl AsRef<str>) -> Option<FontRef> {
        let fonts = self.fonts.read().unwrap();
        fonts.get(name.as_ref()).cloned()
    }

    /// Register a font.
    ///
    /// The font's `name()` method determines the registry key.
    /// If a font with the same name already exists, it will be replaced.
    pub fn register(&self, font: FontRef) {
        let name = font.name().to_string();
        let mut fonts = self.fonts.write().unwrap();
        fonts.insert(name, font);
    }

    /// Check if a font is registered.
    pub fn has(&self, name: impl AsRef<str>) -> bool {
        let fonts = self.fonts.read().unwrap();
        fonts.contains_key(name.as_ref())
    }

    /// List all registered font names.
    ///
    /// The returned list is sorted alphabetically.
    pub fn list(&self) -> Vec<String> {
        let fonts = self.fonts.read().unwrap();
        let mut names: Vec<String> = fonts.keys().cloned().collect();
        names.sort();
        names
    }

    /// Remove a font by name.
    ///
    /// Returns the removed font if it existed.
    pub fn remove(&self, name: impl AsRef<str>) -> Option<FontRef> {
        let mut fonts = self.fonts.write().unwrap();
        fonts.remove(name.as_ref())
    }

    /// Remove all fonts from the registry.
    pub fn clear(&self) {
        let mut fonts = self.fonts.write().unwrap();
        fonts.clear();
    }

    /// Get the number of registered fonts.
    pub fn len(&self) -> usize {
        let fonts = self.fonts.read().unwrap();
        fonts.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        let fonts = self.fonts.read().unwrap();
        fonts.is_empty()
    }
}

impl std::fmt::Debug for FontRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontRegistry")
            .field("fonts", &self.list())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font_types::{FontMetrics, Glyph};

    /// Test font implementation
    struct TestFont {
        name: String,
    }

    impl TestFont {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl Font for TestFont {
        fn name(&self) -> &str {
            &self.name
        }

        fn glyph(&self, _c: char) -> Option<Glyph> {
            None
        }

        fn metrics(&self) -> FontMetrics {
            FontMetrics::default()
        }

        fn available_chars(&self) -> Vec<char> {
            Vec::new()
        }
    }

    #[test]
    fn test_new_registry_is_empty() {
        let registry = FontRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let registry = FontRegistry::new();
        let font: FontRef = Arc::new(TestFont::new("Test Font"));

        registry.register(font);

        assert!(registry.has("Test Font"));
        assert!(!registry.has("Other Font"));

        let retrieved = registry.get("Test Font");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "Test Font");
    }

    #[test]
    fn test_list_is_sorted() {
        let registry = FontRegistry::new();

        registry.register(Arc::new(TestFont::new("Zebra")));
        registry.register(Arc::new(TestFont::new("Alpha")));
        registry.register(Arc::new(TestFont::new("Middle")));

        let list = registry.list();
        assert_eq!(list, vec!["Alpha", "Middle", "Zebra"]);
    }

    #[test]
    fn test_remove() {
        let registry = FontRegistry::new();
        registry.register(Arc::new(TestFont::new("Test Font")));

        assert!(registry.has("Test Font"));

        let removed = registry.remove("Test Font");
        assert!(removed.is_some());
        assert!(!registry.has("Test Font"));

        // Remove non-existent returns None
        let removed = registry.remove("Test Font");
        assert!(removed.is_none());
    }

    #[test]
    fn test_clear() {
        let registry = FontRegistry::new();
        registry.register(Arc::new(TestFont::new("Font 1")));
        registry.register(Arc::new(TestFont::new("Font 2")));

        assert_eq!(registry.len(), 2);

        registry.clear();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_replaces_existing() {
        let registry = FontRegistry::new();

        registry.register(Arc::new(TestFont::new("Same Name")));
        registry.register(Arc::new(TestFont::new("Same Name")));

        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let registry = Arc::new(FontRegistry::new());
        let mut handles = vec![];

        // Spawn multiple threads registering fonts
        for i in 0..4 {
            let r = Arc::clone(&registry);
            handles.push(thread::spawn(move || {
                r.register(Arc::new(TestFont::new(&format!("Font {}", i))));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(registry.len(), 4);
    }

    #[test]
    fn test_get_with_asref_str() {
        let registry = FontRegistry::new();
        registry.register(Arc::new(TestFont::new("Test Font")));

        // Works with &str
        assert!(registry.get("Test Font").is_some());

        // Works with String
        assert!(registry.get(String::from("Test Font")).is_some());

        // Works with &String
        let name = String::from("Test Font");
        assert!(registry.get(&name).is_some());
    }
}
