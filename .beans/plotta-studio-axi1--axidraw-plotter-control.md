---
# plotta-studio-axi1
title: AxiDraw plotter control
status: in-progress
type: epic
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-29T12:30:00Z
---

Implement full AxiDraw pen plotter control via USB serial communication using the EBB (EiBotBoard) protocol.

## Status

Most core functionality is implemented and working:
- Serial communication (completed)
- EBB commands (completed)
- Plot drawing function (completed)
- XM command for mixed-axis geometry (completed)
- Safety and bounds checking (in-progress)

## EBB Protocol Reference

The AxiDraw uses the EiBotBoard (EBB) protocol over USB serial (115200 baud).

Key commands:
- `XM,duration,stepsX,stepsY\r` - Stepper move for mixed-axis geometry (CoreXY)
  - Firmware handles CoreXY transform internally: axis1 = X+Y, axis2 = X-Y
- `SP,value,duration\r` - Servo position (pen up/down)
- `EM,enable1,enable2\r` - Enable/disable motors (1=on, 0=off)
- `QP\r` - Query pen state (returns 1=up, 0=down)
- `QS\r` - Query step position (returns axis1,axis2 in steps)
- `V\r` - Query firmware version

Note: We use XM instead of SM because XM is specifically designed for mixed-axis 
geometry machines like AxiDraw. It handles the CoreXY transform internally.

## Motor Calculations

AxiDraw uses 16 microsteps/step, 200 steps/rev, ~40mm/rev:
- Steps per mm = 80 (16 * 200 / 40)
- XM command takes X/Y steps directly; firmware does CoreXY transform

## Implementation

Located in `crates/drawing-plotter/src/axidraw.rs`:

```rust
pub struct AxiDraw {
    port: BufReader<Box<dyn serialport::SerialPort>>,
    config: PlotConfig,
    current_pos: Point,
    pen_is_down: bool,
}

impl AxiDraw {
    // Connection
    pub fn connect(port_name: &str) -> Result<Self, PlotterError>;
    pub fn auto_connect() -> Result<Self, PlotterError>;
    pub fn find_devices() -> Result<Vec<String>, PlotterError>;

    // Core operations
    pub fn pen_up(&mut self) -> Result<(), PlotterError>;
    pub fn pen_down(&mut self) -> Result<(), PlotterError>;
    pub fn move_to(&mut self, pos: Point) -> Result<(), PlotterError>;

    // High-level
    pub fn plot(&mut self, drawing: &Drawing, config: &PlotConfig, ctx: &RenderContext) -> Result<(), PlotterError>;
    pub fn plot_strokes(&mut self, strokes: &[&Stroke]) -> Result<(), PlotterError>;

    // Utilities
    pub fn home(&mut self) -> Result<(), PlotterError>;
    pub fn enable_motors(&mut self) -> Result<(), PlotterError>;
    pub fn disable_motors(&mut self) -> Result<(), PlotterError>;
    pub fn query_pen(&mut self) -> Result<bool, PlotterError>;
    pub fn query_step_position(&mut self) -> Result<Point, PlotterError>;
}
```

## Remaining Work

- [ ] Add bounds checking for plotter limits
- [ ] Add emergency stop functionality
- [ ] Add progress callbacks during plotting
