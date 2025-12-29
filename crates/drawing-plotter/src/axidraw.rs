//! AxiDraw pen plotter controller
//!
//! This module provides control for AxiDraw pen plotters using the
//! EiBotBoard (EBB) protocol over USB serial.

use crate::config::PlotConfig;
use crate::error::PlotterError;
use crate::event::{PauseControl, PlotEvent, PlotHandle};
use crate::optimize::{optimize_strokes_with_reversal, OptimizedStroke};
use drawing_core::{Drawing, Point, RenderContext, Stroke};
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

        // Small delay to let port settle after open
        std::thread::sleep(Duration::from_millis(100));

        // Flush input buffer (like pyserial's flushInput)
        axidraw
            .port
            .get_mut()
            .clear(serialport::ClearBuffer::Input)
            .ok();

        // Verify connection with version query (also consumes any pending output)
        let version = axidraw.query_version()?;
        log::info!("Connected to AxiDraw: {}", version.trim());

        // Sync state with hardware (position and pen state)
        axidraw.sync_state()?;

        Ok(axidraw)
    }

    /// Auto-connect to the first available AxiDraw
    pub fn auto_connect() -> Result<Self, PlotterError> {
        let port = Self::find_first()?;
        Self::connect(&port)
    }

    /// Set the plot configuration
    pub fn set_config(&mut self, config: PlotConfig) {
        self.config = config;
    }

    /// Query firmware version
    pub fn query_version(&mut self) -> Result<String, PlotterError> {
        self.send_command("V")
    }

    /// Query current step position from hardware
    ///
    /// Returns the current position in mm by querying the EBB's step counters.
    /// The EBB uses CoreXY kinematics where:
    /// - axis1 = X + Y (in steps)
    /// - axis2 = X - Y (in steps)
    ///
    /// This method reverses the transform to get X,Y in mm.
    ///
    /// Response format (legacy mode): `axis1,axis2<NL><CR>OK<CR><NL>`
    pub fn query_step_position(&mut self) -> Result<Point, PlotterError> {
        let response = self.send_command("QS")?;

        // QS returns data on first line, then OK on second line
        // Read and discard the OK line
        let mut ok_line = String::new();
        let _ = self.port.read_line(&mut ok_line);

        let trimmed = response.trim();

        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() < 2 {
            return Err(PlotterError::InvalidResponse(format!(
                "Expected 'axis1,axis2', got: {:?}",
                response
            )));
        }

        let axis1: i32 = parts[0].trim().parse().map_err(|_| {
            PlotterError::InvalidResponse(format!("Invalid axis1 value: {}", parts[0]))
        })?;
        let axis2: i32 = parts[1].trim().parse().map_err(|_| {
            PlotterError::InvalidResponse(format!("Invalid axis2 value: {}", parts[1]))
        })?;

        // Reverse CoreXY transform: X = (axis1 + axis2) / 2, Y = (axis1 - axis2) / 2
        // Use checked arithmetic to prevent overflow
        let steps_x = axis1.checked_add(axis2).ok_or_else(|| {
            PlotterError::InvalidResponse("axis overflow in CoreXY transform".into())
        })? / 2;
        let steps_y = axis1.checked_sub(axis2).ok_or_else(|| {
            PlotterError::InvalidResponse("axis overflow in CoreXY transform".into())
        })? / 2;

        // Convert steps to mm
        let x = steps_x as f64 / Self::STEPS_PER_MM;
        let y = steps_y as f64 / Self::STEPS_PER_MM;

        Ok(Point::new(x, y))
    }

    /// Sync internal state with hardware
    ///
    /// Queries the plotter for current position and pen state, updating
    /// the cached values. Call this after connect or if state may be out of sync.
    pub fn sync_state(&mut self) -> Result<(), PlotterError> {
        self.current_pos = self.query_step_position()?;
        self.pen_is_down = self.query_pen()?;
        log::debug!(
            "Synced state: pos=({:.2}, {:.2}), pen_down={}",
            self.current_pos.x,
            self.current_pos.y,
            self.pen_is_down
        );
        Ok(())
    }

    // ========================================================================
    // Command Protocol
    // ========================================================================

    /// Send a raw command and read the response
    ///
    /// This is a low-level method for sending EBB commands directly.
    /// Most users should use the higher-level methods like `pen_up()`, `move_to()`, etc.
    pub fn send_command(&mut self, cmd: &str) -> Result<String, PlotterError> {
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
    ///
    /// If the pen is already known to be up, this is a no-op unless `force` is true.
    /// Use `force: true` for explicit user commands where you want to ensure the
    /// pen moves regardless of cached state.
    ///
    /// The servo timing is calculated dynamically based on the pen position delta,
    /// matching the Python AxiDraw driver behavior. An optional additional delay
    /// can be configured via `pen_up_delay` in PlotConfig.
    ///
    /// Note: EBB protocol uses SP,1 for pen UP (moves to Servo_Min position)
    pub fn pen_up_with_force(&mut self, force: bool) -> Result<(), PlotterError> {
        if !force && !self.pen_is_down {
            return Ok(());
        }

        // Calculate servo move time based on pen position delta
        let servo_time = self.config.pen_up_move_time();
        let total_wait = self.config.pen_up_total_time();

        // SP,1,duration = pen UP with servo move duration
        // The duration parameter controls the servo speed (time to complete the move)
        let cmd = format!("SP,1,{}", servo_time);
        self.send_command_ok(&cmd)?;

        // Wait for servo to complete move plus any additional delay
        std::thread::sleep(Duration::from_millis(total_wait as u64));
        self.pen_is_down = false;
        Ok(())
    }

    /// Move pen up (skips if already up based on cached state)
    pub fn pen_up(&mut self) -> Result<(), PlotterError> {
        self.pen_up_with_force(false)
    }

    /// Move pen down
    ///
    /// If the pen is already known to be down, this is a no-op unless `force` is true.
    /// Use `force: true` for explicit user commands where you want to ensure the
    /// pen moves regardless of cached state.
    ///
    /// The servo timing is calculated dynamically based on the pen position delta,
    /// matching the Python AxiDraw driver behavior. An optional additional delay
    /// can be configured via `pen_down_delay` in PlotConfig.
    ///
    /// Note: EBB protocol uses SP,0 for pen DOWN (moves to Servo_Max position)
    pub fn pen_down_with_force(&mut self, force: bool) -> Result<(), PlotterError> {
        if !force && self.pen_is_down {
            return Ok(());
        }

        // Calculate servo move time based on pen position delta
        let servo_time = self.config.pen_down_move_time();
        let total_wait = self.config.pen_down_total_time();

        // SP,0,duration = pen DOWN with servo move duration
        // The duration parameter controls the servo speed (time to complete the move)
        let cmd = format!("SP,0,{}", servo_time);
        self.send_command_ok(&cmd)?;

        // Wait for servo to complete move plus any additional delay
        std::thread::sleep(Duration::from_millis(total_wait as u64));
        self.pen_is_down = true;
        Ok(())
    }

    /// Move pen down (skips if already down based on cached state)
    pub fn pen_down(&mut self) -> Result<(), PlotterError> {
        self.pen_down_with_force(false)
    }

    /// Query current pen state (true = down)
    ///
    /// Response format (legacy mode): `1<NL><CR>OK<CR><NL>` (1=up) or `0<NL><CR>OK<CR><NL>` (0=down)
    pub fn query_pen(&mut self) -> Result<bool, PlotterError> {
        let response = self.send_command("QP")?;

        // QP returns data on first line, then OK on second line
        // Read and discard the OK line
        let mut ok_line = String::new();
        let _ = self.port.read_line(&mut ok_line);

        // QP returns "1" for pen UP, "0" for pen DOWN
        // We return true if pen is DOWN
        Ok(response.trim() == "0")
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
    ///
    /// Raises the pen and moves to (0, 0). Since we sync position from
    /// hardware on connect, we have an accurate current position.
    pub fn home(&mut self) -> Result<(), PlotterError> {
        self.pen_up_with_force(true)?;
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

        // Calculate steps in X/Y coordinates (round to prevent drift from truncation)
        let steps_x = (delta.x * Self::STEPS_PER_MM).round() as i32;
        let steps_y = (delta.y * Self::STEPS_PER_MM).round() as i32;

        // Calculate duration based on speed
        let speed = if self.pen_is_down {
            self.config.pen_down_speed
        } else {
            self.config.pen_up_speed
        };
        let duration_ms = ((distance / speed) * 1000.0) as u32;
        let duration_ms = duration_ms.max(1); // Minimum 1ms

        // Use XM command for mixed-axis geometry (CoreXY/H-Bot/AxiDraw)
        // XM handles the CoreXY transform internally: axis1 = X+Y, axis2 = X-Y
        let cmd = format!("XM,{},{},{}", duration_ms, steps_x, steps_y);
        self.send_command_ok(&cmd)?;
        std::thread::sleep(Duration::from_millis(duration_ms as u64));

        self.current_pos = target;
        Ok(())
    }

    // ========================================================================
    // High-level Plotting
    // ========================================================================

    /// Plot a drawing
    pub fn plot(
        &mut self,
        drawing: &Drawing,
        config: &PlotConfig,
        ctx: &RenderContext,
    ) -> Result<(), PlotterError> {
        self.config = config.clone();

        // Flatten drawing to strokes
        let strokes = drawing.flatten(ctx);

        // Optimize stroke order with reversal support
        let optimized = optimize_strokes_with_reversal(&strokes, true);

        // Plot each stroke
        self.plot_optimized_strokes(&optimized)
    }

    /// Plot a sequence of optimized strokes (with reversal support)
    pub fn plot_optimized_strokes(
        &mut self,
        strokes: &[OptimizedStroke<'_>],
    ) -> Result<(), PlotterError> {
        self.pen_up()?;
        self.enable_motors()?;

        for opt_stroke in strokes {
            if opt_stroke.stroke.points.is_empty() {
                continue;
            }

            // Move to stroke start (pen up) - uses effective start considering reversal
            self.move_to(opt_stroke.start())?;

            // Put pen down
            self.pen_down()?;

            // Draw stroke points in correct order
            let points: Vec<_> = opt_stroke.points().collect();
            for point in points.iter().skip(1) {
                self.move_to(*point)?;
            }

            // Close if needed - for closed strokes, return to the first point we drew
            // (which is points[0] after collecting from the iterator)
            if opt_stroke.stroke.closed && points.len() > 2 {
                self.move_to(points[0])?;
            }

            // Lift pen
            self.pen_up()?;
        }

        // Return home
        self.move_to(Point::ZERO)?;
        self.disable_motors()?;

        Ok(())
    }

    /// Plot a sequence of strokes (legacy API, no reversal)
    pub fn plot_strokes(&mut self, strokes: &[&Stroke]) -> Result<(), PlotterError> {
        // Convert to OptimizedStroke without reversal
        let optimized: Vec<_> = strokes
            .iter()
            .map(|s| OptimizedStroke::new(s, false))
            .collect();
        self.plot_optimized_strokes(&optimized)
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

    /// Plot optimized strokes with event callbacks
    pub fn plot_optimized_strokes_with_events<F>(
        &mut self,
        strokes: &[OptimizedStroke<'_>],
        mut on_event: F,
    ) -> Result<(), PlotterError>
    where
        F: FnMut(PlotEvent),
    {
        self.plot_optimized_strokes_with_pause(strokes, &mut on_event, None)
    }

    /// Plot optimized strokes with event callbacks and optional pause control
    pub fn plot_optimized_strokes_with_pause<F>(
        &mut self,
        strokes: &[OptimizedStroke<'_>],
        on_event: &mut F,
        pause_control: Option<&PauseControl>,
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

        for (index, opt_stroke) in strokes.iter().enumerate() {
            // Check for pause between strokes (pen is up at this point)
            if let Some(ctrl) = pause_control {
                if ctrl.is_paused() {
                    on_event(PlotEvent::Paused);
                    ctrl.wait_if_paused();
                    on_event(PlotEvent::Resumed);
                }
            }

            if opt_stroke.stroke.points.is_empty() {
                continue;
            }

            on_event(PlotEvent::StrokeStart { index, total });

            // Move to stroke start (pen up) - uses effective start considering reversal
            let start = opt_stroke.start();
            on_event(PlotEvent::MoveTo {
                position: start,
                pen_down: false,
            });
            self.move_to(start)?;

            // Put pen down
            self.pen_down()?;

            // Draw stroke points in correct order
            let points: Vec<_> = opt_stroke.points().collect();
            for point in points.iter().skip(1) {
                on_event(PlotEvent::MoveTo {
                    position: *point,
                    pen_down: true,
                });
                self.move_to(*point)?;
            }

            // Close if needed - for closed strokes, return to the first point we drew
            if opt_stroke.stroke.closed && points.len() > 2 {
                on_event(PlotEvent::MoveTo {
                    position: points[0],
                    pen_down: true,
                });
                self.move_to(points[0])?;
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

    /// Plot strokes with event callbacks (legacy API, no reversal)
    pub fn plot_strokes_with_events<F>(
        &mut self,
        strokes: &[&Stroke],
        on_event: F,
    ) -> Result<(), PlotterError>
    where
        F: FnMut(PlotEvent),
    {
        // Convert to OptimizedStroke without reversal
        let optimized: Vec<_> = strokes
            .iter()
            .map(|s| OptimizedStroke::new(s, false))
            .collect();
        self.plot_optimized_strokes_with_events(&optimized, on_event)
    }

    /// Plot a drawing with event callbacks
    pub fn plot_with_events<F>(
        &mut self,
        drawing: &Drawing,
        config: &PlotConfig,
        ctx: &RenderContext,
        on_event: F,
    ) -> Result<(), PlotterError>
    where
        F: FnMut(PlotEvent),
    {
        self.config = config.clone();
        let strokes = drawing.flatten(ctx);
        let optimized = optimize_strokes_with_reversal(&strokes, true);
        self.plot_optimized_strokes_with_events(&optimized, on_event)
    }
}

impl Drop for AxiDraw {
    fn drop(&mut self) {
        // Note: We intentionally do NOT disable motors on drop anymore,
        // as this was causing issues with step counter resets on reconnect.
        // Users should explicitly call disable_motors() if needed.
    }
}

/// Spawn a background thread to plot a drawing
///
/// Returns a `PlotHandle` that can be used to monitor progress, pause/resume, and wait for completion.
///
/// # Example
/// ```ignore
/// use drawing_plotter::{plot_in_background, PlotConfig, PlotEvent};
/// use drawing_core::RenderContext;
///
/// let ctx = RenderContext::new();
/// let handle = plot_in_background(drawing, PlotConfig::default(), ctx, None)?;
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
/// // Pause/resume support
/// handle.pause();   // Pauses after current stroke
/// handle.resume();  // Resumes plotting
/// handle.toggle_pause(); // Toggle pause state
///
/// handle.join()?;
/// ```
pub fn plot_in_background(
    drawing: Drawing,
    config: PlotConfig,
    ctx: RenderContext,
    port: Option<String>,
) -> Result<PlotHandle, PlotterError> {
    let (sender, receiver) = mpsc::channel();
    let pause_control = PauseControl::new();
    let pause_control_clone = pause_control.clone();

    let handle = thread::spawn(move || {
        let result = (|| {
            let mut plotter = match port {
                Some(p) => AxiDraw::connect(&p)?,
                None => AxiDraw::auto_connect()?,
            };

            // Apply the config to the plotter
            plotter.set_config(config);

            let strokes = drawing.flatten(&ctx);
            let optimized = optimize_strokes_with_reversal(&strokes, true);

            plotter.plot_optimized_strokes_with_pause(
                &optimized,
                &mut |event| {
                    let _ = sender.send(event);
                },
                Some(&pause_control_clone),
            )
        })();

        if let Err(ref e) = result {
            let _ = sender.send(PlotEvent::Error(e.to_string()));
        }

        result
    });

    Ok(PlotHandle::new(receiver, handle, pause_control))
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

    #[test]
    fn test_corexy_transform() {
        // Test the CoreXY kinematics math
        // Forward: axis1 = X + Y, axis2 = X - Y
        // Reverse: X = (axis1 + axis2) / 2, Y = (axis1 - axis2) / 2

        // Test case: X=10mm, Y=5mm at 80 steps/mm
        let x_mm = 10.0;
        let y_mm = 5.0;
        let steps_x = (x_mm * AxiDraw::STEPS_PER_MM) as i32; // 800
        let steps_y = (y_mm * AxiDraw::STEPS_PER_MM) as i32; // 400

        // Forward transform (what move_to does)
        let axis1 = steps_x + steps_y; // 1200
        let axis2 = steps_x - steps_y; // 400

        // Reverse transform (what query_step_position does)
        let recovered_steps_x = (axis1 + axis2) / 2; // 800
        let recovered_steps_y = (axis1 - axis2) / 2; // 400

        assert_eq!(recovered_steps_x, steps_x);
        assert_eq!(recovered_steps_y, steps_y);

        // Convert back to mm
        let recovered_x = recovered_steps_x as f64 / AxiDraw::STEPS_PER_MM;
        let recovered_y = recovered_steps_y as f64 / AxiDraw::STEPS_PER_MM;

        assert!((recovered_x - x_mm).abs() < 0.001);
        assert!((recovered_y - y_mm).abs() < 0.001);
    }
}

/// Hardware integration tests - run manually with real AxiDraw connected
///
/// Run with: cargo test -p drawing-plotter --test '*' -- --ignored --nocapture
#[cfg(test)]
mod hardware_tests {
    use super::*;

    /// Test connecting to AxiDraw and querying state
    #[test]
    #[ignore = "requires AxiDraw hardware"]
    fn test_connect_and_query_state() {
        let plotter = AxiDraw::auto_connect().expect("Failed to connect to AxiDraw");

        let pos = plotter.position();
        let pen_down = plotter.is_pen_down();

        println!("Connected successfully!");
        println!("  Position: ({:.2}, {:.2}) mm", pos.x, pos.y);
        println!("  Pen: {}", if pen_down { "DOWN" } else { "UP" });
    }

    /// Test multiple connects to check if step position resets
    #[test]
    #[ignore = "requires AxiDraw hardware"]
    fn test_reconnect_preserves_position() {
        println!("=== First connection ===");
        {
            let plotter = AxiDraw::auto_connect().expect("Failed to connect");
            let pos = plotter.position();
            println!("Position: ({:.2}, {:.2}) mm", pos.x, pos.y);
        }

        println!("\n=== Second connection ===");
        {
            let plotter = AxiDraw::auto_connect().expect("Failed to connect");
            let pos = plotter.position();
            println!("Position: ({:.2}, {:.2}) mm", pos.x, pos.y);
        }

        println!("\n=== Third connection ===");
        {
            let plotter = AxiDraw::auto_connect().expect("Failed to connect");
            let pos = plotter.position();
            println!("Position: ({:.2}, {:.2}) mm", pos.x, pos.y);
        }

        println!("\nAll positions should match if reconnect preserves state.");
    }

    /// Test pen up command
    #[test]
    #[ignore = "requires AxiDraw hardware"]
    fn test_pen_up() {
        let mut plotter = AxiDraw::auto_connect().expect("Failed to connect");

        println!(
            "Initial pen state: {}",
            if plotter.is_pen_down() { "down" } else { "up" }
        );

        println!("Sending pen_up command...");
        plotter.pen_up_with_force(true).expect("pen_up failed");

        // Query actual state from hardware
        let pen_down = plotter.query_pen().expect("query_pen failed");
        println!(
            "Pen state after pen_up: {}",
            if pen_down { "down" } else { "up" }
        );

        assert!(!pen_down, "Pen should be UP after pen_up command");
    }

    /// Test pen down command
    #[test]
    #[ignore = "requires AxiDraw hardware"]
    fn test_pen_down() {
        let mut plotter = AxiDraw::auto_connect().expect("Failed to connect");

        println!(
            "Initial pen state: {}",
            if plotter.is_pen_down() { "down" } else { "up" }
        );

        println!("Sending pen_down command...");
        plotter.pen_down_with_force(true).expect("pen_down failed");

        // Query actual state from hardware
        let pen_down = plotter.query_pen().expect("query_pen failed");
        println!(
            "Pen state after pen_down: {}",
            if pen_down { "down" } else { "up" }
        );

        assert!(pen_down, "Pen should be DOWN after pen_down command");

        // Clean up: raise pen
        println!("Cleaning up: raising pen...");
        plotter
            .pen_up_with_force(true)
            .expect("cleanup pen_up failed");
    }

    /// Test pen toggle (up -> down -> up)
    #[test]
    #[ignore = "requires AxiDraw hardware"]
    fn test_pen_toggle() {
        let mut plotter = AxiDraw::auto_connect().expect("Failed to connect");

        // Ensure pen is up first
        println!("Step 1: Raising pen...");
        plotter.pen_up_with_force(true).expect("pen_up failed");
        let state1 = plotter.query_pen().expect("query failed");
        println!("  Pen state: {}", if state1 { "DOWN" } else { "UP" });
        assert!(!state1, "Pen should be UP");

        // Lower pen
        println!("Step 2: Lowering pen...");
        plotter.pen_down_with_force(true).expect("pen_down failed");
        let state2 = plotter.query_pen().expect("query failed");
        println!("  Pen state: {}", if state2 { "DOWN" } else { "UP" });
        assert!(state2, "Pen should be DOWN");

        // Raise pen again
        println!("Step 3: Raising pen again...");
        plotter.pen_up_with_force(true).expect("pen_up failed");
        let state3 = plotter.query_pen().expect("query failed");
        println!("  Pen state: {}", if state3 { "DOWN" } else { "UP" });
        assert!(!state3, "Pen should be UP");

        println!("Pen toggle test PASSED!");
    }

    /// Test move command
    #[test]
    #[ignore = "requires AxiDraw hardware"]
    fn test_move() {
        let mut plotter = AxiDraw::auto_connect().expect("Failed to connect");

        let start_pos = plotter
            .query_step_position()
            .expect("query position failed");
        println!(
            "Start position: ({:.2}, {:.2}) mm",
            start_pos.x, start_pos.y
        );

        // Move to a known position
        let target = Point::new(20.0, 10.0);
        println!("Moving to ({:.2}, {:.2}) mm...", target.x, target.y);
        plotter.move_to(target).expect("move_to failed");

        let end_pos = plotter
            .query_step_position()
            .expect("query position failed");
        println!("End position: ({:.2}, {:.2}) mm", end_pos.x, end_pos.y);

        // Check position is close to target (allow 0.5mm tolerance)
        let dx = (end_pos.x - target.x).abs();
        let dy = (end_pos.y - target.y).abs();
        println!("Position error: dx={:.3}, dy={:.3} mm", dx, dy);

        assert!(dx < 0.5, "X position error too large: {} mm", dx);
        assert!(dy < 0.5, "Y position error too large: {} mm", dy);

        println!("Move test passed!");
    }

    /// Test home command
    #[test]
    #[ignore = "requires AxiDraw hardware"]
    fn test_home() {
        let mut plotter = AxiDraw::auto_connect().expect("Failed to connect");

        // Print synced position from connect
        let initial_pos = plotter.position();
        println!(
            "Initial synced position: ({:.2}, {:.2}) mm",
            initial_pos.x, initial_pos.y
        );

        // Query hardware to verify
        let hw_pos = plotter.query_step_position().expect("query failed");
        println!(
            "Hardware step position: ({:.2}, {:.2}) mm",
            hw_pos.x, hw_pos.y
        );

        // First move away from origin
        println!("\nMoving TO absolute (30, 20) mm...");
        println!(
            "  Current pos: ({:.2}, {:.2})",
            plotter.position().x,
            plotter.position().y
        );
        println!("  Target: (30.0, 20.0)");
        println!(
            "  Expected delta: ({:.2}, {:.2})",
            30.0 - plotter.position().x,
            20.0 - plotter.position().y
        );

        plotter.pen_up().expect("pen_up failed");
        plotter
            .move_to(Point::new(30.0, 20.0))
            .expect("move_to failed");

        let pos_before = plotter.query_step_position().expect("query failed");
        println!(
            "Position after move (hw): ({:.2}, {:.2}) mm",
            pos_before.x, pos_before.y
        );
        println!(
            "Position after move (cached): ({:.2}, {:.2}) mm",
            plotter.position().x,
            plotter.position().y
        );

        // Home
        println!("Sending home command...");
        plotter.home().expect("home failed");

        let pos_after = plotter.query_step_position().expect("query failed");
        println!(
            "Position after home: ({:.2}, {:.2}) mm",
            pos_after.x, pos_after.y
        );

        // Check we're at origin (allow 0.5mm tolerance)
        assert!(
            pos_after.x.abs() < 0.5,
            "X should be near 0, got {:.2}",
            pos_after.x
        );
        assert!(
            pos_after.y.abs() < 0.5,
            "Y should be near 0, got {:.2}",
            pos_after.y
        );

        println!("Home test passed!");
    }

    /// Test full sequence: move, pen down, draw, pen up, home
    #[test]
    #[ignore = "requires AxiDraw hardware"]
    fn test_draw_sequence() {
        let mut plotter = AxiDraw::auto_connect().expect("Failed to connect");

        println!("=== Draw Sequence Test ===");

        // 1. Ensure pen is up and go home
        println!("1. Going home...");
        plotter.home().expect("home failed");

        // 2. Move to start position
        println!("2. Moving to (10, 10) mm...");
        plotter
            .move_to(Point::new(10.0, 10.0))
            .expect("move failed");

        // 3. Pen down
        println!("3. Pen down...");
        plotter.pen_down().expect("pen_down failed");
        assert!(
            plotter.query_pen().expect("query failed"),
            "Pen should be down"
        );

        // 4. Draw a small square
        println!("4. Drawing 10mm square...");
        plotter
            .move_to(Point::new(20.0, 10.0))
            .expect("move failed");
        plotter
            .move_to(Point::new(20.0, 20.0))
            .expect("move failed");
        plotter
            .move_to(Point::new(10.0, 20.0))
            .expect("move failed");
        plotter
            .move_to(Point::new(10.0, 10.0))
            .expect("move failed");

        // 5. Pen up
        println!("5. Pen up...");
        plotter.pen_up().expect("pen_up failed");
        assert!(
            !plotter.query_pen().expect("query failed"),
            "Pen should be up"
        );

        // 6. Home
        println!("6. Going home...");
        plotter.home().expect("home failed");

        let final_pos = plotter.query_step_position().expect("query failed");
        println!(
            "Final position: ({:.2}, {:.2}) mm",
            final_pos.x, final_pos.y
        );

        assert!(final_pos.x.abs() < 0.5, "Should be at home X");
        assert!(final_pos.y.abs() < 0.5, "Should be at home Y");

        println!("=== Draw Sequence Test PASSED ===");
    }
}
