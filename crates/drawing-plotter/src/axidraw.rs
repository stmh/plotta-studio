//! AxiDraw pen plotter controller
//!
//! This module provides control for AxiDraw pen plotters using the
//! EiBotBoard (EBB) protocol over USB serial.

use crate::config::PlotConfig;
use crate::error::PlotterError;
use crate::event::{PlotEvent, PlotHandle};
use crate::optimize::optimize_strokes;
use drawing_core::{Drawing, Point, Stroke};
use serialport::SerialPortType;
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// AxiDraw USB Vendor ID (Microchip)
pub const AXIDRAW_VID: u16 = 0x04D8;
/// AxiDraw USB Product ID (EiBotBoard)
pub const AXIDRAW_PID: u16 = 0xFD92;

/// Information about a serial port
#[derive(Debug, Clone)]
pub struct PortInfo {
    /// Port name (e.g., "/dev/ttyUSB0" or "COM3")
    pub name: String,
    /// Whether this port is an AxiDraw device
    pub is_axidraw: bool,
    /// Product name if available
    pub product: Option<String>,
    /// Manufacturer name if available
    pub manufacturer: Option<String>,
}

/// AxiDraw pen plotter controller
pub struct AxiDraw {
    port: BufReader<Box<dyn serialport::SerialPort>>,
    config: PlotConfig,
    current_pos: Point,
    pen_is_down: bool,
}

impl AxiDraw {
    /// Steps per mm (16 microsteps * 200 steps/rev / 40mm per rev)
    pub const STEPS_PER_MM: f64 = 80.0;

    // ========================================================================
    // Port Discovery
    // ========================================================================

    /// List all available USB serial ports
    pub fn list_ports() -> Result<Vec<String>, PlotterError> {
        let ports =
            serialport::available_ports().map_err(|e| PlotterError::Connection(e.to_string()))?;

        Ok(ports
            .into_iter()
            .filter(|p| matches!(p.port_type, SerialPortType::UsbPort(_)))
            .map(|p| p.port_name)
            .collect())
    }

    /// Find all connected AxiDraw devices by VID/PID
    pub fn find_devices() -> Result<Vec<String>, PlotterError> {
        let ports =
            serialport::available_ports().map_err(|e| PlotterError::Connection(e.to_string()))?;

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
        let ports =
            serialport::available_ports().map_err(|e| PlotterError::Connection(e.to_string()))?;

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
    // Connection
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
    // Command Protocol
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
    // Movement
    // ========================================================================

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

    // ========================================================================
    // Threaded Plotting with Events
    // ========================================================================

    /// Plot strokes with event callbacks
    pub fn plot_strokes_with_events<F>(
        &mut self,
        strokes: &[&Stroke],
        mut on_event: F,
    ) -> Result<(), PlotterError>
    where
        F: FnMut(PlotEvent),
    {
        let total = strokes.len();
        on_event(PlotEvent::Started {
            total_strokes: total,
        });

        self.pen_up()?;
        self.enable_motors()?;

        for (index, stroke) in strokes.iter().enumerate() {
            if stroke.points.is_empty() {
                continue;
            }

            on_event(PlotEvent::StrokeStart { index, total });

            // Move to stroke start (pen up)
            on_event(PlotEvent::MoveTo {
                position: stroke.points[0],
                pen_down: false,
            });
            self.move_to(stroke.points[0])?;

            // Put pen down
            self.pen_down()?;

            // Draw stroke
            for point in &stroke.points[1..] {
                on_event(PlotEvent::MoveTo {
                    position: *point,
                    pen_down: true,
                });
                self.move_to(*point)?;
            }

            // Close if needed
            if stroke.closed && stroke.points.len() > 2 {
                on_event(PlotEvent::MoveTo {
                    position: stroke.points[0],
                    pen_down: true,
                });
                self.move_to(stroke.points[0])?;
            }

            // Lift pen
            self.pen_up()?;

            on_event(PlotEvent::StrokeComplete { index, total });
        }

        // Return home
        on_event(PlotEvent::MoveTo {
            position: Point::ZERO,
            pen_down: false,
        });
        self.move_to(Point::ZERO)?;
        self.disable_motors()?;

        on_event(PlotEvent::Completed);
        Ok(())
    }

    /// Plot a drawing with event callbacks
    pub fn plot_with_events<F>(
        &mut self,
        drawing: &Drawing,
        config: &PlotConfig,
        on_event: F,
    ) -> Result<(), PlotterError>
    where
        F: FnMut(PlotEvent),
    {
        self.config = config.clone();
        let strokes = drawing.flatten();
        let optimized = optimize_strokes(&strokes);
        self.plot_strokes_with_events(&optimized, on_event)
    }
}

impl Drop for AxiDraw {
    fn drop(&mut self) {
        // Best effort: disable motors on drop
        let _ = self.disable_motors();
    }
}

/// Spawn a background thread to plot a drawing
///
/// Returns a `PlotHandle` that can be used to monitor progress and wait for completion.
///
/// # Example
/// ```ignore
/// use drawing_plotter::{plot_in_background, PlotConfig, PlotEvent};
///
/// let handle = plot_in_background(drawing, PlotConfig::default(), None)?;
///
/// // Monitor progress
/// while handle.is_running() {
///     for event in handle.drain_events() {
///         match event {
///             PlotEvent::StrokeComplete { index, total } => {
///                 println!("Stroke {}/{} complete", index + 1, total);
///             }
///             _ => {}
///         }
///     }
///     std::thread::sleep(std::time::Duration::from_millis(100));
/// }
///
/// handle.join()?;
/// ```
pub fn plot_in_background(
    drawing: Drawing,
    _config: PlotConfig,
    port: Option<String>,
) -> Result<PlotHandle, PlotterError> {
    let (sender, receiver) = mpsc::channel();

    let handle = thread::spawn(move || {
        let result = (|| {
            let mut plotter = match port {
                Some(p) => AxiDraw::connect(&p)?,
                None => AxiDraw::auto_connect()?,
            };

            let strokes = drawing.flatten();
            let optimized = optimize_strokes(&strokes);

            plotter.plot_strokes_with_events(&optimized, |event| {
                let _ = sender.send(event);
            })
        })();

        if let Err(ref e) = result {
            let _ = sender.send(PlotEvent::Error(e.to_string()));
        }

        result
    });

    Ok(PlotHandle::new(receiver, handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axidraw_vid_pid() {
        // EiBotBoard identifiers
        assert_eq!(AXIDRAW_VID, 0x04D8); // Microchip
        assert_eq!(AXIDRAW_PID, 0xFD92); // EiBotBoard
    }

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

    #[test]
    fn test_steps_per_mm() {
        // 16 microsteps * 200 steps/rev / 40mm per rev = 80
        assert!((AxiDraw::STEPS_PER_MM - 80.0).abs() < 0.001);
    }
}
