//! AxiDraw pen plotter controller
//!
//! This module provides control for AxiDraw pen plotters using the
//! EiBotBoard (EBB) protocol over USB serial.

use crate::config::PlotConfig;
use crate::error::PlotterError;
use crate::event::{PauseControl, PlotEvent, PlotHandle};
use crate::motion::{LmCommand, MotionPlanner, MotionProfile, PlannedMove};
use crate::optimize::{optimize_strokes_with_reversal, OwnedOptimizedStroke};
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

/// Servo position constants for EBB
///
/// The EBB uses pulse width values for servo positions.
/// Standard servo range is 1ms-2ms pulse width at 50Hz.
/// EBB units are in ~0.83μs increments (1/12MHz * 10).
mod servo_constants {
    /// Minimum servo pulse width (1ms = ~7500 EBB units at standard timing)
    /// This corresponds to pen position 0 (fully up)
    pub const SERVO_MIN: u16 = 7500;

    /// Maximum servo pulse width (2ms = ~28000 EBB units at standard timing)  
    /// This corresponds to pen position 100 (fully down)
    pub const SERVO_MAX: u16 = 28000;

    /// Convert pen position (0-100) to EBB servo units
    ///
    /// Linear interpolation between SERVO_MIN and SERVO_MAX
    pub fn position_to_ebb_units(position: u8) -> u16 {
        let position = position.clamp(0, 100) as u32;
        let range = (SERVO_MAX - SERVO_MIN) as u32;
        let offset = (range * position) / 100;
        SERVO_MIN + offset as u16
    }
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
    ///
    /// This also configures the servo positions on the EBB to match
    /// the pen_up_pos and pen_down_pos settings.
    pub fn set_config(&mut self, config: PlotConfig) -> Result<(), PlotterError> {
        self.config = config;
        self.configure_servo_positions()
    }

    /// Configure servo positions and rates on the EBB
    ///
    /// Uses SC commands to set:
    /// - SC,4,<value> - Servo_Min (pen UP position)
    /// - SC,5,<value> - Servo_Max (pen DOWN position)
    /// - SC,11,<value> - Servo rate when raising (pen up)
    /// - SC,12,<value> - Servo rate when lowering (pen down)
    ///
    /// This must be called after changing pen positions or rates
    /// for the new settings to take effect.
    pub fn configure_servo_positions(&mut self) -> Result<(), PlotterError> {
        use crate::config::servo;

        let servo_min = servo_constants::position_to_ebb_units(self.config.pen_up_pos);
        let servo_max = servo_constants::position_to_ebb_units(self.config.pen_down_pos);

        // Calculate EBB rate values to match our timing formula
        let rate_up = servo::calculate_ebb_rate(
            self.config.pen_down_pos,
            self.config.pen_up_pos,
            self.config.pen_rate_raise,
        );
        let rate_down = servo::calculate_ebb_rate(
            self.config.pen_up_pos,
            self.config.pen_down_pos,
            self.config.pen_rate_lower,
        );

        log::debug!(
            "Configuring servo: pen_up={} -> SC,4,{}, pen_down={} -> SC,5,{}, rate_up={}, rate_down={}",
            self.config.pen_up_pos,
            servo_min,
            self.config.pen_down_pos,
            servo_max,
            rate_up,
            rate_down
        );

        // Set Servo_Min (pen UP position)
        let cmd = format!("SC,4,{}", servo_min);
        self.send_command_ok(&cmd)?;

        // Set Servo_Max (pen DOWN position)
        let cmd = format!("SC,5,{}", servo_max);
        self.send_command_ok(&cmd)?;

        // Set Servo rate for raising (pen up)
        let cmd = format!("SC,11,{}", rate_up);
        self.send_command_ok(&cmd)?;

        // Set Servo rate for lowering (pen down)
        let cmd = format!("SC,12,{}", rate_down);
        self.send_command_ok(&cmd)?;

        Ok(())
    }

    /// Query firmware version
    pub fn query_version(&mut self) -> Result<String, PlotterError> {
        self.send_command("V")
    }

