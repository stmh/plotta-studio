---
# plotta-studio-h5l1
title: Add FontManager for centralized font registry
status: draft
type: feature
created_at: 2025-12-26T12:33:29Z
updated_at: 2025-12-26T12:33:29Z
parent: plotta-studio-ah5h
---

Create a FontManager struct for centralized font management in the drawing-text crate.

## Requirements

1. **Font Registry**
   - Store fonts by name in a HashMap<String, Box<dyn Font>>
   - Support registering fonts from any source (file, embedded, etc.)
   - Provide font lookup by name

2. **Font Discovery**
   - Scan a directory for supported font files (.jhf, .vsf, .svg)
   - Auto-register discovered fonts with their file names
   - Support multiple font directories

3. **Built-in Fonts**
   - Provide shortcuts for loading bundled fonts (Hershey Simplex)
   - Lazy loading to avoid startup overhead

4. **Integration**
   - Consider integration with RenderContext
   - Thread-safe access (Send + Sync)

## API Design

```rust
pub struct FontManager {
    fonts: HashMap<String, Box<dyn Font>>,
}

impl FontManager {
    pub fn new() -> Self;
    
    /// Register a font with a name
    pub fn register(&mut self, name: impl Into<String>, font: Box<dyn Font>);
    
    /// Get a font by name
    pub fn get(&self, name: &str) -> Option<&dyn Font>;
    
    /// Check if font exists
    pub fn has(&self, name: &str) -> bool;
    
    /// List all registered font names
    pub fn list(&self) -> Vec<&str>;
    
    /// Load all fonts from a directory
    pub fn load_directory(&mut self, path: impl AsRef<Path>) -> Result<usize, FontError>;
    
    /// Load built-in Hershey Simplex font
    pub fn with_hershey_simplex(self) -> Self;
}
```

## Checklist

- [ ] Create FontManager struct in drawing-text
- [ ] Implement font registration and lookup
- [ ] Add directory scanning for font discovery
- [ ] Add built-in font loading helpers
- [ ] Write unit tests
- [ ] Update documentation