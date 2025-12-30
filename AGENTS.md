# Agents Guide for plotta-studio

This document provides guidelines for AI coding agents working in this Rust workspace.

## Project Overview

Plotta-studio is a Rust workspace for generative drawings and pen plotter output. It consists of:
- `crates/drawing-core` - Core primitives, transforms, scene graph
- `crates/drawing-text` - Single-line font support (Hershey, VSF, SVG fonts)
- `crates/drawing-svg` - SVG import/export
- `crates/drawing-plotter` - AxiDraw plotter control
- `crates/sketch-runner` - Window, rendering, input handling
- `sketches/*` - Example sketches

## Build Commands

```bash
# Build entire workspace
cargo build

# Build specific package
cargo build -p drawing-core

# Build with all features
cargo build --all-features

# Check without building (faster)
cargo check --all-targets
```

## Test Commands

```bash
# Run all tests
cargo test

# Run tests for a specific package
cargo test -p drawing-text

# Run a single test by name (substring match)
cargo test test_load_hershey

# Run a single test in a specific package
cargo test -p drawing-text test_load_hershey

# Run tests with output shown
cargo test -- --nocapture

# Run only doc tests
cargo test --doc
```

## Lint Commands

```bash
# Format code (check only)
cargo fmt --all -- --check

# Format code (apply changes)
cargo fmt --all

# Run clippy (must pass with no warnings in CI)
cargo clippy --all-targets -- -D warnings
```

## Code Style Guidelines

### Imports

Order imports in groups separated by blank lines:
1. Standard library (`std::`)
2. External crates
3. Workspace crates (`drawing_core::`, `drawing_text::`)
4. Local modules (`crate::`, `super::`)

```rust
use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use drawing_core::{Point, Style};

use crate::error::FontError;
use crate::types::Glyph;
```

### Naming Conventions

- Types: `PascalCase` (`FontManager`, `TextRenderer`)
- Functions/methods: `snake_case` (`load_hershey`, `to_strokes`)
- Constants: `SCREAMING_SNAKE_CASE` (`DEFAULT_TOLERANCE`)
- Modules: `snake_case` (`font_types`, `drawing_core`)

### Error Handling

Use `thiserror` for library errors with descriptive messages:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FontError {
    #[error("Failed to parse font: {0}")]
    ParseError(String),

    #[error("I/O error reading {0}: {1}")]
    IoError(PathBuf, String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
```

Return `Result<T, Error>` from fallible functions. Use `?` for propagation.

### Builder Pattern

Use builder pattern with method chaining for configuration:

```rust
impl Style {
    pub fn with_stroke_width(mut self, w: f64) -> Self {
        self.stroke_width = w;
        self
    }

    pub fn with_stroke_color(mut self, c: Color) -> Self {
        self.stroke_color = c;
        self
    }
}

// Usage
let style = Style::default()
    .with_stroke_width(2.0)
    .with_stroke_color(Color::RED);
```

### Documentation

Add doc comments to all public items:

```rust
/// Font manager for loading fonts into a registry.
///
/// # Example
///
/// ```rust,ignore
/// let manager = FontManager::new();
/// manager.load_hershey(Hershey::Simplex)?;
/// ```
pub struct FontManager { ... }
```

### Module Structure

Each crate follows this pattern:
- `lib.rs` - Re-exports public API, module declarations
- `error.rs` - Error types (if needed)
- Feature modules - One per major feature

### Type Aliases

Use type aliases for clarity, especially for complex types:

```rust
pub type FontRef = Arc<dyn Font + Send + Sync>;
pub type Transform = kurbo::Affine;
```

### Derive Macros

Common derives for data types:
- `Debug` - Always include for debugging
- `Clone` - When copying is needed
- `Serialize, Deserialize` - For JSON/config types
- `PartialEq, Eq` - For comparison
- `Hash` - When used as HashMap keys
- `Default` - When a sensible default exists

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Style {
    pub stroke_width: f64,
    pub stroke_color: Color,
}
```

### Testing

Place tests in the same file using a `tests` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}
```

### Feature Flags

Use feature flags for optional dependencies:

```rust
// In Cargo.toml
[features]
default = ["hardware"]
hardware = ["serialport"]

// In code
#[cfg(feature = "hardware")]
mod serial_impl;
```

## Architecture Notes

- `kurbo` is used for 2D geometry (Point, Rect, BezPath, Affine)
- `vello` + `wgpu` for GPU rendering
- `winit` for windowing
- Coordinate system: origin top-left, Y increases downward, units in mm

## Task Tracking with Beans

This project uses **beans** for tracking plans and progress. Beans are stored in the `.beans/` directory.

### Creating Beans

When creating a new bean:
1. Always specify a type with `-t` (task, feature, bug, epic, milestone)
2. Include a useful description with `-d`
3. Set the appropriate parent relationship if the bean belongs to an epic or feature
4. Check for existing related beans to avoid duplicates

```bash
# Create a new task under an existing epic
beans create "Implement feature X" -t task -d "Description..." -s todo
beans update <new-bean-id> --parent <epic-id>

# Query beans to find related work
beans query '{ beans(filter: { search: "keyword" }) { id title status parentId } }'
```

### Maintaining Relationships

- **Parent-child**: Use `--parent` to organize tasks under epics/features
- **Blocking**: Use `--blocking` for dependencies between beans
- When completing work, check if parent beans need status updates
- Keep bean descriptions and checklists up-to-date as work progresses

### Bean Hierarchy

```
milestone
  └── epic
        └── feature
              └── task/bug
```

## Git Workflow

- Run `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` before committing
- Use conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`
- Use jj (Jujutsu) for version control, not git directly
- Create bookmarks for branches: `jj bookmark create feature-name`
- Include updated bean files in commits when work status changes
