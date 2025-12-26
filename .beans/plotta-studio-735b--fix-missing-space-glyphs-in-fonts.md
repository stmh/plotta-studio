---
# plotta-studio-735b
title: Fix missing space glyphs in fonts
status: completed
type: bug
priority: normal
created_at: 2025-12-26T15:27:51Z
updated_at: 2025-12-26T15:32:53Z
---

Fixed missing space glyphs by adding fallback space handling in TextRenderer.layout().

When a font doesn't have a space glyph, the layout now uses a fallback width of 1/3 em. This fixes:
- Asteroids VSF font (actually has space, was working)
- Hershey Script Complex (now uses fallback)
- minf VSF font (now uses fallback)

The Gothic German 'k' glyph issue was investigated - the glyph data is correctly parsed (13 contours, multiple segments). The visual issue may be in the original Hershey font data itself.