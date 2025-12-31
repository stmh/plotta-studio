---
# plotta-studio-t34x
title: Implement dynamic servo timing like Python AxiDraw driver
status: completed
type: task
priority: normal
created_at: 2025-12-29T13:36:16Z
updated_at: 2025-12-31T13:48:23Z
parent: plotta-studio-q6v8
---

The Python AxiDraw driver calculates servo timing dynamically based on pen position travel distance, rather than using fixed delays.

## Current behavior
- Our implementation uses a fixed delay (e.g., 500ms) for both the SP command duration AND as a sleep time after
- The delay is the same regardless of how far the servo needs to travel

## Python driver approach
- `pen_delay_up/down = 0` by default (no additional delay)
- Servo timing calculated as: `time = (servo_move_slope * distance + servo_move_min) * (100 / rate)`
- Default values: `servo_move_min = 45ms`, `servo_move_slope = 2.69ms per %`
- Default rates: `pen_rate_raise = 75`, `pen_rate_lower = 50`
- For 30% travel at 100% rate: ~126ms
- For 30% travel at 75% rate (raise): ~168ms
- For 30% travel at 50% rate (lower): ~251ms

## Implementation

Added to `PlotConfig`:
- `pen_rate_raise: u8` (default 75) - servo speed when raising pen
- `pen_rate_lower: u8` (default 50) - servo speed when lowering pen

The `servo::calculate_move_time(from_pos, to_pos, rate)` function now takes rate into account, scaling time inversely (lower rate = longer time).

## Checklist
- [x] Add servo timing calculation based on pen position delta
- [x] Use calculated timing for SP command duration parameter
- [x] Make pen_delay_up/down truly optional additional delays (default 0)
- [x] Update PlotConfig defaults to match Python driver
- [x] Add pen_rate_raise and pen_rate_lower to PlotConfig
- [x] Scale timing by rate (lower rate = longer time)
- [x] Update tests for new rate-scaled timing
- [ ] Test with hardware