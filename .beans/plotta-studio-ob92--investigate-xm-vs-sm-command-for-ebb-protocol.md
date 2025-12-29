---
# plotta-studio-ob92
title: Investigate XM vs SM command for EBB protocol
status: completed
type: task
priority: normal
created_at: 2025-12-29T12:24:55Z
updated_at: 2025-12-29T12:31:35Z
parent: plotta-studio-axi1
---

Research the differences between XM and SM commands for EBB stepper motor control. XM is designed for mixed-axis geometries (CoreXY, H-Bot, AxiDraw) and may provide better performance than SM. Document findings, potential changes needed, and side effects.

## Research Findings

### Overview

The EBB (EiBotBoard) firmware provides two stepper motor commands that can be used for movement:

1. **SM (Stepper Move)**: Generic stepper motor command
2. **XM (Stepper Move for Mixed-axis Geometries)**: Specialized command for CoreXY/H-Bot/AxiDraw

### Key Differences

#### SM Command
```
SM,Duration,AxisSteps1,AxisSteps2
```
- Takes motor 1 and motor 2 step counts directly
- Caller must perform CoreXY transform manually: `axis1 = X+Y`, `axis2 = X-Y`

#### XM Command
```
XM,Duration,AxisStepsA,AxisStepsB
```
- Takes X and Y step counts directly
- EBB firmware performs the CoreXY transform internally: `AxisSteps1 = A+B`, `AxisSteps2 = A-B`
- Specifically designed for mixed-axis geometry machines (CoreXY, H-Bot, AxiDraw)

### Current Implementation in `axidraw.rs`

The current code at line 392 uses SM and performs the CoreXY transform in Rust:

```rust
// CoreXY transform: axis1 = X+Y, axis2 = X-Y
let axis1 = steps_x + steps_y;
let axis2 = steps_x - steps_y;

let cmd = format!("SM,{},{},{}", duration_ms, axis1, axis2);
```

### Benefits of Switching to XM

1. **Simpler Code**: The CoreXY transform would be handled by the EBB firmware, not our code
2. **Cleaner Intent**: Using XM makes it explicit that we're controlling a mixed-axis machine
3. **Firmware Optimization**: The EBB firmware is designed specifically for this use case
4. **Consistency**: Matches the official AxiDraw Python drivers which use XM

### Required Changes

The change is minimal - just modify the `move_to` method:

```rust
// Before (current):
let axis1 = steps_x + steps_y;
let axis2 = steps_x - steps_y;
let cmd = format!("SM,{},{},{}", duration_ms, axis1, axis2);

// After (proposed):
let cmd = format!("XM,{},{},{}", duration_ms, steps_x, steps_y);
```

### Side Effects Analysis

| Aspect | Impact | Notes |
|--------|--------|-------|
| Behavior | None | XM performs the same transform internally |
| Performance | Minimal | May be slightly faster (fewer operations on host) |
| Firmware Compatibility | Good | XM available since firmware v2.3.0 (2014) |
| Position Tracking | Compatible | QS still uses axis1/axis2, requires reverse transform |
| Error Messages | Compatible | Same error handling applies |

### Potential Concerns

1. **Firmware Version**: XM requires firmware v2.3.0+. Very old EBBs might not support it, but this is extremely unlikely for any AxiDraw in active use.

2. **Query Step Position**: The `query_step_position()` method still needs to reverse the CoreXY transform because QS returns axis1/axis2 values. This is correct and should not change.

3. **Testing**: Should test with actual hardware to verify behavior matches.

### Recommendation

**Recommend switching to XM** for the following reasons:

1. It's the semantically correct command for AxiDraw (a mixed-axis machine)
2. Simplifies the code by removing manual CoreXY transform in move operations
3. All modern AxiDraw devices support it
4. Official AxiDraw software uses XM

The change is low-risk and improves code clarity.

## Implementation Notes

If we decide to implement this change:

1. Update `move_to()` in `crates/drawing-plotter/src/axidraw.rs`
2. Update the unit test `test_corexy_transform` to reflect the new simpler approach
3. Test with hardware to verify correct behavior
4. Consider adding a firmware version check (optional, low priority)
