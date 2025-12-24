---
# plotta-studio-axi2b
title: Port discovery and auto-detection
status: completed
type: task
created_at: 2025-12-24T00:00:00Z
updated_at: 2025-12-24T00:00:00Z
parent: plotta-studio-axi2
---

Implement port listing and AxiDraw auto-detection by VID/PID.

## USB Identifiers

- Vendor ID: `0x04D8` (Microchip)
- Product ID: `0xFD92` (EiBotBoard)

## Implementation

1. `list_ports()` - list all USB serial ports
2. `find_devices()` - find AxiDraw devices by VID/PID
3. `find_first()` - convenience method for single device
