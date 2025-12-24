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

#[cfg(feature = "hardware")]
use drawing_core::Drawing;
use drawing_core::{Point, Stroke};
#[cfg(feature = "hardware")]
use serialport::SerialPortType;
#[cfg(feature = "hardware")]
use std::io::{BufRead, BufReader, Write};
#[cfg(feature = "hardware")]
use std::time::Duration;
use thiserror::Error;

/// AxiDraw USB identifiers (EiBotBoard)
pub const AXIDRAW_VID: u16 = 0x04D8; // Microchip
pub const AXIDRAW_PID: u16 = 0xFD92; // EiBotBoard

#[derive(Error, Debug)]
pub enum PlotterError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

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

/// Information about a serial port
#[cfg(feature = "hardware")]
#[derive(Debug, Clone)]
pub struct PortInfo {
    pub name: String,
    pub is_axidraw: bool,
    pub product: Option<String>,
    pub manufacturer: Option<String>,
}

/// AxiDraw pen plotter controller
#[cfg(feature = "hardware")]
pub struct AxiDraw {
    port: BufReader<Box<dyn serialport::SerialPort>>,
    config: PlotConfig,
    current_pos: Point,
    pen_is_down: bool,
}

#[cfg(feature = "hardware")]
impl AxiDraw {
    // ========================================================================
    // Port Discovery (axi2b)
    // ========================================================================

    /// List all available USB serial ports
    pub fn list_ports() -> Result<Vec<String>, PlotterError> {
        let ports = serialport::available_ports()
            .map_err(|e| PlotterError::Connection(e.to_string()))?;

        Ok(ports
            .into_iter()
            .filter(|p| matches!(p.port_type, SerialPortType::UsbPort(_)))
            .map(|p| p.port_name)
            .collect())
    }

    /// Find all connected AxiDraw devices by VID/PID
    pub fn find_devices() -> Result<Vec<String>, PlotterError> {
        let ports = serialport::available_ports()
            .map_err(|e| PlotterError::Connection(e.to_string()))?;

        Ok(ports
            .into_iter()
            .filter_map(|p| {
                if let SerialPortType::UsbPort(usb_info) = &p.port_type {
                    if usb_info.vid == AXIDRAW_VID && usb_info.pid == AXIDRAW_PID {
                        return Some(p.port_name);
                    }
                }
                None
            })
            .collect())
    }

    /// Find first AxiDraw device
    pub fn find_first() -> Result<String, PlotterError> {
        let devices = Self::find_devices()?;
        devices
            .into_iter()
            .next()
            .ok_or_else(|| PlotterError::Connection("No AxiDraw device found".into()))
    }

    /// Get detailed information about available USB serial ports
    pub fn list_ports_detailed() -> Result<Vec<PortInfo>, PlotterError> {
        let ports = serialport::available_ports()
            .map_err(|e| PlotterError::Connection(e.to_string()))?;

        Ok(ports
            .into_iter()
            .filter_map(|p| {
                if let SerialPortType::UsbPort(usb_info) = &p.port_type {
                    Some(PortInfo {
                        name: p.port_name,
                        is_axidraw: usb_info.vid == AXIDRAW_VID && usb_info.pid == AXIDRAW_PID,
                        product: usb_info.product.clone(),
                        manufacturer: usb_info.manufacturer.clone(),
                    })
                } else {
                    None
                }
            })
            .collect())
    }

    // ========================================================================
    // Connection (axi2c)
    // ========================================================================

    /// Connect to an AxiDraw on the specified port
    pub fn connect(port_name: &str) -> Result<Self, PlotterError> {
        let port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(1000))
            .open()
            .map_err(|e| PlotterError::Connection(format!("{}: {}", port_name, e)))?;

        let mut axidraw = Self {
            port: BufReader::new(port),
            config: PlotConfig::default(),
            current_pos: Point::ZERO,
            pen_is_down: false,
        };

        // Verify connection with version query
        let version = axidraw.query_version()?;
        log::info!("Connected to AxiDraw: {}", version.trim());

