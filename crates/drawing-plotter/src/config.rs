//! Configuration for plotting

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
