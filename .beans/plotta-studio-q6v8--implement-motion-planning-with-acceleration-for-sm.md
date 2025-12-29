---
# plotta-studio-q6v8
title: Implement motion planning with acceleration for smooth plotting
status: todo
type: feature
created_at: 2025-12-29T18:06:40Z
updated_at: 2025-12-29T18:06:40Z
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