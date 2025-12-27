---
# plotta-studio-izsm
title: Create drawing-utils crate with reusable utilities
status: completed
type: feature
priority: normal
created_at: 2025-12-27T14:25:15Z
updated_at: 2025-12-27T14:33:24Z
---

Create a new drawing-utils crate with reusable functionality:

## Planned utilities

1. **Hatching** - Generate parallel hatch lines for filling shapes
2. **Frame with title** - Draw a border frame with title text in lower left corner (using ReliefSingleLine font)

## Checklist

- [x] Create crate structure
- [x] Implement hatch line generation
- [x] Implement frame with title
- [x] Update sketch-004 to use the new utils
- [x] Add tests