        Ok(axidraw)
    }

    /// Auto-connect to the first available AxiDraw
    pub fn auto_connect() -> Result<Self, PlotterError> {
        let port = Self::find_first()?;
        Self::connect(&port)
    }

    /// Query firmware version
    pub fn query_version(&mut self) -> Result<String, PlotterError> {
        self.send_command("V")
    }

    // ========================================================================
    // Command Protocol (axi2d)
    // ========================================================================

    /// Send a command and read the response
    fn send_command(&mut self, cmd: &str) -> Result<String, PlotterError> {
        let cmd_bytes = format!("{}\r", cmd);

        self.port
            .get_mut()
            .write_all(cmd_bytes.as_bytes())
            .map_err(|e| PlotterError::Communication(e.to_string()))?;

        self.port
            .get_mut()
            .flush()
            .map_err(|e| PlotterError::Communication(e.to_string()))?;

        // Read response line
        let mut response = String::new();
        self.port
            .read_line(&mut response)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::TimedOut => PlotterError::Timeout,
                _ => PlotterError::Communication(e.to_string()),
            })?;

        // Check for error response
        if response.starts_with('!') {
            return Err(PlotterError::Device(response.trim().to_string()));
        }

        Ok(response)
    }

    /// Send a command that returns OK
    fn send_command_ok(&mut self, cmd: &str) -> Result<(), PlotterError> {
        let response = self.send_command(cmd)?;
        if !response.trim().eq_ignore_ascii_case("OK") {
            return Err(PlotterError::InvalidResponse(response));
        }
        Ok(())
    }

    // ========================================================================
    // Pen Control
    // ========================================================================

    /// Move pen up
    pub fn pen_up(&mut self) -> Result<(), PlotterError> {
        if !self.pen_is_down {
            return Ok(());
        }
        let cmd = format!("SP,0,{}", self.config.pen_up_delay);
        self.send_command_ok(&cmd)?;
        std::thread::sleep(Duration::from_millis(self.config.pen_up_delay as u64));
        self.pen_is_down = false;
        Ok(())
    }

    /// Move pen down
    pub fn pen_down(&mut self) -> Result<(), PlotterError> {
        if self.pen_is_down {
            return Ok(());
        }
        let cmd = format!("SP,1,{}", self.config.pen_down_delay);
        self.send_command_ok(&cmd)?;
        std::thread::sleep(Duration::from_millis(self.config.pen_down_delay as u64));
        self.pen_is_down = true;
        Ok(())
    }

    /// Query current pen state (true = down)
    pub fn query_pen(&mut self) -> Result<bool, PlotterError> {
        let response = self.send_command("QP")?;
        Ok(response.trim() == "1")
    }

    // ========================================================================
    // Motor Control
    // ========================================================================

    /// Enable motors
    pub fn enable_motors(&mut self) -> Result<(), PlotterError> {
        self.send_command_ok("EM,1,1")
    }

    /// Disable motors (allows manual movement)
    pub fn disable_motors(&mut self) -> Result<(), PlotterError> {
        self.send_command_ok("EM,0,0")
    }

    /// Home the plotter (move to origin)
    pub fn home(&mut self) -> Result<(), PlotterError> {
        self.pen_up()?;
        self.move_to(Point::ZERO)?;
        Ok(())
    }

    // ========================================================================
    // Movement (from axi3, but needed for basic operation)
    // ========================================================================

    /// Steps per mm (16 microsteps * 200 steps/rev / 40mm per rev)
    const STEPS_PER_MM: f64 = 80.0;

    /// Move to a position
    pub fn move_to(&mut self, target: Point) -> Result<(), PlotterError> {
        let delta = target - self.current_pos;
        let distance = delta.length();

        if distance < 0.01 {
            return Ok(());
        }

        // Calculate steps
        let steps_x = (delta.x * Self::STEPS_PER_MM) as i32;
        let steps_y = (delta.y * Self::STEPS_PER_MM) as i32;

        // CoreXY transform: axis1 = X+Y, axis2 = X-Y
        let axis1 = steps_x + steps_y;
        let axis2 = steps_x - steps_y;

        // Calculate duration based on speed
        let speed = if self.pen_is_down {
            self.config.pen_down_speed
        } else {
            self.config.pen_up_speed
        };
        let duration_ms = ((distance / speed) * 1000.0) as u32;
        let duration_ms = duration_ms.max(1); // Minimum 1ms

        let cmd = format!("SM,{},{},{}", duration_ms, axis1, axis2);
        self.send_command_ok(&cmd)?;
        std::thread::sleep(Duration::from_millis(duration_ms as u64));

        self.current_pos = target;
        Ok(())
    }

    // ========================================================================
    // High-level Plotting
    // ========================================================================

    /// Plot a drawing
    pub fn plot(&mut self, drawing: &Drawing, config: &PlotConfig) -> Result<(), PlotterError> {
        self.config = config.clone();

        // Flatten drawing to strokes
        let strokes = drawing.flatten();

        // Optimize stroke order
        let optimized = optimize_strokes(&strokes);

        // Plot each stroke
        self.plot_strokes(&optimized)
    }

    /// Plot a sequence of strokes
    pub fn plot_strokes(&mut self, strokes: &[&Stroke]) -> Result<(), PlotterError> {
        self.pen_up()?;
        self.enable_motors()?;

        for stroke in strokes {
            if stroke.points.is_empty() {
                continue;
            }

            // Move to stroke start (pen up)
            self.move_to(stroke.points[0])?;

            // Put pen down
            self.pen_down()?;

            // Draw stroke
            for point in &stroke.points[1..] {
                self.move_to(*point)?;
            }

            // Close if needed
            if stroke.closed && stroke.points.len() > 2 {
                self.move_to(stroke.points[0])?;
            }

            // Lift pen
            self.pen_up()?;
        }

        // Return home
        self.move_to(Point::ZERO)?;
        self.disable_motors()?;

        Ok(())
    }

    /// Get current position
    pub fn position(&self) -> Point {
        self.current_pos
    }

    /// Check if pen is down
    pub fn is_pen_down(&self) -> bool {
        self.pen_is_down
    }
}

