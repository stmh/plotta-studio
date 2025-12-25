---
# plotta-studio-axi2a
title: Add serialport dependency
status: completed
type: task
created_at: 2025-12-24T00:00:00Z
updated_at: 2025-12-24T00:00:00Z
parent: plotta-studio-axi2
---

Add the serialport crate and update AxiDraw struct to hold serial port.

## Implementation

1. Uncomment `serialport = "4.3"` in `crates/drawing-plotter/Cargo.toml`
2. Add `Timeout` and `InvalidResponse` error variants
3. Update `AxiDraw` struct with `port: Box<dyn serialport::SerialPort>` field
