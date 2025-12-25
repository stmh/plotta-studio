//! Render context for flattening operations
//!
//! The `RenderContext` holds shared resources needed during rendering,
//! such as font registries for text rendering.

use std::collections::HashMap;

use crate::font_types::Font;

/// Context for rendering operations
///
/// Holds shared resources like fonts that are needed when flattening
/// elements to strokes.
#[derive(Default)]
pub struct RenderContext {
    fonts: HashMap<String, Box<dyn Font>>,
}

impl RenderContext {
    /// Create a new empty render context
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a font with a name
    pub fn register_font(&mut self, name: impl Into<String>, font: Box<dyn Font>) {
        self.fonts.insert(name.into(), font);
    }

    /// Register a font, builder style
    pub fn with_font(mut self, name: impl Into<String>, font: Box<dyn Font>) -> Self {
        self.register_font(name, font);
        self
    }

    /// Get a font by name
    pub fn font(&self, name: &str) -> Option<&dyn Font> {
        self.fonts.get(name).map(|f| f.as_ref())
    }

    /// Check if a font is registered
    pub fn has_font(&self, name: &str) -> bool {
        self.fonts.contains_key(name)
    }

    /// Get list of registered font names
    pub fn font_names(&self) -> Vec<&str> {
        self.fonts.keys().map(|s| s.as_str()).collect()
    }
}

impl std::fmt::Debug for RenderContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderContext")
            .field("fonts", &self.font_names())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_context_new() {
        let ctx = RenderContext::new();
        assert!(ctx.font_names().is_empty());
    }

    #[test]
    fn test_render_context_font_not_found() {
        let ctx = RenderContext::new();
        assert!(ctx.font("NonExistent").is_none());
        assert!(!ctx.has_font("NonExistent"));
    }
}
