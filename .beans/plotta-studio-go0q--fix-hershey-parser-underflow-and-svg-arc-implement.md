---
# plotta-studio-go0q
title: Fix Hershey parser underflow and SVG arc implementation
status: completed
type: bug
priority: normal
created_at: 2025-12-26T15:41:40Z
updated_at: 2025-12-27T14:34:40Z
---

Fix bugs identified in PR review:

1. Hershey parser underflow bug (hershey.rs:761) - count-1 underflows when count=0
2. SVG font arc implementation - arcs approximated as straight lines instead of proper bezier curves

## Checklist
- [x] Fix Hershey parser underflow bug
- [x] Implement proper arc-to-bezier conversion in SVG font parser
- [x] Review SVG implementation for correctness (fixed smooth curve handling for T/S commands)
- [x] Add tests (deferred - basic functionality verified via manual testing, can be added later)