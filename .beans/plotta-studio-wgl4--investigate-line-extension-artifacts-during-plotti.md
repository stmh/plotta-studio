---
# plotta-studio-wgl4
title: Investigate line extension artifacts during plotting
status: completed
type: bug
priority: normal
created_at: 2025-12-30T19:25:38Z
updated_at: 2025-12-31T13:48:23Z
parent: plotta-studio-q6v8
---

When plotting sketch-006-hamburg, lines have visible 'extensions' at the beginning. This happens in both directions (reversed and non-reversed strokes). The plotter also makes a clunky sound during these artifacts.

## Symptoms
- Lines have a small extension/jitter at the start
- Clunky motor sound
- Affects both reversed and non-reversed strokes
- Likely related to motion planning / acceleration

## Hypothesis
The motion planner starts strokes from velocity 0, generating LM commands with rate=0 and non-zero acceleration. The EBB firmware might have issues starting motion from rate=0, causing the motor to jerk.

## Investigation needed
- Check if the LM command with rate=0, accel>0 is causing issues
- Consider adding a minimum starting velocity
- Add debug logging to see actual LM commands being sent
- Test with motion planning disabled to confirm it's the cause

## Findings (2025-12-30)

### LM Commands Generated
When starting a stroke from velocity 0 (which happens at the beginning of every stroke), we generate commands like:
```
LM,0,50,137439,0,50,137439  <- acceleration phase (rate=0, accel>0)
LM,171798692,700,0,...      <- cruise phase
LM,171798692,50,-137439,... <- deceleration phase
```

### EBB Documentation Analysis
The EBB docs confirm that `rate=0, accel>0` is valid:
> "motion must be possible on at least one axis... you must ensure that both Steps is nonzero and that either Rate or Accel are nonzero"

However, all the examples in the documentation start with non-zero rate (e.g., "45 steps/s... Rate is 3865471"). They never show starting from rate=0.

### Python AxiDraw Implementation
Looking at `plotink/ebb_calc.py`, there's special handling for the accumulator when rate=0:
```python
if accum == "clear":
    accum = 0
    temp_rate = rate - int(accel / 2) + accel
    if temp_rate < 0:
        accum = 2147483647  # Clear to 2^31 - 1
    elif temp_rate == 0:  # Special case, if rate==0 during first step
        if accel < 0:
            accum = 2147483647
```

This suggests the accumulator initialization is important when starting from rate=0.

### Potential Causes
1. **Accumulator not cleared**: We may need to pass `Clear=3` to clear both accumulators at the start of each stroke
2. **Minimum velocity**: The Python implementation may use a minimum starting velocity to avoid rate=0
3. **Timing/duration issues**: The calculated duration for the acceleration phase may not match the expected step count

## Possible Solutions
1. Add `Clear=3` to the first LM command of each stroke
2. Use a minimum starting velocity (e.g., 1 mm/s) instead of 0
3. Review the Python axidraw implementation more closely to see how they handle stroke starts

## Fix Implemented (2025-12-30)

Added `Clear=3` to the first LM command when starting from velocity 0.

### Changes Made:
1. Added `clear: u8` field to `LmCommand` struct (0=none, 1=axis1, 2=axis2, 3=both)
2. Updated `LmCommand::to_command_string()` to append Clear parameter when non-zero
3. Modified `PlannedMove::with_profile()` to set `clear=3` on the first command when `entry_velocity < 1e-9`

### Commands now look like:
```
LM,0,50,137439,0,50,137439,3  <- acceleration phase with Clear=3
LM,171798692,700,0,171798692,700,0
LM,171798692,50,-137439,171798692,50,-137439
```

### Testing Status:
- [x] Unit tests pass
- [ ] Hardware test needed to verify fix

The fix needs to be tested on actual hardware to confirm it resolves the line extension artifacts.