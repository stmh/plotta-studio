---
# plotta-studio-dot8
title: Fix LM command invalid parameter error during plotting
status: completed
type: bug
priority: normal
created_at: 2025-12-29T21:06:47Z
updated_at: 2025-12-29T21:09:41Z
parent: plotta-studio-q6v8
---

The plotter throws '!6 Err: Invalid parameter value' during plotting. This is likely caused by LM commands with invalid parameters (zero Rate when Steps is non-zero, or other edge cases in motion planning calculations). Need to validate LM command parameters before sending and handle edge cases properly.