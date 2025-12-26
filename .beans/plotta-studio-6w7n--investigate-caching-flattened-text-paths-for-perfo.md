---
# plotta-studio-6w7n
title: Investigate caching flattened text paths for performance
status: completed
type: task
priority: normal
created_at: 2025-12-26T15:35:38Z
updated_at: 2025-12-26T15:37:20Z
---

Investigated text rendering performance and caching opportunities.

## Findings

### Current Architecture
1. **sketch-runner already caches** flattened strokes at the Drawing level (`cached_strokes` + `strokes_dirty` flag)
2. Cache is invalidated when drawing changes, and `refresh_strokes()` recalculates only when needed
3. This is the right level for most use cases

### Cost Breakdown
1. Layout calculation - iterates text, applies kerning (moderate cost)
2. Glyph lookup - HashMap lookup per character (cheap)
3. **Contour flattening** - bezier to polyline via kurbo (expensive)
4. Transform application - simple math per point (cheap)

### Caching Options Considered

**Option A: Font-level glyph cache**
- Cache flattened contours per (char, tolerance)
- Pro: Shared across all text using same font
- Con: Requires interior mutability (RwLock), complicates serialization

**Option B: Text-level cache**
- Cache untransformed strokes in Text shape
- Pro: Simple invalidation
- Con: Requires mutable cache, breaks immutable design

**Option C: Drawing-level cache (current)**
- Already implemented in sketch-runner
- Pro: Simple, works for most cases
- Con: Entire drawing invalidated on any change

## Recommendation

The current architecture is sufficient for most use cases. The sketch-runner's Drawing-level caching handles the common case well.

For text-heavy applications that need more optimization, future options include:
1. Add `GlyphCache` struct that can be optionally passed to `to_strokes()`
2. Pre-flatten frequently used text into polyline elements
3. Use coarser tolerance for preview, finer for export

No code changes needed at this time - marking as completed.