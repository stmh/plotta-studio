---
# plotta-studio-vd8d
title: Implement plotta CLI commands
status: completed
type: task
priority: normal
created_at: 2025-12-28T15:31:56Z
updated_at: 2025-12-28T20:07:23Z
parent: plotta-studio-7wzz
---

Implement all CLI commands in plotta-cli.

## Commands to Implement

### Device Commands

**list** - List available plotter ports
- Use `AxiDraw::list_ports_detailed()`
- Show port path and description

**status** - Check plotter connection
- Connect to plotter (via --port or auto-detect)
- Query pen state with `query_pen()`
- Print connection status and pen position

### Pen Commands

**pen-up** - Raise the pen
- Connect to plotter
- Call `pen_up()`
- Print confirmation

**pen-down** - Lower the pen
- Connect to plotter
- Call `pen_down()`
- Print confirmation

**home** - Send to home position
- Connect to plotter
- Call `home()`
- Print confirmation

### File Commands

**preview** - Show drawing stats
- Load JSON file, deserialize to `Drawing`
- Call `calculate_stats()`
- Print stats in human-readable format:
  ```
  Drawing: my-drawing.json
    Strokes: 142
    Pen-down distance: 1,234.5 mm
    Travel distance: 567.8 mm
    Estimated time: ~3m 24s
  ```

**plot** - Plot with progress bar
- Load JSON file, deserialize to `Drawing`
- Connect to plotter
- Optimize strokes with `optimize_strokes()`
- Call `plot_in_background()` to get `PlotHandle`
- Create `indicatif::ProgressBar`
- Loop over events from handle:
  - `Started` → set bar length
  - `StrokeComplete` → increment bar
  - `Completed` → finish bar, print summary
  - `Error` → abort with error

## Helper Function

Create connection helper:
```rust
fn connect_plotter(port: Option<&str>) -> Result<AxiDraw> {
    match port {
        Some(p) => AxiDraw::connect(p),
        None => AxiDraw::auto_connect(),
    }
}
```