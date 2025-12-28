---
# plotta-studio-zmeb
title: Fix pen-up, pen-down, and home commands not working
status: completed
type: bug
priority: normal
created_at: 2025-12-28T15:59:28Z
updated_at: 2025-12-28T16:47:17Z
---

The move command works, but pen-up, pen-down, and home commands do not work. Need to investigate why.

## Root Causes

There were multiple issues:

1. **Pen commands inverted**: `SP,0` = pen DOWN, `SP,1` = pen UP (we had it backwards)
2. **QP parsing inverted**: `QP` returns `1` for UP, `0` for DOWN (we had it backwards)
3. **Step counter reset on disconnect**: The `Drop` implementation called `disable_motors()` which was resetting the EBB step counters, causing position to show (0,0) on reconnect
4. **Multi-line response parsing**: `QS` and `QP` commands return data followed by `OK` on separate lines

## Solution

1. Fixed pen commands: `SP,1` for UP, `SP,0` for DOWN
2. Fixed `query_pen()`: Returns true (DOWN) when QP response is "0"
3. Removed `disable_motors()` from Drop implementation
4. Added proper multi-line response handling for QS and QP commands
5. Added `query_step_position()` method with CoreXY reverse transform
6. Added `sync_state()` to query hardware state on connect
7. Added `pen_up_with_force()` and `pen_down_with_force()` for explicit user commands
8. Created README.md documenting EBB protocol compatibility

## Verified Working

- `plotta status` - Shows correct position and pen state across reconnects
- `plotta pen-up` / `plotta pen-down` - Correctly raises/lowers pen
- `plotta home` - Moves to (0,0) from any position
- `plotta move X Y` - Moves to absolute position