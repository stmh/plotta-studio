---
# plotta-studio-bz5z
title: Add randomness option to hatch algorithm
status: completed
type: feature
priority: normal
created_at: 2026-01-01T16:44:19Z
updated_at: 2026-01-01T16:46:46Z
---

Extend the hatch line generation algorithm to support optional randomness. This could include random offsets to line positions, random variations in angle, or jitter in line endpoints.

## Checklist
- [x] Add randomness fields to HatchOptions
- [x] Implement random offset for line positions
- [x] Implement random offset for line endpoints
- [x] Update generate_hatch_lines to use randomness
- [x] Add tests for randomness options
- [x] Update sketch-011 to demonstrate randomness