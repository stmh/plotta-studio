//! AxiDraw plotter control for plotta-studio
//!
//! This crate provides control for AxiDraw pen plotters.
//!
//! ## AxiDraw Protocol Notes
//!
//! The AxiDraw uses the EBB (EiBotBoard) protocol over USB serial.
//! Key commands:
//! - `SM,duration,axis1,axis2` - Stepper move
//! - `SP,value,duration` - Servo position (pen up/down)
//! - `EM,enable1,enable2` - Enable/disable motors
//! - `QP` - Query pen state
//!
//! ## Path Optimization
//!
//! For efficient plotting, paths should be optimized to minimize pen-up travel.
//! Consider implementing or using algorithms like:
//! - Greedy nearest neighbor
//! - 2-opt improvement
//! - Simulated annealing
//!
//! ## Example (future)
//!
//! ```ignore
//! use drawing_plotter::{AxiDraw, PlotConfig};
//!
//! let mut plotter = AxiDraw::connect()?;
//! plotter.plot(&drawing, &PlotConfig::default())?;
//! ```

use drawing_core::{Drawing, Point, Stroke};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlotterError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Plotter error: {0}")]
    Device(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration for plotting
#[derive(Debug, Clone)]
pub struct PlotConfig {
    /// Speed for pen-down movement (mm/s)
    pub pen_down_speed: f64,
    /// Speed for pen-up movement (mm/s)
    pub pen_up_speed: f64,
    /// Pen down position (servo units, typically 0-100)
    pub pen_down_pos: u8,
    /// Pen up position (servo units)
    pub pen_up_pos: u8,
    /// Delay after pen down (ms)
    pub pen_down_delay: u32,
    /// Delay after pen up (ms)
    pub pen_up_delay: u32,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            pen_down_speed: 25.0,
            pen_up_speed: 75.0,
            pen_down_pos: 40,
            pen_up_pos: 60,
            pen_down_delay: 150,
            pen_up_delay: 150,
        }
    }
}

/// Optimize stroke order to minimize pen-up travel distance
pub fn optimize_strokes(strokes: &[Stroke]) -> Vec<&Stroke> {
    if strokes.is_empty() {
        return vec![];
    }

    // Simple greedy nearest-neighbor algorithm
    let mut remaining: Vec<_> = strokes.iter().collect();
    let mut ordered = Vec::with_capacity(strokes.len());
    let mut current_pos = Point::ZERO;

    while !remaining.is_empty() {
        // Find nearest stroke start
        let (idx, _) = remaining
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = current_pos.distance(a.points[0]);
                let dist_b = current_pos.distance(b.points[0]);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap();

        let stroke = remaining.remove(idx);
        if let Some(last) = stroke.points.last() {
            current_pos = *last;
        }
        ordered.push(stroke);
    }

    ordered
}

/// Calculate total travel distance for a set of strokes
pub fn total_travel_distance(strokes: &[&Stroke]) -> f64 {
    let mut total = 0.0;
    let mut pos = Point::ZERO;

    for stroke in strokes {
        if stroke.points.is_empty() {
            continue;
        }

        // Pen-up travel to start
        total += pos.distance(stroke.points[0]);

        // Pen-down travel along stroke
        for pts in stroke.points.windows(2) {
            total += pts[0].distance(pts[1]);
        }

        if let Some(last) = stroke.points.last() {
            pos = *last;
        }
    }

    total
}

/// Calculate pen-down distance only
pub fn pen_down_distance(strokes: &[&Stroke]) -> f64 {
    strokes
        .iter()
        .map(|s| {
            s.points
                .windows(2)
                .map(|w| w[0].distance(w[1]))
                .sum::<f64>()
        })
        .sum()
}

// TODO: Implement actual AxiDraw communication
// This requires the `serialport` crate and EBB protocol implementation

/// Placeholder for AxiDraw connection
pub struct AxiDraw {
    // port: Box<dyn serialport::SerialPort>,
}

impl AxiDraw {
    /// List available serial ports
    pub fn list_ports() -> Result<Vec<String>, PlotterError> {
        // TODO: Use serialport::available_ports()
        Ok(vec![])
    }

    /// Connect to an AxiDraw
    pub fn connect(_port: &str) -> Result<Self, PlotterError> {
        Err(PlotterError::Connection(
            "AxiDraw connection not yet implemented".into(),
        ))
    }

    /// Plot a drawing
    pub fn plot(&mut self, _drawing: &Drawing, _config: &PlotConfig) -> Result<(), PlotterError> {
        Err(PlotterError::Device("Not implemented".into()))
    }

    /// Move pen up
    pub fn pen_up(&mut self) -> Result<(), PlotterError> {
        Err(PlotterError::Device("Not implemented".into()))
    }

    /// Move pen down
    pub fn pen_down(&mut self) -> Result<(), PlotterError> {
        Err(PlotterError::Device("Not implemented".into()))
    }

    /// Disable motors (allows manual movement)
    pub fn disable_motors(&mut self) -> Result<(), PlotterError> {
        Err(PlotterError::Device("Not implemented".into()))
    }

    /// Home the plotter
    pub fn home(&mut self) -> Result<(), PlotterError> {
        Err(PlotterError::Device("Not implemented".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_core::{Point, Style};

    #[test]
    fn test_optimize_strokes() {
        let strokes = vec![
            Stroke::line(Point::new(100.0, 100.0), Point::new(150.0, 150.0), Style::default()),
            Stroke::line(Point::new(0.0, 0.0), Point::new(50.0, 50.0), Style::default()),
            Stroke::line(Point::new(50.0, 50.0), Point::new(100.0, 100.0), Style::default()),
        ];

        let optimized = optimize_strokes(&strokes);

        // Should start with stroke closest to origin
        assert_eq!(optimized[0].points[0], Point::new(0.0, 0.0));
    }
}
