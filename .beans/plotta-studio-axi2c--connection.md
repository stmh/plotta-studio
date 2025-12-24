---
# plotta-studio-axi2c
title: Implement connection
status: completed
type: task
created_at: 2025-12-24T00:00:00Z
updated_at: 2025-12-24T00:00:00Z
parent: plotta-studio-axi2
---

Implement serial port connection with EBB handshake.

## Implementation

1. `connect(port_name: &str)` - open port at 115200 baud, 1s timeout
2. `auto_connect()` - find first device and connect
3. Verify connection with `V` (version query) command
4. Implement `Drop` to disable motors on disconnect
