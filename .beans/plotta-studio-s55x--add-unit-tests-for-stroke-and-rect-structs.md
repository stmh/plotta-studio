---
# plotta-studio-s55x
title: Add unit tests for Stroke and Rect structs
status: todo
type: task
created_at: 2025-12-23T18:35:54Z
updated_at: 2025-12-23T18:35:54Z
parent: plotta-studio-gfob
---

Test Stroke::bounds(), Stroke::length(), Rect::centered(), Rect::corners(), Rect::center() in drawing-core. Cover edge cases: empty points, single point, zero dimensions, negative coordinates.