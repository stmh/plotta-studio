---
# plotta-studio-9awy
title: Investigate plotter drifting/overdraw issue
status: in-progress
type: bug
created_at: 2025-12-31T16:41:53Z
updated_at: 2025-12-31T16:41:53Z
---

The plotter is drifting during plots, causing overdrawn sections that shouldn't be overdrawn. This is visible as offset/doubled strokes in curves (see Schnellstrasse map photo).

## Root Cause Analysis (Completed)

After comparing with Python AxiDraw driver, identified **two sources of cumulative drift**:

### 1. Position Tracking Uses Requested vs Actual Position
- **Rust**: `self.current_pos = planned.end` tracks the **requested** floating-point target
- **Python**: Tracks position based on **actual integer steps sent**
- Each move has tiny rounding errors that accumulate over thousands of segments

**Location:** `axidraw.rs:559` and `axidraw.rs:594`

### 2. validate_and_adjust() Can Modify/Skip Steps
- `SmCommand::validate_and_adjust()` can zero out steps for underspeed (motor resonance prevention)
- `generate_sm_commands()` tracks **intended** steps, not **actual** steps after validation
- Skipped commands still increment cumulative tracking, causing drift

**Location:** `motion.rs:898-911`

## Fix Plan

### Phase 1: Create Drift Test Patterns
**File:** `crates/plotta-cli/examples/create_drift_test.rs`
- Concentric squares pattern (10 squares, all starting from same corner)
- Zigzag pattern returning to origin
- Both saved to project root directory
- Quick to plot (< 3 minutes), clearly shows drift if present

### Phase 2: Add Verify Position Option
**File:** `crates/drawing-plotter/src/config.rs`
- Add `verify_position: bool` to `PlotConfig`
- When enabled, queries hardware position (QS) after each stroke
- Logs any discrepancy between tracked and hardware position
- Reports cumulative/max error at end of plot

### Phase 3: Fix Position Tracking (Core Fix)

#### 3a. Modify SmPlannedMove struct
**File:** `crates/drawing-plotter/src/motion.rs` (lines 789-816)
- Add `actual_steps_x: i32` and `actual_steps_y: i32` fields
- These track the actual steps sent after validation/adjustment

#### 3b. Update generate_sm_commands()
**File:** `crates/drawing-plotter/src/motion.rs` (lines 826-920)
- Track actual steps from validated commands (after `validate_and_adjust()`)
- Sum up actual steps from each command that gets pushed
- Handle skipped commands correctly (don't count their steps)

#### 3c. Update generate_single_sm_command()
**File:** `crates/drawing-plotter/src/motion.rs` (lines 922-966)
- Same treatment - track actual steps after validation

#### 3d. Update execute_sm_planned_move()
**File:** `crates/drawing-plotter/src/axidraw.rs` (lines 587-596)
```rust
// Use actual steps to calculate position:
self.current_pos = Point::new(
    planned.start.x + (planned.actual_steps_x as f64 / Self::STEPS_PER_MM),
    planned.start.y + (planned.actual_steps_y as f64 / Self::STEPS_PER_MM),
);
```

#### 3e. Update move_to()
**File:** `crates/drawing-plotter/src/axidraw.rs` (lines 510-561)
- Calculate actual position from integer steps sent (not requested target)

### Phase 4: Add Position Verification Logging
**File:** `crates/drawing-plotter/src/axidraw.rs`
- Add `verify_position()` method that queries QS and compares to tracked
- Track max and cumulative error during plot
- Log summary at end: "Position verification: N checks, max error=X.XXXXmm, avg error=X.XXXXmm"

### Phase 5: Add CLI Flags
**File:** `crates/plotta-cli/src/main.rs`
- Add `--verify-position` flag to both `Plot` and `Preview` commands
- For Preview, flag is accepted but does nothing (API consistency)

## Files to Modify/Create

| File | Action | Description |
|------|--------|-------------|
| `crates/plotta-cli/examples/create_drift_test.rs` | CREATE | Test pattern generator |
| `crates/drawing-plotter/src/config.rs` | MODIFY | Add `verify_position: bool` |
| `crates/drawing-plotter/src/motion.rs` | MODIFY | Add `actual_steps_x/y` to SmPlannedMove |
| `crates/drawing-plotter/src/axidraw.rs` | MODIFY | Fix position tracking, add verification |
| `crates/plotta-cli/src/main.rs` | MODIFY | Add `--verify-position` CLI flag |

## Checklist

### Investigation (Completed)
- [x] Review motion planning code for potential issues
- [x] Check servo timing parameters
- [x] Investigate step rate calculations
- [x] Look for missing delays between commands
- [x] Compare with known working AxiDraw implementations

### Implementation (Todo)
- [ ] Create drift test patterns (concentric squares + zigzag)
- [ ] Add `verify_position` option to PlotConfig
- [ ] Add `actual_steps_x/y` fields to SmPlannedMove
- [ ] Update generate_sm_commands() to track actual steps
- [ ] Update generate_single_sm_command() to track actual steps
- [ ] Fix execute_sm_planned_move() to use actual steps
- [ ] Fix move_to() to use actual steps
- [ ] Add verify_position() method with cumulative error tracking
- [ ] Add --verify-position CLI flag to Plot command
- [ ] Add --verify-position CLI flag to Preview command
- [ ] Test fix with drift test patterns
- [ ] Verify no regression in existing functionality
