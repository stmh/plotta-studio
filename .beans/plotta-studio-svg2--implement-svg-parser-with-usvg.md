---
# plotta-studio-svg2
title: Implement SVG parser with usvg
status: completed
type: task
priority: normal
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-27T18:05:48Z
parent: plotta-studio-svg1
---

Add the usvg dependency and implement the core SVG parsing logic.

## Implementation Details

1. Add to `crates/drawing-svg/Cargo.toml`:
```toml
usvg = "0.44"
```

2. Create parsing function in `lib.rs`:
```rust
use usvg::{Tree, Node, NodeKind};

pub fn import_svg_string(svg: &str) -> Result<Drawing, SvgError> {
    let opt = usvg::Options::default();
    let tree = Tree::from_str(svg, &opt)
        .map_err(|e| SvgError::Parse(e.to_string()))?;

    let size = tree.size();
    let mut drawing = Drawing::new(size.width() as f64, size.height() as f64);

    // Walk tree and convert nodes
    convert_node(&tree.root(), Transform::IDENTITY, &mut drawing)?;

    Ok(drawing)
}
```

3. Implement node conversion recursively handling:
   - Groups with transforms
   - Paths with all segment types
   - Basic shapes (rect, circle, ellipse, line)

## Files to Modify
- `crates/drawing-svg/Cargo.toml` - add usvg dependency
- `crates/drawing-svg/src/lib.rs` - add import functions
