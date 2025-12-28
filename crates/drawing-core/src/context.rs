//! Render context for flattening operations
//!
//! The `RenderContext` holds shared resources needed during rendering,
//! such as font registries for text rendering.

use std::sync::Arc;

use crate::font_registry::{FontRef, FontRegistry};

/// Context for rendering operations
///
/// Holds shared resources like fonts that are needed when flattening
/// elements to strokes.
///
/// # Example
///
/// ```rust,ignore
/// use drawing_core::{FontRegistry, RenderContext};
/// use std::sync::Arc;
///
/// let registry = Arc::new(FontRegistry::new());
/// // ... register fonts with registry ...
///
/// let ctx = RenderContext::new(registry);
/// let font = ctx.font("Hershey Simplex");
/// ```
pub struct RenderContext {
    font_registry: Arc<FontRegistry>,
}

impl RenderContext {
    /// Create a new render context with the given font registry.
    pub fn new(font_registry: Arc<FontRegistry>) -> Self {
        Self { font_registry }
    }

    /// Create a render context with an empty font registry.
    ///
    /// Useful for CLI tools or other contexts where fonts are not needed
    /// (e.g., when drawings contain only geometric primitives).
    pub fn empty() -> Self {
        Self {
            font_registry: Arc::new(FontRegistry::new()),
        }
    }

    /// Get the font registry.
    pub fn font_registry(&self) -> &Arc<FontRegistry> {
        &self.font_registry
    }

    /// Get a font by name (convenience method).
    ///
    /// This is equivalent to `ctx.font_registry().get(name)`.
    pub fn font(&self, name: impl AsRef<str>) -> Option<FontRef> {
        self.font_registry.get(name)
    }

    /// Check if a font is registered (convenience method).
    pub fn has_font(&self, name: impl AsRef<str>) -> bool {
        self.font_registry.has(name)
    }

    /// Get list of registered font names (convenience method).
    pub fn font_names(&self) -> Vec<String> {
        self.font_registry.list()
    }
}

impl std::fmt::Debug for RenderContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderContext")
            .field("font_registry", &self.font_registry)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_context_new() {
        let registry = Arc::new(FontRegistry::new());
        let ctx = RenderContext::new(registry);
        assert!(ctx.font_names().is_empty());
    }

    #[test]
    fn test_render_context_font_not_found() {
        let registry = Arc::new(FontRegistry::new());
        let ctx = RenderContext::new(registry);
        assert!(ctx.font("NonExistent").is_none());
        assert!(!ctx.has_font("NonExistent"));
    }

    #[test]
    fn test_render_context_font_registry_access() {
        let registry = Arc::new(FontRegistry::new());
        let ctx = RenderContext::new(registry.clone());

        // Both should reference the same registry
        assert!(Arc::ptr_eq(ctx.font_registry(), &registry));
    }
}
