---
# plotta-studio-svg5
title: Add SVG import tests
status: completed
type: task
priority: normal
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-27T18:06:30Z
parent: plotta-studio-svg1
---

Add comprehensive tests for SVG import functionality.

## Test Cases

```rust
#[cfg(test)]
mod import_tests {
    use super::*;

    #[test]
    fn test_import_simple_line() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <line x1="0" y1="0" x2="100" y2="100" stroke="black"/>
        </svg>"#;
        let drawing = import_svg_string(svg).unwrap();
        assert_eq!(drawing.width, 100.0);
        assert_eq!(drawing.elements.len(), 1);
    }

    #[test]
    fn test_import_circle() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <circle cx="50" cy="50" r="25" stroke="red" fill="none"/>
        </svg>"#;
        let drawing = import_svg_string(svg).unwrap();
        assert_eq!(drawing.elements.len(), 1);
    }

    #[test]
    fn test_import_path() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M10,10 L90,10 L90,90 L10,90 Z" stroke="black" fill="none"/>
        </svg>"#;
        let drawing = import_svg_string(svg).unwrap();
        assert!(!drawing.elements.is_empty());
    }

    #[test]
    fn test_import_with_transforms() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <g transform="translate(10, 10)">
                <rect x="0" y="0" width="20" height="20" stroke="black" fill="none"/>
            </g>
        </svg>"#;
        let drawing = import_svg_string(svg).unwrap();
        // Verify transform was applied
    }

    #[test]
    fn test_roundtrip() {
        // Create drawing, export to SVG, import back, compare
        let mut original = Drawing::new(100.0, 100.0);
        original.add(Element::circle((50.0, 50.0), 25.0));

        let svg = drawing_to_svg_string(&original);
        let imported = import_svg_string(&svg).unwrap();

        assert_eq!(original.width, imported.width);
        assert_eq!(original.height, imported.height);
    }
}
```

## Files to Modify
- `crates/drawing-svg/src/lib.rs`
