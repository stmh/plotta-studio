---
# plotta-studio-h5l1
title: Add FontManager for centralized font registry
status: completed
type: feature
priority: normal
created_at: 2025-12-26T12:33:29Z
updated_at: 2025-12-26T14:23:47Z
parent: plotta-studio-ah5h
---

Create a FontManager struct for centralized font management in the drawing-text crate.

## Goals

1. **Sketch convenience** (primary) - Make it easy for sketches to list/cycle through available fonts without manual enum boilerplate
2. **Centralized caching** - Load fonts once and share them across the application
3. **Runtime font discovery** - Scan directories to find and load fonts dynamically

## Design Decisions

- **Lazy loading for Hershey fonts only** - All 8+ Hershey variants are "known" but only loaded when first accessed. VSF/SVG/UFO fonts must be explicitly registered.
- **Arc-based storage** - `Arc<dyn Font + Send + Sync>` for thread-safe sharing without lifetime complexity
- **Explicit format selection** - User passes `FontFormat` enum when loading files/directories (no extension guessing)
- **Filename-based naming** - Files registered with filename stem (e.g., `apple410.vsf` -> `"apple410"`)
- **Fail-fast on errors** - `load_directory()` stops on first parse error
- **Stateless cycling** - FontManager provides `list()`, sketches manage their own index/cycling logic

## API

```rust
use std::sync::{Arc, RwLock};
use std::path::Path;
use std::collections::HashMap;

pub struct FontManager {
    fonts: RwLock<HashMap<String, Arc<dyn Font + Send + Sync>>>,
}

impl FontManager {
    /// Create new manager (built-in Hershey fonts known but not loaded)
    pub fn new() -> Self;
    
    /// Get font by name (lazy-loads built-in Hershey fonts)
    pub fn get(&self, name: &str) -> Option<Arc<dyn Font + Send + Sync>>;
    
    /// Check if font exists (includes unloaded built-ins)
    pub fn has(&self, name: &str) -> bool;
    
    /// List all available font names (includes unloaded built-ins)
    pub fn list(&self) -> Vec<String>;
    
    /// Register a font manually
    pub fn register(&self, name: impl Into<String>, font: impl Font + Send + Sync + 'static);
    
    /// Load a single font file, returns registered name (filename stem)
    pub fn load_file(&self, path: impl AsRef<Path>, format: FontFormat) -> Result<String, FontError>;
    
    /// Load all fonts from directory, returns count loaded (fails fast)
    pub fn load_directory(&self, path: impl AsRef<Path>, format: FontFormat) -> Result<usize, FontError>;
}
```

## Built-in Hershey Fonts (lazy-loaded)

These are "known" at construction and loaded on first access:

- Hershey Simplex
- Hershey Duplex  
- Hershey Triplex
- Hershey Script Simplex
- Hershey Script Complex
- Hershey Gothic German Bold
- Hershey Gothic German
- Hershey Gothic Italian

## Checklist

- [x] Create FontManager struct with RwLock<HashMap<String, Arc<...>>>
- [x] Implement lazy-loading registry for built-in Hershey fonts
- [x] Implement `get()`, `has()`, `list()` methods
- [x] Implement `register()` for manual font registration
- [x] Implement `load_file()` with FontFormat parameter
- [x] Implement `load_directory()` with fail-fast behavior
- [x] Add unit tests for all methods
- [x] Update sketch-003-text to use FontManager (demo integration)
