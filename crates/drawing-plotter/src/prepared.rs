//! PreparedDrawing - cached intermediate results for plotting
//!
//! This module provides a "pipeline context" pattern that accumulates
//! computed results as a drawing flows through the plotting pipeline,
//! avoiding duplicate computation.

use drawing_core::{Drawing, RenderContext, Stroke};
use std::time::Duration;

use crate::config::PlotConfig;
use crate::optimize::{
    merge_adjacent_strokes, optimize_strokes_with_reversal, pen_down_distance_optimized,
    travel_distance_optimized, OwnedOptimizedStroke,
};
use crate::stats::{estimate_plot_time_optimized, DrawingStats};

/// A drawing that has been prepared for plotting.
///
/// Contains cached intermediate results to avoid recomputation:
/// - Flattened strokes (expensive for complex drawings)
/// - Optimized stroke order (expensive for many strokes)
/// - Pre-calculated statistics
///
/// This struct owns all data and can be safely moved between threads.
#[derive(Debug, Clone)]
pub struct PreparedDrawing {
    /// Original drawing width (mm)
    pub width: f64,
    /// Original drawing height (mm)
    pub height: f64,
    /// Flattened strokes (computed once from drawing elements)
    pub strokes: Vec<Stroke>,
    /// Optimized stroke order with reversal support (computed once)
    pub optimized: Vec<OwnedOptimizedStroke>,
    /// Pre-calculated statistics
    pub stats: DrawingStats,
}

impl PreparedDrawing {
    /// Prepare a drawing for plotting.
    ///
    /// This performs all expensive operations once:
    /// 1. Flattens the drawing to strokes
    /// 2. Optimizes stroke order (with reversal)
    /// 3. Calculates statistics
    ///
    /// The resulting `PreparedDrawing` can be used for both displaying
    /// stats and for actual plotting, without any recomputation.
    pub fn new(drawing: &Drawing, config: &PlotConfig, ctx: &RenderContext) -> Self {
        log::debug!("Preparing drawing: flattening...");
        let flatten_start = std::time::Instant::now();
        let strokes = drawing.flatten(ctx);
        log::info!(
            "Flattened to {} strokes in {:.2}s",
            strokes.len(),
            flatten_start.elapsed().as_secs_f64()
        );

        log::debug!("Preparing drawing: optimizing stroke order...");
        let optimize_start = std::time::Instant::now();
        let optimized = optimize_strokes_with_reversal(&strokes, true);
        log::info!(
            "Optimized {} strokes in {:.2}s",
            optimized.len(),
            optimize_start.elapsed().as_secs_f64()
        );

        // Count reversals BEFORE merging - merging bakes reversal into points order.
        let reversed_count = optimized.iter().filter(|s| s.reversed).count();

        let fixed_tol = if config.merge_strokes {
            config.merge_tolerance
        } else {
            0.0
        };
        let width_factor = if config.connect_close_strokes {
            config.connect_distance_factor
        } else {
            0.0
        };

        let (optimized, merged_count) = if fixed_tol > 0.0 || width_factor > 0.0 {
            let merge_start = std::time::Instant::now();
            let result = merge_adjacent_strokes(&optimized, fixed_tol, width_factor);
            log::info!(
                "Merged {} adjacent strokes in {:.2}s ({} -> {} strokes; fixed_tol={}mm, width_factor={})",
                result.merged_count,
                merge_start.elapsed().as_secs_f64(),
                strokes.len(),
                result.strokes.len(),
                fixed_tol,
                width_factor,
            );
            (result.strokes, result.merged_count)
        } else {
            (optimized, 0)
        };

        // Calculate stats from already-optimized strokes
        log::debug!("Preparing drawing: calculating statistics...");
        let pen_down = pen_down_distance_optimized(&optimized);
        let travel = travel_distance_optimized(&optimized);
        let estimated_time = estimate_plot_time_optimized(&optimized, config);

        let stats = DrawingStats {
            stroke_count: optimized.len(),
            pen_down_distance: pen_down,
            travel_distance: travel,
            estimated_time,
            reversed_strokes: reversed_count,
            merged_strokes: merged_count,
        };

        log::debug!(
            "Prepared: {} strokes, pen_down={:.1}mm, travel={:.1}mm, {} reversed, {} merged",
            stats.stroke_count,
            stats.pen_down_distance,
            stats.travel_distance,
            stats.reversed_strokes,
            stats.merged_strokes
        );

        Self {
            width: drawing.width,
            height: drawing.height,
            strokes,
            optimized,
            stats,
        }
    }

    /// Get the number of strokes
    pub fn stroke_count(&self) -> usize {
        self.strokes.len()
    }

    /// Get the estimated plot time
    pub fn estimated_time(&self) -> Duration {
        self.stats.estimated_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepared_drawing_empty() {
        let drawing = Drawing::new(100.0, 100.0);
        let config = PlotConfig::default();
        let ctx = RenderContext::empty();

        let prepared = PreparedDrawing::new(&drawing, &config, &ctx);

        assert_eq!(prepared.stroke_count(), 0);
        assert_eq!(prepared.width, 100.0);
        assert_eq!(prepared.height, 100.0);
    }
}
