---
# plotta-studio-ckzl
title: Add plotter setup diagram to plotta-cli
status: completed
type: feature
priority: normal
created_at: 2026-01-01T16:04:45Z
updated_at: 2026-01-04T17:21:36Z
---

Add a terminal-based setup diagram to the `plot` command showing the plotter, bed, and drawing placement with color support. This helps users verify their physical setup matches the expected layout before plotting.

## Overview

A colored terminal diagram rendered before plotting that shows:
- Plotter body (700x100mm) - grey
- Bed area (460x325mm) - grey  
- Drawing rectangle (actual dimensions) - white
- Markers for paper origin, printhead home, up-vector - green

The diagram scales proportionally to fit 40-60 characters width, with aspect ratio compensation for terminal characters.

## CLI Changes

### New flags for `plotta plot`:
- `--plotter-setup <POSITION>` / `-s`: One of `top`, `bottom`, `left`, `right` (default: `top`)
- `--yes` / `-y`: Skip confirmation, start plotting immediately
- Env var `PLOTTA_PLOTTER_SETUP` as fallback before the default

### Layout positions:
- `top` - Plotter above bed, up-vector points up (default, easiest)
- `bottom` - Plotter below bed, up-vector points down
- `left` - Plotter left of bed, up-vector points left
- `right` - Plotter right of bed, up-vector points right

### Flow:
1. Load and validate drawing
2. Render the setup diagram to terminal (unless `--yes`)
3. Show legend and existing stats
4. Prompt: "Press Enter to start plotting, Ctrl+C to cancel"
5. On Enter -> proceed to plotting

## New Modules

### `crates/plotta-cli/src/canvas.rs`

Generic 2D terminal canvas with ANSI color support:

```rust
pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,  // [y][x] for row-major access
}

struct Cell {
    char: char,
    color: Color,
}

pub enum Color {
    Default,
    Grey,
    White,
    Green,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color);
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Color);
    pub fn draw_char(&mut self, x: i32, y: i32, ch: char, color: Color);
    pub fn render(&self);  // Output to terminal with ANSI codes
}
```

Features:
- Box-drawing characters for rectangles (corners, edges)
- Aspect ratio compensation (2 horizontal chars per vertical unit)
- ANSI 16-color support via crossterm

### `crates/plotta-cli/src/setup_diagram.rs`

Plotter setup diagram renderer:

```rust
pub enum PlotterSetup {
    Top,     // Plotter above bed, up-vector points up
    Bottom,  // Plotter below bed, up-vector points down
    Left,    // Plotter left of bed, up-vector points left
    Right,   // Plotter right of bed, up-vector points right
}

pub struct SetupDiagram { ... }

impl SetupDiagram {
    pub fn new(setup: PlotterSetup, drawing_width: f64, drawing_height: f64) -> Self;
    pub fn render_to_terminal(&self);
    pub fn print_legend();
}
```

## Dimensions (hardcoded for now)

- Plotter body: 700mm x 100mm
- Bed: 460mm x 325mm
- Drawing: from the loaded drawing file

## Drawing Order (back to front)

1. Plotter body rectangle (grey)
2. Bed rectangle (grey)
3. Drawing rectangle (white) - positioned at paper origin on bed
4. Markers (green):
   - Paper origin: Unicode marker at (0,0) of drawing
   - Printhead home: Unicode marker at plotter's home position
   - Up-vector arrow: Points toward plotter

## Legend Output

```
Legend:
  [marker]  Paper origin (0,0)
  [marker]  Printhead home
  [arrow]   Up direction

Drawing: example.json (297 x 210 mm)

[existing stats output]

Press Enter to start plotting, Ctrl+C to cancel
```

## Debug Example

Create `crates/plotta-cli/examples/setup_diagrams.rs` that renders all 4 orientations side by side (or sequentially) with a sample A4 landscape drawing size. This makes it easy to visually verify all layouts without needing a real drawing file.

```bash
cargo run -p plotta-cli --example setup_diagrams
```

Output should show:
1. `top` layout with header
2. `bottom` layout with header
3. `left` layout with header
4. `right` layout with header

Each with the legend printed once at the end.

## Checklist

- [x] Create `canvas.rs` module with Canvas struct and Color enum
- [x] Implement `draw_rect` with box-drawing characters
- [x] Implement `draw_text` and `draw_char`
- [x] Implement `render()` with ANSI color output
- [x] Create `setup_diagram.rs` module with PlotterSetup enum
- [x] Implement SetupDiagram::new() with scaling logic
- [x] Implement render_to_terminal() for all 4 orientations
- [x] Implement print_legend_for_setup()
- [x] Create `examples/setup_diagrams.rs` to render all 4 orientations
- [x] Add `--plotter-setup` CLI argument with env var fallback
- [x] Add `--yes` / `-y` CLI argument
- [x] Integrate into plot command flow
- [x] Add wait_for_enter() function using crossterm
