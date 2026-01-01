---
# plotta-studio-sp87
title: Add invert option to clip functionality
status: completed
type: feature
priority: normal
created_at: 2026-01-01T16:30:13Z
updated_at: 2026-01-01T16:33:53Z
---

Add an `invert` option to ClipGroup that inverts the clipping behavior. Currently clipping keeps content inside the clip region; with invert=true, it should keep content outside the clip region instead.

## Checklist
- [x] Add `invert` field to ClipGroup struct
- [x] Add `invert()` builder method to ClipGroup
- [x] Add `invert()` builder method to Element for clip groups
- [x] Modify clipping logic to support inverted clipping
- [x] Add test for rect clipping diagonal lines (bottom to top)
- [x] Add test for inverted clipping behavior
- [x] Run tests and ensure all pass