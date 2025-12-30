---
# plotta-studio-dd8k
title: Address PR review feedback for plotta-cli PR
status: in-progress
type: task
priority: normal
created_at: 2025-12-29T12:45:10Z
updated_at: 2025-12-29T12:45:18Z
parent: plotta-studio-7wzz
---

Fix issues identified in PR #12 code review.

## Critical Issues (Must Fix Before Merge)

### 1. Race Condition in PauseControl::toggle()
**File:** crates/drawing-plotter/src/event.rs:63-67
- toggle() method is not atomic - between load and store another thread could modify
- **Fix:** Use fetch_xor for atomic toggle

### 2. Integer Truncation in Step Calculation
**File:** crates/drawing-plotter/src/axidraw.rs:376-377
- Casting f64 to i32 truncates, could cause positioning drift
- **Fix:** Use round() before casting

### 3. Add Bounds Checking for Plotter Movements
**File:** crates/drawing-plotter/src/axidraw.rs:367-398
- move_to() accepts any Point without checking physical limits
- **Fix:** Add bounds validation with MAX_X/MAX_Y constants

## High Priority Issues

### 4. Overflow Potential in CoreXY Transform
**File:** crates/drawing-plotter/src/axidraw.rs:189-198
- No overflow checking for axis1/axis2 addition
- **Fix:** Use checked arithmetic

### 5. Closed Stroke Logic for Reversed Strokes
**File:** crates/drawing-plotter/src/axidraw.rs:449-451
- Reversed closed strokes may not close properly
- **Fix:** Always close if marked as closed, regardless of reversal

### 6. Validate Input File Sizes
**File:** crates/plotta-cli/src/main.rs:240-241, 299-300
- No limits on JSON file size, could cause OOM
- **Fix:** Add MAX_FILE_SIZE and MAX_STROKES limits

### 7. Validate Serial Port Paths
**File:** crates/plotta-cli/src/main.rs:160-163
- User port paths passed directly without validation
- **Fix:** Validate port paths format

## Checklist

- [x] Fix race condition in PauseControl::toggle() with fetch_xor
- [x] Use round() before casting f64 to i32 in step calculation
- [ ] Add bounds checking for plotter movements (deferred to plotta-studio-axi5)
- [x] Add checked arithmetic for CoreXY transform
- [x] Fix closed stroke handling for reversed strokes
- [x] Add input file size validation
- [x] Add serial port path validation