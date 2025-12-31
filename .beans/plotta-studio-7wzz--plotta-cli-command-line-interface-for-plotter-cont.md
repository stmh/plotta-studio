---
# plotta-studio-7wzz
title: Plotta CLI - Command-line interface for plotter control
status: completed
type: epic
priority: normal
created_at: 2025-12-28T15:31:25Z
updated_at: 2025-12-31T13:55:03Z
---

A command-based CLI tool for controlling the plotter and plotting drawings.

## Commands

| Command | Description |
|---------|-------------|
| `list` | List available plotter ports |
| `status` | Check connection to plotter |
| `pen-up` | Raise the pen |
| `pen-down` | Lower the pen |
| `home` | Send pen to home position (0,0) |
| `plot <file>` | Plot drawing from JSON file with progress bar |
| `preview <file>` | Show drawing stats without plotting |

## Global Options

- `--port <path>` - Serial port path (e.g., `/dev/tty.usbmodem1234`). Auto-detects if omitted.

## Architecture

```
┌─────────────────┐
│   plotta-cli    │  ← CLI concerns only: args, progress bar, output
├─────────────────┤
│ drawing-plotter │  ← Stats, time estimation, plotting
├─────────────────┤
│  drawing-core   │  ← Drawing type, JSON serialization
└─────────────────┘
```

Keep the CLI thin - push reusable logic into libraries so a future UI can reuse it.

## File Format

JSON serialization of the `Drawing` struct from `drawing-core`. Sketches save `.json` files, CLI loads them.

## Progress Bar

During `plot` command:
```
Plotting: [████████████░░░░░░░░░░░░] 34/100 strokes (34%)
```

## Dependencies

- `clap` - argument parsing (already in workspace)
- `indicatif` - progress bar
- `drawing-plotter` - plotter control
- `drawing-core` - Drawing type
- `anyhow` - error handling

## Checklist

- [x] Add JSON serialization to Drawing in drawing-core
- [x] Add DrawingStats and calculate_stats() to drawing-plotter
- [x] Add estimate_plot_time() to drawing-plotter  
- [x] Create plotta-cli crate with Cargo.toml
- [x] Implement list command
- [x] Implement status command
- [x] Implement pen-up command
- [x] Implement pen-down command
- [x] Implement home command
- [x] Implement preview command
- [x] Implement plot command with progress bar