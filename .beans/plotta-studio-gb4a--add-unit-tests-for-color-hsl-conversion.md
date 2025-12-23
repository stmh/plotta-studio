---
# plotta-studio-gb4a
title: Add unit tests for Color HSL conversion
status: todo
type: task
created_at: 2025-12-23T18:35:50Z
updated_at: 2025-12-23T18:35:50Z
parent: plotta-studio-gfob
---

Test Color::hsl() in drawing-core. Cover all hue ranges (0-60, 60-120, etc.), saturation boundaries (0 = grayscale, 1 = full), lightness boundaries (0 = black, 0.5 = pure, 1 = white).