---
# plotta-studio-ckzl
title: Add ASCII plotter layout diagram to plotta-cli
status: scrapped
type: feature
priority: normal
created_at: 2026-01-01T16:04:45Z
updated_at: 2026-01-01T16:24:17Z
---

Add an ASCII schematic diagram to preview and plot commands showing the plotter, bed, paper/drawing placement, and orientation. This helps users verify their physical setup matches the expected layout before plotting.

## Design

The diagram shows:
- Plotter device (tall narrow rectangle on left)
- Bed area (330x465mm default, portrait orientation)
- Drawing bounds (positioned at HOME, bottom-left of bed)
- Content orientation arrows (←←← stacked vertically, always pointing left toward plotter)
- HOME marker (↖)

Pages are rotated 90° CCW, so:
- Landscape drawings appear as portrait on bed (swap w/h)
- Portrait drawings appear as landscape on bed (swap w/h)

## CLI Changes

- Add `--bed-size WxH` flag (default: 465x330)
- Add `--plotter-position` flag (left, top, right, bottom - default: left)
- Add `--yes` / `-y` flag to skip confirmation on plot command
- Show diagram in both `preview` and `plot` commands
- Prompt for confirmation before plotting (unless -y)

## Example Output

```
Plotter layout (bed: 330 x 465 mm)

┌──┐
│  │
│  │ ┌─────────────────┐
│  │ │                 │
│  │ │                 │
│  │ │                 │
│  │ │                 │
│  │ │  ┌───────┐      │
│  │ │  │       │      │
│  │ │  │   ←   │      │
│  │ │  │   ←   │      │
│  │ │  │   ←   │      │
│  │ │  │       │      │
│  │ │  └───────┘      │
│  │ │  ↖ HOME         │
│  │ └─────────────────┘
└──┘

Drawing: 297 x 210 mm (landscape)
     ← = content orientation (up)
```

## Checklist

- [x] Create LayoutConfig and render functions in layout.rs
- [x] Add --bed-size flag to CLI (preview and plot commands)
- [x] Add --plotter-position flag to CLI (preview and plot commands)
- [x] Add --yes/-y flag to plot command
- [x] Integrate diagram into preview command
- [x] Integrate diagram into plot command with confirmation
- [x] Test with various drawing sizes (unit tests pass)