---
# plotta-studio-345m
title: 'Refactor font system: split FontRegistry and FontManager'
status: completed
type: task
priority: normal
created_at: 2025-12-26T14:58:34Z
updated_at: 2025-12-26T15:09:22Z
parent: plotta-studio-ah5h
---

Refactor the font system based on PR review feedback to separate concerns and improve performance.

## Goals

1. **Separate concerns** - Split storage (FontRegistry) from loading logic (FontManager)
2. **Fix performance** - PositionedGlyph stores char instead of cloned Glyph
3. **Simplify API** - Font trait has name(), type-safe Hershey enum
4. **Address PR review feedback** - Resolve duplication between RenderContext and FontManager

## Design

### Module Structure

**drawing-core:**
- `FontRegistry` - Thread-safe storage: `RwLock<HashMap<String, FontRef>>`
- `FontRef` type alias for `Arc<dyn Font + Send + Sync>`
- `RenderContext` holds `Arc<FontRegistry>`
- Full collection API: get/register/has/list/remove/clear/len/is_empty

**drawing-text:**
- `FontManager` - Loading logic, holds `Arc<FontRegistry>`
- `Hershey` enum - Type-safe variants with `name()` and `AsRef<str>`
- `font_manager.load_hershey(Hershey::Simplex)` - Explicit loading
- File-based loading: `load_file()`, `load_directory()`

### Font Trait Changes

```rust
pub trait Font: Send + Sync {
    fn name(&self) -> &str;  // NEW: font knows its name
    fn glyph(&self, c: char) -> Option<Glyph>;
    fn metrics(&self) -> FontMetrics;
    fn kerning(&self, left: char, right: char) -> f64;
    fn has_glyph(&self, c: char) -> bool;
}

pub type FontRef = Arc<dyn Font + Send + Sync>;
```

### FontRegistry API (drawing-core)

```rust
pub struct FontRegistry {
    fonts: RwLock<HashMap<String, FontRef>>,
}

impl FontRegistry {
    pub fn new() -> Self;
    pub fn get(&self, name: impl AsRef<str>) -> Option<FontRef>;
    pub fn register(&self, font: FontRef);  // Uses font.name()
    pub fn has(&self, name: impl AsRef<str>) -> bool;
    pub fn list(&self) -> Vec<String>;
    pub fn remove(&self, name: impl AsRef<str>) -> Option<FontRef>;
    pub fn clear(&self);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### FontManager API (drawing-text)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hershey {
    Simplex, Duplex, Triplex,
    ScriptSimplex, ScriptComplex,
    GothicGermanBold, GothicGerman, GothicItalian,
}

impl Hershey {
    pub fn name(&self) -> &'static str;
}

impl AsRef<str> for Hershey {
    fn as_ref(&self) -> &str { self.name() }
}

pub struct FontManager {
    registry: Arc<FontRegistry>,
}

impl FontManager {
    pub fn new(registry: Arc<FontRegistry>) -> Self;
    pub fn registry(&self) -> &Arc<FontRegistry>;
    pub fn load_hershey(&self, variant: Hershey) -> Result<(), FontError>;
    pub fn load_file(&self, path: impl AsRef<Path>, format: FontFormat) -> Result<String, FontError>;
    pub fn load_directory(&self, path: impl AsRef<Path>, format: FontFormat) -> Result<usize, FontError>;
}
```

### Performance Fix: TextLayout & PositionedGlyph

```rust
pub struct PositionedGlyph {
    pub char: char,        // Lookup key instead of cloned glyph
    pub position: Point,
    pub scale: f64,
}

pub struct TextLayout {
    pub font: FontRef,     // Keeps font alive
    pub glyphs: Vec<PositionedGlyph>,
    pub bounds: Option<Rect>,
    pub line_count: usize,
}
```

### RenderContext Changes

```rust
pub struct RenderContext {
    font_registry: Arc<FontRegistry>,
}

impl RenderContext {
    pub fn new(font_registry: Arc<FontRegistry>) -> Self;
    pub fn font_registry(&self) -> &Arc<FontRegistry>;
    pub fn font(&self, name: impl AsRef<str>) -> Option<FontRef>;
}
```

### Usage Example

```rust
let registry = Arc::new(FontRegistry::new());
let manager = FontManager::new(registry.clone());
manager.load_hershey(Hershey::Simplex)?;

let ctx = RenderContext::new(registry);
let font = ctx.font(Hershey::Simplex).unwrap();  // Type-safe!
let layout = renderer.layout("Hello", font, &options);
```

## Checklist

- [x] Add `FontRef` type alias to drawing-core
- [x] Add `name()` method to Font trait
- [x] Create FontRegistry in drawing-core with full API
- [x] Update RenderContext to hold Arc<FontRegistry>
- [x] Add Hershey enum with name() and AsRef<str>
- [x] Refactor FontManager to use Arc<FontRegistry>
- [x] Update PositionedGlyph to store char instead of Glyph
- [x] Update TextLayout to store FontRef
- [x] Update TextRenderer::layout() to take FontRef
- [x] Update all Font implementations (Hershey, VSF, SVG) with name()
- [x] Update sketch-003-text for new API
- [x] Update tests
- [x] Remove old RenderContext font storage code