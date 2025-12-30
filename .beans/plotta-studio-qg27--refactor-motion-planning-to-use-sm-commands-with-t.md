---
# plotta-studio-qg27
title: Refactor motion planning to use SM commands with time-slice interpolation
status: completed
type: task
priority: high
created_at: 2025-12-30T20:36:30Z
updated_at: 2025-12-30T21:06:28Z
parent: plotta-studio-q6v8
---

Replace LM-based acceleration with SM-based time-slice motion planning to fix jitter at line start/end and poor curve drawing.

## Problem

Our current LM-based motion planning causes jitter:
1. LM command with acceleration is complex and error-prone (timing, accumulator issues)
2. Starting from velocity 0 with LM causes artifacts
3. No overspeed/underspeed guards
4. Command buffering doesn't leave lead time

## Solution

Match the AxiDraw Python driver approach:
1. Use SM (constant-velocity) commands instead of LM with acceleration
2. Break motion into 25ms time slices with velocity interpolation
3. Software computes trapezoid profile, generates many small SM commands
4. Add 30ms command buffer lead time

## Technical Analysis

See `docs/axidraw-motion-planning-analysis.md` for full analysis of Python driver.

Key insight: Python driver **never** uses LM acceleration for normal plotting - it uses SM commands exclusively with software-based velocity planning.

## Checklist

### Phase 1: SM Command Infrastructure
- [x] Create `SmCommand` struct with `to_command_string()`
- [x] Add `execute_sm_command()` method with 30ms buffer lead time
- [x] Add min/max step rate constants for guards

### Phase 2: Time-Slice Motion Generation
- [x] Create `generate_sm_commands()` function
- [x] Implement 25ms time-slice velocity interpolation
- [x] Handle accel/cruise/decel phases with SM commands
- [x] Add overspeed guard (increase duration if rate too high)
- [x] Add underspeed guard (zero out steps if rate too low)

### Phase 3: Integration
- [x] Replace `execute_lm_command()` calls with SM-based execution
- [x] Update `draw_stroke_with_planning()` to use new SM generation
- [x] Keep LM command code as legacy (marked with `#[allow(dead_code)]`)
- [x] Create `SmPlannedMove` to use `Vec<SmCommand>`

### Phase 4: Parameter Tuning
- [x] Match Python acceleration defaults (~1016 mm/s² = 40 in/s²)
- [x] Set time_slice to 25ms
- [x] Tune cornering parameters to match Python (junction_deviation = 0.05mm)

### Phase 5: Testing
- [x] Unit tests for SM command generation
- [x] Unit tests for time-slice interpolation
- [ ] Hardware test: straight lines (verify no start/end jitter)
- [ ] Hardware test: curves (verify smooth motion)
- [ ] Hardware test: sharp corners (verify proper deceleration)

## References
- `docs/axidraw-motion-planning-analysis.md` - Full Python driver analysis
- Python `motion.py:616-917` - compute_segment() with time-slice generation
- Python `dripfeed.py:125-130` - 30ms buffer strategy