    /// Send a raw command (for debugging/testing)
    pub fn raw_command(&mut self, cmd: &str) -> Result<String, PlotterError> {
        self.send_command(cmd)
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

    /// LM command Rate calculation constant
    /// Rate = 2^31 / 25000 * frequency_hz = 85899.3459 * frequency_hz
    /// (25000 Hz = 40μs ISR interval)
    const RATE_FACTOR: f64 = 85899.3459;

    /// Move to a position using the LM (Low-level Move) command
    ///
    /// LM provides smoother motion than XM by using precise 40μs timing intervals
    /// and allowing acceleration control (though we use constant velocity for now).
    ///
    /// LM command format: LM,Rate1,Steps1,Accel1,Rate2,Steps2,Accel2[,Clear]
    /// - Rate: step rate factor, must be positive (Rate = 85899.35 * steps_per_second)
    /// - Steps: number of steps to move (sign indicates direction in firmware 2.x)
    /// - Accel: acceleration (0 for constant velocity)
    ///
    /// Note: LM uses motor axes directly, so we need to apply CoreXY transform:
    /// - axis1 = steps_x + steps_y
    /// - axis2 = steps_x - steps_y
    pub fn move_to(&mut self, target: Point) -> Result<(), PlotterError> {
        let delta = target - self.current_pos;
        let distance = delta.length();

        if distance < 0.01 {
            return Ok(());
        }

        // Calculate steps in X/Y coordinates (round to prevent drift from truncation)
        let steps_x = (delta.x * Self::STEPS_PER_MM).round() as i32;
        let steps_y = (delta.y * Self::STEPS_PER_MM).round() as i32;

        // Apply CoreXY transform for motor axes
        // axis1 = X + Y, axis2 = X - Y
        let steps_axis1 = steps_x + steps_y;
        let steps_axis2 = steps_x - steps_y;

        // Calculate duration based on speed
        let speed = if self.pen_is_down {
            self.config.pen_down_speed
        } else {
            self.config.pen_up_speed
        };
        let duration_secs = distance / speed;
        let duration_ms = (duration_secs * 1000.0) as u32;
        let duration_ms = duration_ms.max(1); // Minimum 1ms

        // Calculate step frequencies for each axis (always positive)
        let freq_axis1 = if duration_secs > 0.0 {
            (steps_axis1.abs() as f64) / duration_secs
        } else {
            0.0
        };
        let freq_axis2 = if duration_secs > 0.0 {
            (steps_axis2.abs() as f64) / duration_secs
        } else {
            0.0
        };

        // Calculate Rate values (always positive, direction is in Steps sign)
        let rate1 = (Self::RATE_FACTOR * freq_axis1).round() as u32;
        let rate2 = (Self::RATE_FACTOR * freq_axis2).round() as u32;

        // Steps carry the sign for direction (firmware 2.x requirement)
        let cmd = format!("LM,{},{},0,{},{},0", rate1, steps_axis1, rate2, steps_axis2);
        log::debug!("LM command: {} (duration: {}ms)", cmd, duration_ms);
        self.send_command_ok(&cmd)?;
        std::thread::sleep(Duration::from_millis(duration_ms as u64));

        self.current_pos = target;
        Ok(())
    }

    /// Execute an SM command
    ///
    /// SM commands use constant velocity and are the recommended approach
    /// for motion control, matching the Python AxiDraw driver.
    fn execute_sm_command(&mut self, cmd: &crate::motion::SmCommand) -> Result<(), PlotterError> {
        // Skip empty commands
        if cmd.is_empty() {
            log::trace!("Skipping empty SM command");
            return Ok(());
        }

        let cmd_str = cmd.to_command_string();
        log::trace!("SM command: {} (sleep: {}ms)", cmd_str, cmd.sleep_time_ms());
        self.send_command_ok(&cmd_str)?;

        // Sleep with buffer lead time to ensure continuous motion
        let sleep_ms = cmd.sleep_time_ms();
        if sleep_ms > 0 {
            std::thread::sleep(Duration::from_millis(sleep_ms as u64));
        }
        Ok(())
    }

    /// Execute a planned move using SM commands (recommended)
    fn execute_sm_planned_move(
        &mut self,
        planned: &crate::motion::SmPlannedMove,
    ) -> Result<(), PlotterError> {
        for cmd in &planned.commands {
            self.execute_sm_command(cmd)?;
        }
        self.current_pos = planned.end;
        Ok(())
    }

    /// Execute an LM command (legacy, kept for compatibility)
    #[allow(dead_code)]
    fn execute_lm_command(&mut self, cmd: &LmCommand) -> Result<(), PlotterError> {
        // Skip empty commands
        if cmd.is_empty() {
            log::debug!("Skipping empty LM command");
            return Ok(());
        }

        // Validate command before sending
        if !cmd.is_valid() {
            log::warn!(
                "Invalid LM command detected (steps without rate): steps1={}, rate1={}, steps2={}, rate2={}",
                cmd.steps1, cmd.rate1, cmd.steps2, cmd.rate2
            );
            // Fall back to SM command for this move, or skip if truly invalid
            // For now, skip the command to avoid EBB error
            return Ok(());
        }

        let cmd_str = cmd.to_command_string();
        log::debug!("LM command: {} (duration: {}ms)", cmd_str, cmd.duration_ms);
        self.send_command_ok(&cmd_str)?;
        std::thread::sleep(Duration::from_millis(cmd.duration_ms as u64));
        Ok(())
    }

    /// Execute a planned move (sequence of LM commands) - legacy
    #[allow(dead_code)]
    fn execute_planned_move(&mut self, planned: &PlannedMove) -> Result<(), PlotterError> {
        for cmd in &planned.commands {
            self.execute_lm_command(cmd)?;
        }
        self.current_pos = planned.end;
        Ok(())
    }

    /// Move to a position with motion planning (uses acceleration profiles)
    ///
    /// This creates a simple single-segment move with trapezoidal velocity profile
    /// using SM commands with time-slice interpolation.
    /// For multi-segment moves (like drawing a stroke), use `draw_stroke_with_planning`.
    pub fn move_to_with_planning(&mut self, target: Point) -> Result<(), PlotterError> {
        let delta = target - self.current_pos;
        let distance = delta.length();

        if distance < 0.01 {
            return Ok(());
        }

        let motion_config = self.config.motion_config();
        let speed = if self.pen_is_down {
            self.config.pen_down_speed
        } else {
            self.config.pen_up_speed
        };

        // For single-segment moves, we start and end at zero velocity
        let profile = MotionProfile::calculate(
            0.0,                            // entry velocity
            0.0,                            // exit velocity
            speed,                          // max velocity
            distance,                       // distance
            motion_config.max_acceleration, // max acceleration
        );

        // Generate SM commands using time-slice interpolation
        let planned = crate::motion::generate_sm_commands(
            &profile,
            self.current_pos,
            target,
            motion_config.steps_per_mm,
        );

        self.execute_sm_planned_move(&planned)
    }

    /// Draw a stroke with motion planning
    ///
    /// Uses the motion planner to compute optimal velocities through corners,
    /// creating smooth motion with proper acceleration/deceleration.
    /// Motion is executed using SM commands with time-slice interpolation.
    pub fn draw_stroke_with_planning(&mut self, points: &[Point]) -> Result<(), PlotterError> {
        if points.len() < 2 {
            return Ok(());
        }

        let motion_config = self.config.motion_config();
        let planner = MotionPlanner::new(motion_config.clone());

        // Plan velocities for the entire stroke
        let segments = planner.plan(points);
        if segments.is_empty() {
            return Ok(());
        }

        // Generate and execute motion profiles for each segment
        let profiles = planner.generate_profiles(&segments);

        for (segment, profile) in segments.iter().zip(profiles.iter()) {
            // Generate SM commands using time-slice interpolation
            let planned = crate::motion::generate_sm_commands(
                profile,
                segment.start,
                segment.end,
                motion_config.steps_per_mm,
            );
            self.execute_sm_planned_move(&planned)?;
        }

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
        self.set_config(config.clone())?;

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
        strokes: &[OwnedOptimizedStroke],
    ) -> Result<(), PlotterError> {
        self.pen_up()?;
        self.enable_motors()?;

        let use_motion_planning = self.config.motion_planning_enabled;

        for opt_stroke in strokes {
            if opt_stroke.is_empty() {
                continue;
            }

            // Move to stroke start (pen up) - always use constant velocity for travel
            self.move_to(opt_stroke.start())?;

            // Put pen down
            self.pen_down()?;

            // Draw stroke points in correct order
            let points: Vec<_> = opt_stroke.points_iter().collect();

            if use_motion_planning && points.len() >= 2 {
                // Use motion planning for the entire stroke
                self.draw_stroke_with_planning(&points)?;
            } else {
                // Use constant velocity for each segment
                for point in points.iter().skip(1) {
                    self.move_to(*point)?;
                }
            }

            // Close if needed - for closed strokes, return to the first point we drew
            // (which is points[0] after collecting from the iterator)
            if opt_stroke.closed && points.len() > 2 {
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
    pub fn plot_strokes(&mut self, strokes: &[Stroke]) -> Result<(), PlotterError> {
        let optimized = optimize_strokes_with_reversal(strokes, false);
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
        strokes: &[OwnedOptimizedStroke],
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
        strokes: &[OwnedOptimizedStroke],
        on_event: &mut F,
        pause_control: Option<&PauseControl>,
    ) -> Result<(), PlotterError>
    where
        F: FnMut(PlotEvent),
    {
        let total = strokes.len();
        log::info!("Starting to plot {} strokes", total);

        on_event(PlotEvent::Started {
            total_strokes: total,
        });

        log::debug!("Raising pen and enabling motors...");
        self.pen_up()?;
        self.enable_motors()?;

        // Calculate total points for logging
        let total_points: usize = strokes.iter().map(|s| s.points.len()).sum();
        log::debug!("Total points to plot: {}", total_points);

        let plot_start = std::time::Instant::now();

        for (index, opt_stroke) in strokes.iter().enumerate() {
            // Check for cancellation
            if let Some(ctrl) = pause_control {
                if ctrl.is_cancelled() {
                    // Cancel requested - raise pen, go home, and exit
                    self.pen_up()?;
                    self.move_to(Point::ZERO)?;
                    self.disable_motors()?;
                    on_event(PlotEvent::Cancelled);
                    return Ok(());
                }

                // Check for pause between strokes (pen is up at this point)
                if ctrl.is_paused() {
                    on_event(PlotEvent::Paused);
                    ctrl.wait_if_paused();
                    // Check if we were cancelled while paused
                    if ctrl.is_cancelled() {
                        self.pen_up()?;
                        self.move_to(Point::ZERO)?;
                        self.disable_motors()?;
                        on_event(PlotEvent::Cancelled);
                        return Ok(());
                    }
                    on_event(PlotEvent::Resumed);
                }
            }

            if opt_stroke.is_empty() {
                continue;
            }

            // Log progress every 100 strokes or 10% of total
            let log_interval = (total / 10).max(100);
            if index > 0 && index % log_interval == 0 {
                let elapsed = plot_start.elapsed();
                let rate = index as f64 / elapsed.as_secs_f64();
                let remaining_strokes = total - index;
                let eta_secs = remaining_strokes as f64 / rate;
                log::info!(
                    "Progress: {}/{} strokes ({:.0}%) - {:.1} strokes/s - ETA: {:.0}s",
                    index,
                    total,
                    (index as f64 / total as f64) * 100.0,
                    rate,
                    eta_secs
                );
            }

            log::trace!(
                "Stroke {}/{}: {} points, reversed={}",
                index + 1,
                total,
                opt_stroke.points.len(),
                opt_stroke.reversed
            );

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
            let points: Vec<_> = opt_stroke.points_iter().collect();

            if self.config.motion_planning_enabled && points.len() >= 2 {
                // Use motion planning for the entire stroke
                // Emit events for each point (for progress tracking)
                for point in points.iter().skip(1) {
                    on_event(PlotEvent::MoveTo {
                        position: *point,
                        pen_down: true,
                    });
                }
                self.draw_stroke_with_planning(&points)?;
            } else {
                // Use constant velocity for each segment
                for point in points.iter().skip(1) {
                    on_event(PlotEvent::MoveTo {
                        position: *point,
                        pen_down: true,
                    });
                    self.move_to(*point)?;
                }
            }

            // Close if needed - for closed strokes, return to the first point we drew
            if opt_stroke.closed && points.len() > 2 {
                on_event(PlotEvent::MoveTo {
                    position: points[0],
                    pen_down: true,
                });
                if self.config.motion_planning_enabled {
                    // For closed strokes with motion planning, we need to include
                    // the closing segment in the motion plan
                    // For now, just do a simple move
                    self.move_to(points[0])?;
                } else {
                    self.move_to(points[0])?;
                }
            }

            // Lift pen
            self.pen_up()?;

            on_event(PlotEvent::StrokeComplete { index, total });
        }

        // Return home
        log::debug!("Returning to home position...");
        on_event(PlotEvent::MoveTo {
            position: Point::ZERO,
            pen_down: false,
        });
        self.move_to(Point::ZERO)?;
        self.disable_motors()?;

        let elapsed = plot_start.elapsed();
        log::info!(
            "Plotting completed: {} strokes in {:.1}s ({:.1} strokes/s)",
            total,
            elapsed.as_secs_f64(),
            total as f64 / elapsed.as_secs_f64()
        );

        on_event(PlotEvent::Completed);
        Ok(())
    }

    /// Plot strokes with event callbacks (legacy API, no reversal)
    pub fn plot_strokes_with_events<F>(
        &mut self,
        strokes: &[Stroke],
        on_event: F,
    ) -> Result<(), PlotterError>
    where
        F: FnMut(PlotEvent),
    {
        let optimized = optimize_strokes_with_reversal(strokes, false);
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
        self.set_config(config.clone())?;
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

    log::info!("Starting background plot thread...");

    let handle = thread::spawn(move || {
        let result = (|| {
            log::debug!("Background thread: connecting to plotter...");
            let mut plotter = match port {
                Some(ref p) => {
                    log::info!("Connecting to port: {}", p);
                    AxiDraw::connect(p)?
                }
                None => {
                    log::info!("Auto-detecting plotter...");
                    AxiDraw::auto_connect()?
                }
            };

            // Apply the config to the plotter (also configures servo positions)
            log::debug!("Applying plot configuration...");
            plotter.set_config(config)?;

            log::debug!("Flattening drawing to strokes...");
            let flatten_start = std::time::Instant::now();
            let strokes = drawing.flatten(&ctx);
            log::info!(
                "Flattened drawing to {} strokes in {:.2}s",
                strokes.len(),
                flatten_start.elapsed().as_secs_f64()
            );

            log::debug!("Optimizing stroke order...");
            let optimize_start = std::time::Instant::now();
            let optimized = optimize_strokes_with_reversal(&strokes, true);
            log::info!(
                "Optimized {} strokes in {:.2}s",
                optimized.len(),
                optimize_start.elapsed().as_secs_f64()
            );

            log::info!("Starting to plot {} strokes...", optimized.len());
            plotter.plot_optimized_strokes_with_pause(
                &optimized,
                &mut |event| {
                    let _ = sender.send(event);
                },
                Some(&pause_control_clone),
            )
        })();

        if let Err(ref e) = result {
            log::error!("Plot error: {}", e);
            let _ = sender.send(PlotEvent::Error(e.to_string()));
        }

        result
    });

    Ok(PlotHandle::new(receiver, handle, pause_control))
}

use crate::prepared::PreparedDrawing;

/// Spawn a background thread to plot a prepared drawing
///
/// This is more efficient than `plot_in_background` when you already have a
/// `PreparedDrawing`, as it avoids re-flattening and re-optimizing the strokes.
///
/// # Example
/// ```ignore
/// use drawing_plotter::{plot_prepared_in_background, PlotConfig, PlotEvent, PreparedDrawing};
/// use drawing_core::RenderContext;
///
/// let ctx = RenderContext::new();
/// let prepared = PreparedDrawing::new(&drawing, &PlotConfig::default(), &ctx);
///
/// // Use prepared.stats for preview
/// println!("Estimated time: {}", prepared.stats.format_time());
///
/// // Then plot - no re-computation needed
/// let handle = plot_prepared_in_background(prepared, PlotConfig::default(), None)?;
/// handle.join()?;
/// ```
pub fn plot_prepared_in_background(
    prepared: PreparedDrawing,
    config: PlotConfig,
    port: Option<String>,
) -> Result<PlotHandle, PlotterError> {
    let (sender, receiver) = mpsc::channel();
    let pause_control = PauseControl::new();
    let pause_control_clone = pause_control.clone();

    log::info!(
        "Starting background plot thread with {} pre-optimized strokes...",
        prepared.optimized.len()
    );

    let handle = thread::spawn(move || {
        let result = (|| {
            log::debug!("Background thread: connecting to plotter...");
            let mut plotter = match port {
                Some(ref p) => {
                    log::info!("Connecting to port: {}", p);
                    AxiDraw::connect(p)?
                }
                None => {
                    log::info!("Auto-detecting plotter...");
                    AxiDraw::auto_connect()?
                }
            };

            // Apply the config to the plotter (also configures servo positions)
            log::debug!("Applying plot configuration...");
            plotter.set_config(config)?;

            // Use the pre-optimized strokes directly - no flatten or optimize needed!
            log::info!(
                "Starting to plot {} pre-optimized strokes...",
                prepared.optimized.len()
            );
            plotter.plot_optimized_strokes_with_pause(
                &prepared.optimized,
                &mut |event| {
                    let _ = sender.send(event);
                },
                Some(&pause_control_clone),
            )
        })();

        if let Err(ref e) = result {
            log::error!("Plot error: {}", e);
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
