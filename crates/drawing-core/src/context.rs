//! Render context for flattening operations
//!
//! The `RenderContext` holds shared resources needed during rendering,
//! such as font registries for text rendering.

use std::sync::Arc;

use crate::font_registry::{FontRef, FontRegistry};

/// Default curve flattening tolerance in mm
pub const DEFAULT_TOLERANCE: f64 = 0.05;

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
    /// Curve flattening tolerance in mm (default: 0.05)
    pub tolerance: f64,
}

impl RenderContext {
    /// Create a new render context with the given font registry.
    pub fn new(font_registry: Arc<FontRegistry>) -> Self {
        Self {
            font_registry,
            tolerance: DEFAULT_TOLERANCE,
        }
    }

    /// Create a render context with an empty font registry.
    pub fn empty() -> Self {
        Self {
            font_registry: Arc::new(FontRegistry::new()),
            tolerance: DEFAULT_TOLERANCE,
        }
    }

    /// Set the curve flattening tolerance in mm.
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
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
            .field("tolerance", &self.tolerance)
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

    #[test]
    fn test_render_context_default_tolerance() {
        let registry = Arc::new(FontRegistry::new());
        let ctx = RenderContext::new(registry);
        assert_eq!(ctx.tolerance, DEFAULT_TOLERANCE);
    }

    #[test]
    fn test_render_context_with_tolerance() {
        let registry = Arc::new(FontRegistry::new());
        let ctx = RenderContext::new(registry).with_tolerance(0.1);
        assert_eq!(ctx.tolerance, 0.1);
    }

    #[test]
    fn test_render_context_empty() {
        let ctx = RenderContext::empty();
        assert!(ctx.font_names().is_empty());
        assert_eq!(ctx.tolerance, DEFAULT_TOLERANCE);
    }
}
