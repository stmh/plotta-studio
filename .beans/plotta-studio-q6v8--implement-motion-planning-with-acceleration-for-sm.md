---
# plotta-studio-q6v8
title: Implement motion planning with acceleration for smooth plotting
status: in-progress
type: feature
priority: normal
created_at: 2025-12-29T18:06:40Z
updated_at: 2025-12-29T21:05:05Z
---

Implement proper motion planning to eliminate harsh motor noise when plotting curves.

## Problem
The plotter makes harsh sounds when plotting curves due to constant-velocity motion creating sudden direction changes at segment boundaries. Testing confirmed that segment length doesn't matter - the issue is lack of acceleration control.

## Solution
Implement a motion planner that uses the LM command's acceleration parameter to create smooth trapezoidal velocity profiles.

## Requirements
1. **Velocity lookahead** - Look ahead at upcoming segments to plan corner velocities
2. **Cornering velocity calculation** - Based on direction change angle and max acceleration
3. **Trapezoidal profiles** - Accelerate from corner velocity, cruise, decelerate to next corner velocity
4. **LM Accel parameter** - Calculate and use non-zero acceleration values

## Technical Approach
- Create a MotionPlanner struct that takes a list of segments
- For each segment, calculate: start velocity, end velocity, acceleration, cruise velocity
- Use kinematic equations: v² = v₀² + 2as, s = v₀t + ½at²
- Output LM commands with proper Rate and Accel values

## References
- Python plotink ebb_motion.py and ebb_calc.py
- EBB LM command documentation
- Grbl motion planning (similar problem domain)

## Checklist

### Phase 1: Core Motion Planner Module
- [x] Create `motion.rs` module in `drawing-plotter` crate
- [x] Define `MotionSegment` struct with start/end velocity, acceleration, steps, timing
- [x] Define `MotionPlanner` struct that takes segments and computes velocities
- [x] Add max velocity and max acceleration configuration to `PlotConfig`

### Phase 2: Velocity Planning Algorithm
- [x] Implement corner velocity calculation based on direction change angle
- [x] Implement forward pass: compute max entry velocity per segment (limited by corner velocity)
- [x] Implement backward pass: compute max exit velocity (deceleration limited)
- [x] Combine passes to get junction velocities between segments

### Phase 3: Trapezoidal Profile Generation
- [x] For each segment, compute accel/cruise/decel distances and times
- [x] Handle short segments that can't reach cruise velocity (triangular profile)
- [x] Calculate LM Rate and Accel parameters from velocity profile
- [x] Generate LM command sequences for multi-phase moves

### Phase 4: Integration with AxiDraw
- [x] Add `move_to_with_motion()` method that uses motion planner
- [x] Update `plot_optimized_strokes()` to use motion planning for pen-down moves
- [x] Keep existing constant-velocity for pen-up (travel) moves initially
- [x] Add configuration flag to enable/disable motion planning

### Phase 5: Testing and Tuning
- [x] Add unit tests for velocity calculations
- [x] Add unit tests for corner velocity formula
- [x] Add unit tests for trapezoidal profile generation
- [x] Add unit tests for LmCommand and PlannedMove
- [ ] Test on hardware with curves and measure noise reduction
- [ ] Tune max acceleration parameter for optimal results

## Implementation Notes

### LM Command Format
```
LM,Rate1,Steps1,Accel1,Rate2,Steps2,Accel2[,Clear]
```
- Rate: step rate added to accumulator every 40us (Rate = 85899.35 * frequency_hz)
- Accel: change in Rate every 40us (can be positive or negative)
- Steps carry direction sign (firmware 2.x)

### Kinematic Equations
- `v² = v₀² + 2as` - velocity from acceleration over distance
- `v = v₀ + at` - velocity from acceleration over time
- `s = v₀t + ½at²` - distance from initial velocity and acceleration
- `t = (v - v₀) / a` - time to change velocity

### Corner Velocity Formula (from Grbl)
```
junction_velocity = min(max_velocity, sqrt(2 * max_accel * deviation))
```
Where `deviation` depends on the angle between segments:
```
sin_half_angle = sin(angle / 2)
deviation = junction_deviation * sin_half_angle / (1 - sin_half_angle)
```

### LM Timing
- ISR interval: 40μs (25kHz)
- Rate = 2^31 / 25000 * freq_hz = 85899.3459 * freq_hz
- Accel = Rate_change_per_40μs

## Additional Features Added

### Servo Position and Rate Configuration
The EBB needs explicit servo configuration via SC commands:
- `SC,4,<value>` sets Servo_Min (pen UP position)
- `SC,5,<value>` sets Servo_Max (pen DOWN position)
- `SC,11,<value>` sets servo rate when raising (pen up)
- `SC,12,<value>` sets servo rate when lowering (pen down)
- EBB servo units: ~7500 (1ms pulse) to ~28000 (2ms pulse)
- Our 0-100 scale maps linearly to this range

**Bug Fix:** The pen wasn't fully raising before short travel moves because we weren't configuring the servo movement rate. The SP command's Duration parameter tells the EBB how long to wait before the next command, but the actual servo movement speed is controlled by SC,11/SC,12. Added `calculate_ebb_rate()` function to compute the correct EBB rate values that match our timing calculations.

### Plot Cancellation Support
Added user-initiated plot cancellation to the CLI:
- Press 'q' or 'Q' to cancel during plotting
- Ctrl+C also cancels (in raw terminal mode)
- On cancel: pen raises, returns home, motors disabled
- `PlotEvent::Cancelled` emitted when cancellation completes