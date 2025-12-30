---
# plotta-studio-uweh
title: Investigate harsh motor noise when plotting curves
status: completed
type: task
priority: normal
created_at: 2025-12-29T17:39:11Z
updated_at: 2025-12-29T18:06:26Z
---

The plotter makes heavy/harsh noises when plotting curves, but the Python AxiDraw driver doesn't have this issue. Need to research what the Python implementation does differently for smoother motion.

## Root Cause Analysis

Our Rust implementation uses the **XM command** for motion:
```
XM,<duration>,<stepsX>,<stepsY>
```

This is a simple timed move - the motors accelerate/decelerate abruptly at the start and end of each segment. When plotting curves (many short segments), this creates harsh motor noise from constant acceleration/deceleration changes.

The Python driver uses more sophisticated motion commands:

1. **LM command** (Low-level Move) - Allows specifying rate and acceleration:
   ```
   LM,<Rate1>,<Steps1>,<Accel1>,<Rate2>,<Steps2>,<Accel2>[,Clear]
   ```
   This enables smoother trapezoidal velocity profiles.

2. **T3 command** (Third-order motion, firmware 3.0+) - Adds jerk control:
   ```
   T3,<Intervals>,<Rate1>,<Accel1>,<Jerk1>,<Rate2>,<Accel2>,<Jerk2>
   ```
   This provides S-curve acceleration for the smoothest motion.

## Key Differences

| Feature | XM (current) | LM | T3 |
|---------|--------------|----|----|
| Velocity profile | Square | Trapezoidal | S-curve |
| Acceleration control | None | Constant | With jerk |
| Motor noise | Harsh | Moderate | Smooth |
| Firmware required | 2.0+ | 2.7+ | 3.0+ |

## Possible Solutions

1. **Switch to LM command** - Requires implementing motion planning with acceleration
2. **Switch to T3 command** - Smoothest, but requires firmware 3.0+
3. **Segment coalescing** - Combine short segments into longer ones (lossy)
4. **Velocity lookahead** - Plan velocity to maintain smooth motion through curves

## Technical Details from EBB Documentation

### XM Command (current implementation)
```
XM,<Duration>,<AxisStepsA>,<AxisStepsB>
```
- Simple timed move with constant velocity
- Motors accelerate/decelerate abruptly at segment boundaries
- Duration is in milliseconds

### LM Command (improved motion)
```
LM,<Rate1>,<Steps1>,<Accel1>,<Rate2>,<Steps2>,<Accel2>[,Clear]
```
- Rate: step rate added to accumulator every 40us (Rate = 85899.35 * frequency_hz)
- Accel: added to Rate every 40us for acceleration control
- Enables trapezoidal velocity profiles
- Requires firmware 2.7.0+

### T3 Command (smoothest motion)
```
T3,<Intervals>,<Rate1>,<Accel1>,<Jerk1>,<Rate2>,<Accel2>,<Jerk2>[,Clear]
```
- Adds Jerk parameter for S-curve acceleration
- Smoothest possible motion
- Requires firmware 3.0+

## Implementation Path

**Target: Firmware 2.x (use LM command, not T3)**

### Phase 1: Implement LM command support
1. Add `move_to_lm()` method using LM command instead of XM
2. Calculate Rate parameter: `Rate = 85899.35 * steps_per_second`
3. For constant velocity (no accel), set `Accel = 0`
4. This alone should help because LM has better timing precision

### Phase 2: Add acceleration/deceleration
1. Implement trapezoidal velocity profile
2. Accelerate at start of segment, decelerate at end
3. Reduces abrupt speed changes between segments

### Phase 3: Velocity lookahead (advanced)
1. Look ahead at upcoming segments
2. Calculate corner velocities based on direction change
3. Maintain velocity through gentle curves
4. Only slow down for sharp corners

### Quick test first
Let's try switching from XM to LM with constant velocity first and see if it helps.

## Implementation Done

Switched `move_to()` from XM to LM command:

```rust
// Old (XM): XM,<duration_ms>,<steps_x>,<steps_y>
// New (LM): LM,<rate1>,<steps1>,0,<rate2>,<steps2>,0
```

Key changes:
- LM uses 40μs timing intervals (25kHz) vs XM's millisecond timing
- Rate = 85899.35 * steps_per_second (always positive)
- Steps carry the sign for direction (firmware 2.x requirement - negative Rate not supported!)
- Manually apply CoreXY transform (axis1 = x+y, axis2 = x-y)
- Accel = 0 for constant velocity (Phase 1)

## Testing Results

**Finding:** LM with Accel=0 sounds the same as XM - both are constant velocity moves.

The harsh sound comes from **sudden velocity changes between segments**, not from timing precision.
To fix this, we need to implement proper motion planning with acceleration.

## Additional Testing

Tested reducing curve segment count (fewer points = longer segments). Result: **no significant improvement**.

This confirms the issue is not segment length but the **lack of acceleration control**. Even with longer segments, constant-velocity motion creates harsh sounds at direction changes.

## Conclusion

The harsh motor noise is caused by:
1. Constant velocity motion (Accel=0) creating sudden direction changes
2. No velocity planning across segment boundaries

**Solution required:** Implement proper motion planning with:
1. **Trapezoidal velocity profiles** - Accelerate at start, constant in middle, decelerate at end
2. **Velocity lookahead** - Plan corner velocities based on path geometry  
3. **Non-zero Accel values** in LM commands

This is a significant feature that requires:
- Motion planner module
- Velocity/acceleration calculations per segment
- Cornering velocity based on direction change angle
- Proper use of LM's Accel parameter

This should be a separate epic/feature bean.

## References

- EBB LM command: http://evil-mad.github.io/EggBot/ebb.html#LM
- EBB T3 command: http://evil-mad.github.io/EggBot/ebb.html#T3  
- plotink ebb_motion.py: doLowLevelMove(), move_dist_lt()
- plotink ebb_calc.py: calculate_lm(), move_dist_t3()