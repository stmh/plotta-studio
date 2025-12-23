---
# plotta-studio-axi1
title: AxiDraw plotter control
status: todo
type: epic
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
---

Implement full AxiDraw pen plotter control via USB serial communication using the EBB (EiBotBoard) protocol.

## Investigation

### Current State
- `drawing-plotter/src/lib.rs` has scaffolding:
  - `PlotConfig` struct with speed/pen settings
  - `PlotterError` enum for error handling
  - `AxiDraw` struct with placeholder methods
  - `optimize_strokes()` greedy nearest-neighbor implementation
- TODO comment at line 151-152 mentions needing `serialport` crate

### EBB Protocol Reference
The AxiDraw uses the EiBotBoard (EBB) protocol over USB serial (typically 9600-115200 baud).

Key commands:
- `SM,duration,axis1,axis2\r` - Stepper move (duration in ms, steps)
- `SP,value,duration,portB\r` - Servo position (pen up/down)
- `EM,enable1,enable2\r` - Enable/disable motors (1=on, 0=off)
- `QP\r` - Query pen state (returns 0=up, 1=down)
- `V\r` - Query firmware version
- `R\r` - Reset

### Motor Calculations
AxiDraw uses 16 microsteps/step, 200 steps/rev, ~40mm/rev:
- Steps per mm ≈ 80 (16 * 200 / 40)
- Axis mapping: axis1 = X+Y, axis2 = X-Y (CoreXY-like)

### Implementation Architecture

```rust
pub struct AxiDraw {
    port: Box<dyn serialport::SerialPort>,
    config: PlotConfig,
    current_pos: Point,
    pen_is_down: bool,
}

impl AxiDraw {
    pub fn connect(port_name: &str) -> Result<Self, PlotterError>;
    pub fn list_ports() -> Result<Vec<String>, PlotterError>;

    // Core operations
    pub fn pen_up(&mut self) -> Result<(), PlotterError>;
    pub fn pen_down(&mut self) -> Result<(), PlotterError>;
    pub fn move_to(&mut self, pos: Point) -> Result<(), PlotterError>;
    pub fn line_to(&mut self, pos: Point) -> Result<(), PlotterError>;

    // High-level
    pub fn plot(&mut self, drawing: &Drawing) -> Result<(), PlotterError>;
    pub fn plot_strokes(&mut self, strokes: &[&Stroke]) -> Result<(), PlotterError>;

    // Utilities
    pub fn home(&mut self) -> Result<(), PlotterError>;
    pub fn disable_motors(&mut self) -> Result<(), PlotterError>;
    pub fn query_pen(&mut self) -> Result<bool, PlotterError>;
}
```

### Dependencies to Add
```toml
[dependencies]
serialport = "4.3"
```

### Key Challenges
1. **Timing**: Must wait for moves to complete before sending next command
2. **Coordinate transforms**: Drawing coords (mm) → motor steps
3. **Speed control**: Calculate move duration based on distance and speed settings
4. **Error recovery**: Handle communication errors, emergency stop
