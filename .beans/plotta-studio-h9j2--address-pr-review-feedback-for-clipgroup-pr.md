---
# plotta-studio-h9j2
title: Address PR review feedback for ClipGroup PR
status: completed
type: task
priority: normal
created_at: 2025-12-27T16:03:59Z
updated_at: 2025-12-27T16:06:11Z
---

Implement suggestions from PR #8 review:

## High Priority
1. Fix numerical stability in line clipping (clip.rs:198-203)
2. Handle NaN in sort_by with unwrap_or (clip.rs:222)
3. Verify ClipGroup is in Shape enum

## Medium Priority
4. Add complexity comment to union_polygons
5. Add Debug to FrameOptions (can't - FontRef doesn't impl Debug)
6. Improve font not found warning

## Low Priority
7. Add comment for clippy allow
8. Extract hatch extent multiplier constant
9. Use i64 for line_key (note: this is in sketch code, not library)

## Checklist
- [x] Fix numerical stability in t calculation
- [x] Handle NaN in sort_by
- [x] Verify ClipGroup in Shape enum
- [x] Add complexity comment to union_polygons
- [x] Improve font not found warning
- [x] Add comment for clippy allow
- [x] Extract HATCH_EXTENT_MULTIPLIER constant