---
# plotta-studio-cemr
title: Add verbose logging to plotta-cli and drawing-plotter for debugging large files
status: completed
type: task
priority: normal
created_at: 2025-12-30T17:26:00Z
updated_at: 2025-12-30T17:29:06Z
---

Add more log messages throughout the plotting pipeline to help diagnose issues when plotting large files. This includes:
- File loading and parsing progress
- Stroke optimization progress  
- Motion planning details
- Serial communication timing
- Memory usage for large drawings

The CLI already has a --verbose flag, but needs more log statements.