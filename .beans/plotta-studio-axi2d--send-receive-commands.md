---
# plotta-studio-axi2d
title: Implement send/receive commands
status: completed
type: task
created_at: 2025-12-24T00:00:00Z
updated_at: 2025-12-24T00:00:00Z
parent: plotta-studio-axi2
---

Implement core EBB command sending and response reading.

## EBB Protocol

- Commands end with `\r`
- Responses end with `\r\n`
- Success: `OK\r\n`
- Error: starts with `!`

## Implementation

```rust
fn send_command(&mut self, cmd: &str) -> Result<String, PlotterError> {
    // 1. Write "{cmd}\r" to port
    // 2. Read response until "\r\n"
    // 3. Check for error prefix "!"
    // 4. Return response string
}

fn send_command_no_response(&mut self, cmd: &str) -> Result<(), PlotterError> {
    // For commands that don't return data
}
```
