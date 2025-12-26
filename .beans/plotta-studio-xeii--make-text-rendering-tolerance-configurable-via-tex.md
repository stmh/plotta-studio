---
# plotta-studio-xeii
title: Make text rendering tolerance configurable via TextOptions
status: completed
type: task
priority: normal
created_at: 2025-12-26T15:18:13Z
updated_at: 2025-12-26T15:19:58Z
---

The tolerance value in text.rs:96 is hardcoded to 0.5. This should be configurable via TextOptions for users who want finer or coarser curves when rendering text to strokes.