---
# plotta-studio-a9wi
title: Add PreparedDrawing struct to cache flatten/optimize results
status: todo
type: feature
priority: high
created_at: 2025-12-30T18:02:41Z
updated_at: 2025-12-30T18:02:41Z
---

Create a 'variable bag' pattern to avoid duplicate computation when plotting.

## Problem

Currently, when plotting a drawing:
1. CLI flattens the drawing to strokes
2. CLI calls `DrawingStats::calculate()` which optimizes strokes internally
3. `plot_in_background()` re-flattens the drawing AND re-optimizes strokes

This means both flattening and optimization happen twice, which is wasteful - especially for large drawings (100k+ strokes).

## Solution

Create a `PreparedDrawing` struct that acts as a 'pipeline context' - accumulating computed results as the drawing flows through the pipeline.

## Design

```rust
/// A drawing that has been prepared for plotting.
/// Contains cached intermediate results to avoid recomputation.
pub struct PreparedDrawing {
    /// Original drawing dimensions
    pub width: f64,
    pub height: f64,
    
    /// Flattened strokes (computed once)
    pub strokes: Vec<Stroke>,
    
    /// Optimized stroke order (computed once, after strokes)
    pub optimized: Vec<OwnedOptimizedStroke>,
    
    /// Pre-calculated statistics (computed once, after optimization)
    pub stats: DrawingStats,
}

impl PreparedDrawing {
    /// Prepare a drawing for plotting with the given config.
    /// This flattens, optimizes, and calculates stats - all in one pass.
    pub fn new(drawing: &Drawing, config: &PlotConfig, ctx: &RenderContext) -> Self {
        // Step 1: Flatten (expensive for complex drawings)
        let strokes = drawing.flatten(ctx);
        
        // Step 2: Optimize (expensive for many strokes)  
        let optimized = optimize_strokes_with_reversal(&strokes, true);
        
        // Step 3: Calculate stats from already-optimized strokes (cheap)
        let stats = DrawingStats::from_optimized(&optimized, config);
        
        // Convert to owned for thread safety
        let optimized_owned = optimized.into_iter()
            .map(|o| o.to_owned())
            .collect();
        
        Self {
            width: drawing.width,
            height: drawing.height,
            strokes,
            optimized: optimized_owned,
            stats,
        }
    }
}
```

## Updated CLI Flow

```rust
fn cmd_plot(...) -> Result<()> {
    let drawing = load_drawing_with_validation(file)?;
    let ctx = create_render_context()?;
    
    // Single preparation step - does flatten + optimize + stats
    let prepared = PreparedDrawing::new(&drawing, &config, &ctx);
    
    // Use cached stats for display
    println!("  {} strokes, estimated time: {}", 
        prepared.stats.stroke_count, 
        prepared.stats.format_time());
    
    // Pass prepared drawing to background thread - no re-computation!
    let handle = plot_prepared_in_background(prepared, config, port)?;
    // ...
}
```

## Benefits

1. **No duplicate work** - flatten and optimize happen exactly once
2. **Clear ownership** - `PreparedDrawing` owns all the data, can be moved to thread
3. **Extensible** - easy to add more cached data (bounding box, path lengths, etc.)
4. **Self-documenting** - the struct shows what's been computed

## Checklist

- [ ] Create `OwnedOptimizedStroke` struct in optimize.rs (owns points instead of borrowing)
- [ ] Add `to_owned()` method to `OptimizedStroke`
- [ ] Create `PreparedDrawing` struct (new module or in lib.rs)
- [ ] Add `DrawingStats::from_optimized()` that calculates stats from pre-optimized strokes
- [ ] Add `plot_prepared_in_background()` function in axidraw.rs
- [ ] Update `AxiDraw::plot_owned_strokes()` to accept owned optimized strokes
- [ ] Update CLI `cmd_plot` to use `PreparedDrawing`
- [ ] Update CLI `cmd_preview` to use `PreparedDrawing` (optional, for consistency)
- [ ] Add tests for new structs
- [ ] Update documentation