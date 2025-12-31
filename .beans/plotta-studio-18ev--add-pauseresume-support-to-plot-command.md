---
# plotta-studio-18ev
title: Add pause/resume support to plot command
status: completed
type: feature
priority: normal
created_at: 2025-12-28T20:29:08Z
updated_at: 2025-12-31T13:55:10Z
parent: plotta-studio-7wzz
---

Add ability to pause plotting by pressing space bar. When paused, wait for another space press to resume.

## Requirements
- Press space to pause after current stroke completes (pen up)
- Press space again to resume
- Show pause status in progress bar or console
- Handle terminal raw mode for key detection

## Implementation
- Use crossterm crate for terminal input handling
- Check for key press between strokes in the plot event loop
- When paused: display 'PAUSED - press space to resume'

## Checklist
- [x] Add crossterm dependency for terminal input
- [x] Implement non-blocking key detection
- [x] Add pause check between strokes in plot command
- [x] Show pause/resume status to user
- [ ] Test with actual plotting