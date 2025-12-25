---
# plotta-studio-axi2
title: Implement serial communication
status: completed
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-axi1
---

Add serialport dependency and implement basic serial communication with the EBB.

## Implementation Details

1. Add to `crates/drawing-plotter/Cargo.toml`:
```toml
serialport = "4.3"
```

2. Implement connection:
```rust
use serialport::{SerialPort, SerialPortType};
use std::time::Duration;

impl AxiDraw {
    pub fn list_ports() -> Result<Vec<String>, PlotterError> {
        let ports = serialport::available_ports()
            .map_err(|e| PlotterError::Connection(e.to_string()))?;

        Ok(ports
            .into_iter()
            .filter(|p| matches!(p.port_type, SerialPortType::UsbPort(_)))
            .map(|p| p.port_name)
            .collect())
    }

    pub fn connect(port_name: &str) -> Result<Self, PlotterError> {
        let port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(1000))
            .open()
            .map_err(|e| PlotterError::Connection(e.to_string()))?;

        let mut axidraw = Self {
            port,
            config: PlotConfig::default(),
            current_pos: Point::ZERO,
            pen_is_down: false,
        };

        // Verify connection with version query
        axidraw.send_command("V")?;

        Ok(axidraw)
    }

    fn send_command(&mut self, cmd: &str) -> Result<String, PlotterError> {
        let cmd_bytes = format!("{}\r", cmd);
        self.port
            .write_all(cmd_bytes.as_bytes())
            .map_err(|e| PlotterError::Communication(e.to_string()))?;

        // Read response until OK or error
        let mut response = String::new();
        // ... read logic
        Ok(response)
    }
}
```

## Auto-detect AxiDraw
Look for USB devices with specific VID/PID:
- Vendor ID: 0x04D8 (Microchip)
- Product ID: 0xFD92 (EiBotBoard)

## Subtasks

Implementation is broken into focused child beans:

1. **axi2a** - Add serialport dependency and update types
2. **axi2b** - Port discovery and auto-detection (VID/PID)
3. **axi2c** - Connection with EBB handshake
4. **axi2d** - Send/receive command protocol

## Files to Modify

- `crates/drawing-plotter/Cargo.toml`
- `crates/drawing-plotter/src/lib.rs`
