---
# plotta-studio-5pdf
title: Implement smart pen delays for smoother plotting
status: completed
type: feature
priority: normal
created_at: 2025-12-28T17:18:54Z
updated_at: 2025-12-31T13:54:47Z
parent: plotta-studio-axi1
---

Reduce pen up/down delays by using smarter EBB commands instead of fixed sleep delays.

## Problem
Current implementation uses fixed 150ms delays after pen up/down commands, causing sluggish start/endpoints of strokes.

## Solution - NEEDS RESEARCH
Initial attempt using SC commands failed - SC,4 and SC,5 control servo **positions**, not rates!

Correct approach:
1. **SC,10,<rate>** - Set servo rate (higher = slower movement, lower = faster)
2. **S2,<pos>,<output>,<rate>,<delay>** - Set servo position with rate control

Need to study AxiDraw Python implementation more carefully.

## Status
**REVERTED** - SC,4/SC,5 commands corrupted servo positions, breaking pen-up.
Currently using safe defaults: 150ms delays, no servo rate configuration.

## Checklist
- [ ] Research correct EBB servo rate commands (SC,10, S2)
- [ ] Study AxiDraw Python implementation for servo configuration
- [ ] Implement servo rate configuration properly
- [ ] Add --pen-up-delay and --pen-down-delay CLI options (already done)
- [ ] Test on hardware to find optimal values

## References
- EBB Protocol: https://evil-mad.github.io/EggBot/ebb.html#SC
- AxiDraw Python: Uses servo_rate around 150-400