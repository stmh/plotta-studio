# drawing-plotter

AxiDraw pen plotter control for plotta-studio.

## Hardware Compatibility

This crate has been tested with:
- **AxiDraw V3** with EBB Firmware Version 2.8.1

## EBB Protocol

This crate communicates with the AxiDraw using the EiBotBoard (EBB) serial protocol.

**Protocol Documentation:** https://evil-mad.github.io/EggBot/ebb.html

### Key Commands Used

| Command | Description |
|---------|-------------|
| `V` | Query firmware version |
| `EM,e1,e2` | Enable/disable motors (0=disable, 1-5=enable with microstep mode) |
| `SM,duration,axis1,axis2` | Stepper move (CoreXY: axis1=X+Y, axis2=X-Y) |
| `SP,state[,duration]` | Set pen state (0=down, 1=up) |
| `QP` | Query pen state (returns 0=up, 1=down) |
| `QS` | Query step position (returns axis1,axis2 in steps) |

### Coordinate System

The AxiDraw uses CoreXY kinematics:
- **Forward transform** (mm to steps): `axis1 = X + Y`, `axis2 = X - Y`
- **Reverse transform** (steps to mm): `X = (axis1 + axis2) / 2`, `Y = (axis1 - axis2) / 2`
- **Steps per mm**: 80 (16 microsteps * 200 steps/rev / 40mm per rev)

### Pen Control

The EBB uses inverted logic for pen commands vs queries:
- `SP,0` = Move pen to DOWN position (Servo_Max)
- `SP,1` = Move pen to UP position (Servo_Min)
- `QP` returns `0` when pen is UP, `1` when pen is DOWN

## Features

- `hardware` (default): Enables serial port communication with real AxiDraw devices.

## Usage

```rust
use drawing_plotter::{AxiDraw, PlotConfig};
use drawing_core::RenderContext;

// Auto-connect to first available AxiDraw
let mut plotter = AxiDraw::auto_connect()?;

// Plot a drawing
plotter.plot(&drawing, &PlotConfig::default(), &ctx)?;
```

## Hardware Tests

Run manual hardware tests with a connected AxiDraw:

```bash
# Run all hardware tests
cargo test -p drawing-plotter -- --ignored --nocapture

# Run specific test
cargo test -p drawing-plotter test_pen_toggle -- --ignored --nocapture
```
