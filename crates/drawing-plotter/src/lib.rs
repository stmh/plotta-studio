//! AxiDraw plotter control for plotta-studio
//!
//! This crate provides control for AxiDraw pen plotters.
//!
//! ## Features
//!
//! - `hardware` (default): Enables serial port communication with real AxiDraw devices.
//!   On Linux, this requires the `libudev-dev` package to be installed.
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
//! This crate includes a greedy nearest-neighbor algorithm for stroke ordering.
//!
//! ## Example
//!
//! ```ignore
//! use drawing_plotter::{AxiDraw, PlotConfig};
//!
//! let mut plotter = AxiDraw::auto_connect()?;
//! plotter.plot(&drawing, &PlotConfig::default())?;
//! ```
//!
//! ## Background Plotting
//!
//! ```ignore
//! use drawing_plotter::{plot_in_background, PlotConfig, PlotEvent};
//!
//! let handle = plot_in_background(drawing, PlotConfig::default(), None)?;
//!
//! while handle.is_running() {
//!     for event in handle.drain_events() {
//!         println!("{:?}", event);
//!     }
//! }
//!
//! handle.join()?;
//! ```

mod config;
mod error;
mod optimize;

#[cfg(feature = "hardware")]
mod axidraw;
#[cfg(feature = "hardware")]
mod event;

// Re-export public API
pub use config::PlotConfig;
pub use error::PlotterError;
pub use optimize::{optimize_strokes, pen_down_distance, total_travel_distance};

#[cfg(feature = "hardware")]
pub use axidraw::{plot_in_background, AxiDraw, PortInfo, AXIDRAW_PID, AXIDRAW_VID};
#[cfg(feature = "hardware")]
pub use event::{PlotEvent, PlotHandle};

// Re-export constants even without hardware feature for reference
#[cfg(not(feature = "hardware"))]
pub const AXIDRAW_VID: u16 = 0x04D8;
#[cfg(not(feature = "hardware"))]
pub const AXIDRAW_PID: u16 = 0xFD92;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plot_config_default() {
        let config = PlotConfig::default();
        assert!(config.pen_down_speed > 0.0);
        assert!(config.pen_up_speed > 0.0);
        assert!(config.pen_up_speed > config.pen_down_speed); // Up should be faster
        assert!(config.pen_up_pos > config.pen_down_pos);
    }

    #[test]
    fn test_plotter_error_display() {
        let err = PlotterError::Connection("port not found".to_string());
        assert!(err.to_string().contains("Connection error"));

        let err = PlotterError::Communication("write failed".to_string());
        assert!(err.to_string().contains("Communication error"));

        let err = PlotterError::Timeout;
        assert!(err.to_string().contains("Timeout"));

        let err = PlotterError::InvalidResponse("unexpected".to_string());
        assert!(err.to_string().contains("Invalid response"));

        let err = PlotterError::Device("motor stuck".to_string());
        assert!(err.to_string().contains("Plotter error"));
    }

    #[test]
    fn test_axidraw_vid_pid() {
        // EiBotBoard identifiers
        assert_eq!(AXIDRAW_VID, 0x04D8); // Microchip
        assert_eq!(AXIDRAW_PID, 0xFD92); // EiBotBoard
    }
}
