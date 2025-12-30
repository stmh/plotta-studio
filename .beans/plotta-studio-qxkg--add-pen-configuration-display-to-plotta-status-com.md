---
# plotta-studio-qxkg
title: Add pen configuration display to plotta status command
status: completed
type: task
priority: normal
created_at: 2025-12-29T14:02:46Z
updated_at: 2025-12-29T14:04:52Z
---

Query and display the actual pen servo configuration (min/max positions, rates) from the EBB when running the status command. This helps diagnose timing issues like ghost lines from insufficient pen-up time.

## Implementation

Added to `plotta status` command:
- Display pen positions (down/up) and delta percentage
- Display servo rates for raise/lower
- Display calculated servo timing
- Display additional delays
- Display total wait times

Added new CLI options to `plotta plot` and `plotta preview`:
- `--pen-down-pos` (0-100, default 30)
- `--pen-up-pos` (0-100, default 60)
- `--pen-rate-raise` (1-100, default 75, lower=slower)
- `--pen-rate-lower` (1-100, default 50, lower=slower)

Note: The EBB firmware doesn't provide a query for reading back SC configuration values (Servo_Min, Servo_Max, Servo_Rate). We display our configured values and calculated timings instead.

## Fixing ghost lines

If you're getting ghost lines (pen not fully lifted before travel):

1. **Increase pen-up delay**: `--pen-up-delay 200` adds 200ms after lifting
2. **Slow down the raise rate**: `--pen-rate-raise 50` (default is 75)
3. **Increase pen-up position**: `--pen-up-pos 70` lifts higher (default is 60)