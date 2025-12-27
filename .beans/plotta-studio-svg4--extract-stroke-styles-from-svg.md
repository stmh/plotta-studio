---
# plotta-studio-svg4
title: Extract stroke styles from SVG
status: completed
type: task
priority: normal
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-27T18:05:50Z
parent: plotta-studio-svg1
---

Extract stroke width and color from SVG elements and apply to drawing-core Style.

## Implementation Details

usvg provides stroke information via the `stroke` property:

```rust
fn extract_style(stroke: &Option<usvg::Stroke>) -> Style {
    match stroke {
        Some(s) => {
            let color = match &s.paint {
                usvg::Paint::Color(c) => Color::rgb(c.red, c.green, c.blue),
                _ => Color::BLACK, // Fallback for gradients, etc.
            };
            Style {
                stroke_width: s.width.get() as f64,
                stroke_color: color,
            }
        }
        None => Style::default(),
    }
}
```

## Considerations
- SVG stroke-width is affected by transforms - need to decide if we scale it
- Opacity: SVG has stroke-opacity, should map to Color alpha
- Stroke linecap/linejoin: Currently not in drawing-core Style, could be added later
- Dashed strokes: Not currently supported, would need extension

## Files to Modify
- `crates/drawing-svg/src/lib.rs`