#[cfg(feature = "hardware")]
impl Drop for AxiDraw {
    fn drop(&mut self) {
        // Best effort: disable motors on drop
        let _ = self.disable_motors();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing_core::{Point, Style};

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    // ========================================================================
    // optimize_strokes tests
    // ========================================================================

    #[test]
    fn test_optimize_strokes_empty() {
        let strokes: Vec<Stroke> = vec![];
        let optimized = optimize_strokes(&strokes);
        assert!(optimized.is_empty());
    }

    #[test]
    fn test_optimize_strokes_single() {
        let strokes = vec![Stroke::line(
            Point::new(50.0, 50.0),
            Point::new(100.0, 100.0),
            Style::default(),
        )];
        let optimized = optimize_strokes(&strokes);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].points[0], Point::new(50.0, 50.0));
    }

    #[test]
    fn test_optimize_strokes_nearest_neighbor() {
        let strokes = vec![
            Stroke::line(
                Point::new(100.0, 100.0),
                Point::new(150.0, 150.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(50.0, 50.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(50.0, 50.0),
                Point::new(100.0, 100.0),
                Style::default(),
            ),
        ];

        let optimized = optimize_strokes(&strokes);

        // Should start with stroke closest to origin (0,0)
        assert_eq!(optimized[0].points[0], Point::new(0.0, 0.0));
        // Second stroke should start where first ended (50, 50)
        assert_eq!(optimized[1].points[0], Point::new(50.0, 50.0));
        // Third stroke should start where second ended (100, 100)
        assert_eq!(optimized[2].points[0], Point::new(100.0, 100.0));
    }

    #[test]
    fn test_optimize_strokes_already_optimal() {
        let strokes = vec![
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 10.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(10.0, 10.0),
                Point::new(20.0, 20.0),
                Style::default(),
            ),
        ];

        let optimized = optimize_strokes(&strokes);

        // Order should remain the same
        assert_eq!(optimized[0].points[0], Point::new(0.0, 0.0));
        assert_eq!(optimized[1].points[0], Point::new(10.0, 10.0));
    }

    #[test]
    fn test_optimize_strokes_reverse_order() {
        // Strokes in reverse order should be reordered
        let strokes = vec![
            Stroke::line(
                Point::new(100.0, 0.0),
                Point::new(110.0, 0.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(50.0, 0.0),
                Point::new(60.0, 0.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Style::default(),
            ),
        ];

        let optimized = optimize_strokes(&strokes);

        // Should be reordered to start from origin
        assert_eq!(optimized[0].points[0], Point::new(0.0, 0.0));
    }

    #[test]
    fn test_optimize_strokes_preserves_count() {
        let strokes = vec![
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 10.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(20.0, 20.0),
                Point::new(30.0, 30.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(40.0, 40.0),
                Point::new(50.0, 50.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(60.0, 60.0),
                Point::new(70.0, 70.0),
                Style::default(),
            ),
        ];

        let optimized = optimize_strokes(&strokes);
        assert_eq!(optimized.len(), strokes.len());
    }

    // ========================================================================
    // total_travel_distance tests
    // ========================================================================

    #[test]
    fn test_total_travel_distance_empty() {
        let strokes: Vec<&Stroke> = vec![];
        assert!(approx_eq(total_travel_distance(&strokes), 0.0));
    }

    #[test]
    fn test_total_travel_distance_single_stroke() {
        let stroke = Stroke::line(Point::new(0.0, 0.0), Point::new(3.0, 4.0), Style::default());
        let strokes = vec![&stroke];
        // Pen-up from origin (0) + pen-down distance (5) = 5
        assert!(approx_eq(total_travel_distance(&strokes), 5.0));
    }

    #[test]
    fn test_total_travel_distance_includes_pen_up() {
        let stroke = Stroke::line(
            Point::new(10.0, 0.0), // 10 units from origin
            Point::new(13.0, 4.0), // 5 unit stroke (3-4-5)
            Style::default(),
        );
        let strokes = vec![&stroke];
        // Pen-up travel (10) + pen-down travel (5) = 15
        assert!(approx_eq(total_travel_distance(&strokes), 15.0));
    }

    #[test]
    fn test_total_travel_distance_multiple_strokes() {
        let stroke1 = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Style::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(10.0, 0.0), // No pen-up travel from stroke1 end
            Point::new(20.0, 0.0),
            Style::default(),
        );
        let strokes = vec![&stroke1, &stroke2];
        // Pen-up from origin (0) + stroke1 (10) + pen-up (0) + stroke2 (10) = 20
        assert!(approx_eq(total_travel_distance(&strokes), 20.0));
    }

    #[test]
    fn test_total_travel_distance_with_gap() {
        let stroke1 = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Style::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(20.0, 0.0), // 10 units gap
            Point::new(30.0, 0.0),
            Style::default(),
        );
        let strokes = vec![&stroke1, &stroke2];
        // Pen-up from origin (0) + stroke1 (10) + pen-up (10) + stroke2 (10) = 30
        assert!(approx_eq(total_travel_distance(&strokes), 30.0));
    }

    #[test]
    fn test_total_travel_distance_multi_point_stroke() {
        let stroke = Stroke::new(
            vec![
                Point::new(0.0, 0.0),
                Point::new(3.0, 4.0), // 5 units
                Point::new(3.0, 0.0), // 4 units
            ],
            Style::default(),
        );
        let strokes = vec![&stroke];
        // Pen-up from origin (0) + 5 + 4 = 9
        assert!(approx_eq(total_travel_distance(&strokes), 9.0));
    }

    // ========================================================================
    // pen_down_distance tests
    // ========================================================================

    #[test]
    fn test_pen_down_distance_empty() {
        let strokes: Vec<&Stroke> = vec![];
        assert!(approx_eq(pen_down_distance(&strokes), 0.0));
    }

    #[test]
    fn test_pen_down_distance_single_stroke() {
        let stroke = Stroke::line(
            Point::new(100.0, 100.0), // Far from origin
            Point::new(103.0, 104.0), // 5 unit stroke
            Style::default(),
        );
        let strokes = vec![&stroke];
        // Only pen-down distance, ignores position
        assert!(approx_eq(pen_down_distance(&strokes), 5.0));
    }

    #[test]
    fn test_pen_down_distance_multiple_strokes() {
        let stroke1 = Stroke::line(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Style::default(),
        );
        let stroke2 = Stroke::line(
            Point::new(100.0, 100.0), // Position doesn't matter
            Point::new(100.0, 120.0), // 20 units
            Style::default(),
        );
        let strokes = vec![&stroke1, &stroke2];
        assert!(approx_eq(pen_down_distance(&strokes), 30.0));
    }

    #[test]
    fn test_pen_down_distance_multi_point_stroke() {
        let stroke = Stroke::new(
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),  // 10 units
                Point::new(10.0, 10.0), // 10 units
                Point::new(0.0, 10.0),  // 10 units
            ],
            Style::default(),
        );
        let strokes = vec![&stroke];
        assert!(approx_eq(pen_down_distance(&strokes), 30.0));
    }

    #[test]
    fn test_pen_down_distance_ignores_pen_up_travel() {
        // Two strokes far apart
        let stroke1 = Stroke::line(Point::new(0.0, 0.0), Point::new(5.0, 0.0), Style::default());
        let stroke2 = Stroke::line(
            Point::new(1000.0, 1000.0), // Very far away
            Point::new(1005.0, 1000.0), // 5 units
            Style::default(),
        );
        let strokes = vec![&stroke1, &stroke2];
        // Should only count pen-down: 5 + 5 = 10
        assert!(approx_eq(pen_down_distance(&strokes), 10.0));
    }

    // ========================================================================
    // Optimization verification tests
    // ========================================================================

    #[test]
    fn test_optimized_has_less_or_equal_travel() {
        let strokes = vec![
            Stroke::line(
                Point::new(100.0, 0.0),
                Point::new(110.0, 0.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(50.0, 0.0),
                Point::new(60.0, 0.0),
                Style::default(),
            ),
        ];

        let unoptimized: Vec<_> = strokes.iter().collect();
        let optimized = optimize_strokes(&strokes);

        let unoptimized_distance = total_travel_distance(&unoptimized);
        let optimized_distance = total_travel_distance(&optimized);

        // Optimized should have less or equal travel distance
        assert!(optimized_distance <= unoptimized_distance);
    }

    #[test]
    fn test_optimization_preserves_pen_down_distance() {
        let strokes = vec![
            Stroke::line(
                Point::new(100.0, 0.0),
                Point::new(110.0, 0.0),
                Style::default(),
            ),
            Stroke::line(
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Style::default(),
            ),
        ];

        let unoptimized: Vec<_> = strokes.iter().collect();
        let optimized = optimize_strokes(&strokes);

        // Pen-down distance should be the same regardless of order
        assert!(approx_eq(
            pen_down_distance(&unoptimized),
            pen_down_distance(&optimized)
        ));
    }

    // ========================================================================
    // PlotConfig tests
    // ========================================================================

    #[test]
    fn test_plot_config_default() {
        let config = PlotConfig::default();
        assert!(config.pen_down_speed > 0.0);
        assert!(config.pen_up_speed > 0.0);
        assert!(config.pen_up_speed > config.pen_down_speed); // Up should be faster
        assert!(config.pen_up_pos > config.pen_down_pos);
    }

    // ========================================================================
    // AxiDraw USB identifiers tests
    // ========================================================================

    #[test]
    fn test_axidraw_vid_pid() {
        // EiBotBoard identifiers
        assert_eq!(AXIDRAW_VID, 0x04D8); // Microchip
        assert_eq!(AXIDRAW_PID, 0xFD92); // EiBotBoard
    }

    // ========================================================================
    // PortInfo tests (hardware feature only)
    // ========================================================================

    #[cfg(feature = "hardware")]
    #[test]
    fn test_port_info_creation() {
        let info = PortInfo {
            name: "/dev/ttyUSB0".to_string(),
            is_axidraw: true,
            product: Some("EiBotBoard".to_string()),
            manufacturer: Some("Microchip".to_string()),
        };
        assert_eq!(info.name, "/dev/ttyUSB0");
        assert!(info.is_axidraw);
        assert_eq!(info.product, Some("EiBotBoard".to_string()));
    }

    #[cfg(feature = "hardware")]
    #[test]
    fn test_port_info_clone() {
        let info = PortInfo {
            name: "/dev/ttyACM0".to_string(),
            is_axidraw: false,
            product: None,
            manufacturer: None,
        };
        let cloned = info.clone();
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.is_axidraw, cloned.is_axidraw);
    }

    // ========================================================================
    // PlotterError tests
    // ========================================================================

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

    // ========================================================================
    // AxiDraw constants tests (hardware feature only)
    // ========================================================================

    #[cfg(feature = "hardware")]
    #[test]
    fn test_steps_per_mm() {
        // 16 microsteps * 200 steps/rev / 40mm per rev = 80
        assert!((AxiDraw::STEPS_PER_MM - 80.0).abs() < 0.001);
    }
}
