//! Plotta CLI - Command-line interface for plotter control

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Maximum allowed drawing file size (10 MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
/// Maximum allowed number of strokes in a drawing
const MAX_STROKES: usize = 100_000;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use drawing_core::{Drawing, RenderContext};
use drawing_plotter::{plot_in_background, AxiDraw, DrawingStats, PlotConfig, PlotEvent};
use drawing_text::FontManager;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(name = "plotta")]
#[command(about = "CLI for plotter control", version)]
struct Cli {
    /// Serial port path (auto-detects if not provided)
    #[arg(long, global = true)]
    port: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available plotter ports
    List,

    /// Check connection to plotter
    Status,

    /// Raise the pen
    PenUp,

    /// Lower the pen
    PenDown,

    /// Send pen to home position
    Home,

    /// Move pen to X,Y position (in mm) without drawing
    Move {
        /// X coordinate in mm
        x: f64,
        /// Y coordinate in mm
        y: f64,
    },

    /// Send a raw EBB command (for debugging)
    Raw {
        /// The command to send (e.g., "V" for version, "QS" for step position)
        command: String,
    },

    /// Plot a drawing from JSON file
    Plot {
        /// Path to JSON drawing file
        file: PathBuf,

        /// Drawing speed in mm/s (pen down movement)
        #[arg(long)]
        draw_speed: Option<f64>,

        /// Travel speed in mm/s (pen up movement)
        #[arg(long)]
        travel_speed: Option<f64>,

        /// Additional delay after lowering pen (ms)
        #[arg(long)]
        pen_down_delay: Option<u32>,

        /// Additional delay after raising pen (ms)
        #[arg(long)]
        pen_up_delay: Option<u32>,

        /// Pen down position (0-100, default 30)
        #[arg(long)]
        pen_down_pos: Option<u8>,

        /// Pen up position (0-100, default 60)
        #[arg(long)]
        pen_up_pos: Option<u8>,

        /// Pen raise rate (1-100, default 75, lower=slower)
        #[arg(long)]
        pen_rate_raise: Option<u8>,

        /// Pen lower rate (1-100, default 50, lower=slower)
        #[arg(long)]
        pen_rate_lower: Option<u8>,
    },

    /// Show drawing stats without plotting
    Preview {
        /// Path to JSON drawing file
        file: PathBuf,

        /// Drawing speed in mm/s (for time estimate)
        #[arg(long)]
        draw_speed: Option<f64>,

        /// Travel speed in mm/s (for time estimate)
        #[arg(long)]
        travel_speed: Option<f64>,

        /// Additional delay after lowering pen (ms, for time estimate)
        #[arg(long)]
        pen_down_delay: Option<u32>,

        /// Additional delay after raising pen (ms, for time estimate)
        #[arg(long)]
        pen_up_delay: Option<u32>,

        /// Pen down position (0-100, default 30)
        #[arg(long)]
        pen_down_pos: Option<u8>,

        /// Pen up position (0-100, default 60)
        #[arg(long)]
        pen_up_pos: Option<u8>,

        /// Pen raise rate (1-100, default 75, lower=slower)
        #[arg(long)]
        pen_rate_raise: Option<u8>,

        /// Pen lower rate (1-100, default 50, lower=slower)
        #[arg(long)]
        pen_rate_lower: Option<u8>,
    },
}

/// Build a PlotConfig from optional CLI overrides, using defaults from PlotConfig::default()
#[allow(clippy::too_many_arguments)]
fn build_config(
    draw_speed: Option<f64>,
    travel_speed: Option<f64>,
    pen_down_delay: Option<u32>,
    pen_up_delay: Option<u32>,
    pen_down_pos: Option<u8>,
    pen_up_pos: Option<u8>,
    pen_rate_raise: Option<u8>,
    pen_rate_lower: Option<u8>,
) -> PlotConfig {
    let defaults = PlotConfig::default();
    PlotConfig {
        pen_down_speed: draw_speed.unwrap_or(defaults.pen_down_speed),
        pen_up_speed: travel_speed.unwrap_or(defaults.pen_up_speed),
        pen_down_delay: pen_down_delay.unwrap_or(defaults.pen_down_delay),
        pen_up_delay: pen_up_delay.unwrap_or(defaults.pen_up_delay),
        pen_down_pos: pen_down_pos.unwrap_or(defaults.pen_down_pos),
        pen_up_pos: pen_up_pos.unwrap_or(defaults.pen_up_pos),
        pen_rate_raise: pen_rate_raise.unwrap_or(defaults.pen_rate_raise),
        pen_rate_lower: pen_rate_lower.unwrap_or(defaults.pen_rate_lower),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let log_level = if cli.verbose { "debug" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    match cli.command {
        Commands::List => cmd_list(),
        Commands::Status => cmd_status(cli.port.as_deref()),
        Commands::PenUp => cmd_pen_up(cli.port.as_deref()),
        Commands::PenDown => cmd_pen_down(cli.port.as_deref()),
        Commands::Home => cmd_home(cli.port.as_deref()),
        Commands::Move { x, y } => cmd_move(cli.port.as_deref(), x, y),
        Commands::Raw { command } => cmd_raw(cli.port.as_deref(), &command),
        Commands::Plot {
            file,
            draw_speed,
            travel_speed,
            pen_down_delay,
            pen_up_delay,
            pen_down_pos,
            pen_up_pos,
            pen_rate_raise,
            pen_rate_lower,
        } => {
            let config = build_config(
                draw_speed,
                travel_speed,
                pen_down_delay,
                pen_up_delay,
                pen_down_pos,
                pen_up_pos,
                pen_rate_raise,
                pen_rate_lower,
            );
            cmd_plot(cli.port.as_deref(), &file, config)
        }
        Commands::Preview {
            file,
            draw_speed,
            travel_speed,
            pen_down_delay,
            pen_up_delay,
            pen_down_pos,
            pen_up_pos,
            pen_rate_raise,
            pen_rate_lower,
        } => {
            let config = build_config(
                draw_speed,
                travel_speed,
                pen_down_delay,
                pen_up_delay,
                pen_down_pos,
                pen_up_pos,
                pen_rate_raise,
                pen_rate_lower,
            );
            cmd_preview(&file, config)
        }
    }
}

/// Validate and load a drawing file with size limits
fn load_drawing_with_validation(file: &PathBuf) -> Result<Drawing> {
    // Check file size before loading
    let metadata = fs::metadata(file)
        .with_context(|| format!("Failed to read file metadata: {}", file.display()))?;

    if metadata.len() > MAX_FILE_SIZE {
        anyhow::bail!(
            "Drawing file too large: {} bytes (max {} MB)",
            metadata.len(),
            MAX_FILE_SIZE / 1024 / 1024
        );
    }

    let drawing = Drawing::load(file)
        .with_context(|| format!("Failed to load drawing from {}", file.display()))?;

    Ok(drawing)
}

/// Validate stroke count after flattening
fn validate_stroke_count(stroke_count: usize) -> Result<()> {
    if stroke_count > MAX_STROKES {
        anyhow::bail!("Too many strokes: {} (max {})", stroke_count, MAX_STROKES);
    }
    Ok(())
}

/// Create a RenderContext with all built-in fonts loaded
fn create_render_context() -> Result<RenderContext> {
    let font_manager = FontManager::new();

    // Load all built-in fonts (Hershey + ReliefSingleLine)
    if let Err(e) = font_manager.load_all_builtin() {
        log::warn!("Failed to load built-in fonts: {}", e);
    }

    Ok(RenderContext::new(font_manager.registry().clone()))
}

/// Validate serial port path format
fn validate_port_path(port: &str) -> Result<()> {
    // Basic validation: port should look like a device path
    #[cfg(unix)]
    {
        if !port.starts_with("/dev/") {
            anyhow::bail!(
                "Invalid port path: '{}'. Expected path starting with /dev/ (e.g., /dev/ttyUSB0)",
                port
            );
        }
    }
    #[cfg(windows)]
    {
        let upper = port.to_uppercase();
        if !upper.starts_with("COM") {
            anyhow::bail!(
                "Invalid port path: '{}'. Expected COM port (e.g., COM3)",
                port
            );
        }
    }
    Ok(())
}

/// Connect to plotter, using specified port or auto-detecting
fn connect_plotter(port: Option<&str>) -> Result<AxiDraw> {
    match port {
        Some(p) => {
            validate_port_path(p)?;
            AxiDraw::connect(p).with_context(|| format!("Failed to connect to {}", p))
        }
        None => AxiDraw::auto_connect().context("Failed to auto-connect to AxiDraw"),
    }
}

/// List available plotter ports
fn cmd_list() -> Result<()> {
    let ports = AxiDraw::list_ports_detailed()?;

    if ports.is_empty() {
        println!("No USB serial ports found.");
        return Ok(());
    }

    println!("Available ports:");
    for port in ports {
        let marker = if port.is_axidraw { " [AxiDraw]" } else { "" };
        let product = port.product.as_deref().unwrap_or("Unknown");
        println!("  {} - {}{}", port.name, product, marker);
    }

    Ok(())
}

/// Check connection status
fn cmd_status(port: Option<&str>) -> Result<()> {
    let mut plotter = connect_plotter(port)?;

    // Query firmware version
    let version = plotter
        .query_version()
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Position and pen state are synced from hardware on connect
    let pen_down = plotter.is_pen_down();
    let pos = plotter.position();

    // Get the default config to show timing info
    let config = PlotConfig::default();

    println!("Connected to AxiDraw");
    println!("  Firmware: {}", version);
    println!("  Position: ({:.2}, {:.2}) mm", pos.x, pos.y);
    println!("  Pen: {}", if pen_down { "down" } else { "up" });
    println!();
    println!("Pen configuration (defaults):");
    println!(
        "  Positions: down={}, up={} (delta={}%)",
        config.pen_down_pos,
        config.pen_up_pos,
        (config.pen_up_pos as i16 - config.pen_down_pos as i16).abs()
    );
    println!(
        "  Rates: raise={}, lower={} (1-100, 100=fastest)",
        config.pen_rate_raise, config.pen_rate_lower
    );
    println!(
        "  Calculated timing: up={}ms, down={}ms",
        config.pen_up_move_time(),
        config.pen_down_move_time()
    );
    println!(
        "  Additional delays: up={}ms, down={}ms",
        config.pen_up_delay, config.pen_down_delay
    );
    println!(
        "  Total wait time: up={}ms, down={}ms",
        config.pen_up_total_time(),
        config.pen_down_total_time()
    );

    Ok(())
}

/// Raise the pen
fn cmd_pen_up(port: Option<&str>) -> Result<()> {
    let mut plotter = connect_plotter(port)?;
    plotter.pen_up_with_force(true)?;
    println!("Pen raised");
    Ok(())
}

/// Lower the pen
fn cmd_pen_down(port: Option<&str>) -> Result<()> {
    let mut plotter = connect_plotter(port)?;
    plotter.pen_down_with_force(true)?;
    println!("Pen lowered");
    Ok(())
}

/// Move to home position
fn cmd_home(port: Option<&str>) -> Result<()> {
    let mut plotter = connect_plotter(port)?;
    plotter.home()?;
    println!("Moved to home position (0, 0)");
    Ok(())
}

/// Move to specified X,Y position
fn cmd_move(port: Option<&str>, x: f64, y: f64) -> Result<()> {
    let mut plotter = connect_plotter(port)?;
    plotter.pen_up()?;
    plotter.move_to(drawing_core::Point::new(x, y))?;
    println!("Moved to ({:.1}, {:.1}) mm", x, y);
    Ok(())
}

/// Send a raw EBB command
fn cmd_raw(port: Option<&str>, command: &str) -> Result<()> {
    let mut plotter = connect_plotter(port)?;
    println!("Sending: {}", command);
    match plotter.raw_command(command) {
        Ok(response) => {
            println!("Response: {}", response.trim());
            Ok(())
        }
        Err(e) => {
            println!("Error: {}", e);
            Err(e.into())
        }
    }
}

/// Preview a drawing without plotting
fn cmd_preview(file: &PathBuf, config: PlotConfig) -> Result<()> {
    let drawing = load_drawing_with_validation(file)?;

    let ctx = create_render_context()?;
    let strokes = drawing.flatten(&ctx);
    validate_stroke_count(strokes.len())?;
    let stats = DrawingStats::calculate(&strokes, &config);

    println!("Drawing: {}", file.display());
    println!("  Size: {:.0} x {:.0} mm", drawing.width, drawing.height);
    println!(
        "  Strokes: {} ({} reversed for shorter travel)",
        stats.stroke_count, stats.reversed_strokes
    );
    println!("  Pen-down distance: {:.1} mm", stats.pen_down_distance);
    println!("  Travel distance: {:.1} mm", stats.travel_distance);
    println!(
        "  Speed: draw={} mm/s, travel={} mm/s",
        config.pen_down_speed, config.pen_up_speed
    );
    println!(
        "  Servo timing: down={}ms, up={}ms (+ {}ms/{}ms extra delay)",
        config.pen_down_move_time(),
        config.pen_up_move_time(),
        config.pen_down_delay,
        config.pen_up_delay
    );
    println!("  Estimated time: {}", stats.format_time());

    Ok(())
}

/// Check for space key press (non-blocking)
fn check_for_space_key() -> bool {
    // Poll for events with a short timeout
    if event::poll(Duration::from_millis(0)).unwrap_or(false) {
        if let Ok(Event::Key(key_event)) = event::read() {
            // Only respond to key press events (not release)
            if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char(' ') {
                return true;
            }
        }
    }
    false
}

/// Plot a drawing with progress bar
fn cmd_plot(port: Option<&str>, file: &PathBuf, config: PlotConfig) -> Result<()> {
    let drawing = load_drawing_with_validation(file)?;

    let ctx = create_render_context()?;
    let strokes = drawing.flatten(&ctx);
    validate_stroke_count(strokes.len())?;
    let stats = DrawingStats::calculate(&strokes, &config);

    println!("Plotting: {}", file.display());
    println!(
        "  {} strokes, estimated time: {}",
        stats.stroke_count,
        stats.format_time()
    );
    println!(
        "  Speed: draw={} mm/s, travel={} mm/s",
        config.pen_down_speed, config.pen_up_speed
    );
    println!(
        "  Servo timing: down={}ms, up={}ms (+ {}ms/{}ms extra delay)",
        config.pen_down_move_time(),
        config.pen_up_move_time(),
        config.pen_down_delay,
        config.pen_up_delay
    );
    println!("  Press SPACE to pause/resume");
    println!();

    let handle = plot_in_background(drawing, config, ctx, port.map(String::from))?;

    let progress = ProgressBar::new(stats.stroke_count as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40}] {pos}/{len} strokes ({percent}%)")
            .unwrap()
            .progress_chars("##-"),
    );
    progress.set_message("Plotting");

    // Enable raw mode for keyboard input
    terminal::enable_raw_mode()?;

    let result = (|| -> Result<()> {
        while handle.is_running() {
            // Check for space key to toggle pause
            if check_for_space_key() {
                let is_paused = handle.toggle_pause();
                if is_paused {
                    progress.set_message("PAUSED (space to resume)");
                } else {
                    progress.set_message("Plotting");
                }
            }

            // Process plot events
            for event in handle.drain_events() {
                match event {
                    PlotEvent::Started { total_strokes } => {
                        progress.set_length(total_strokes as u64);
                    }
                    PlotEvent::StrokeComplete { index, .. } => {
                        progress.set_position((index + 1) as u64);
                    }
                    PlotEvent::Paused => {
                        progress.set_message("PAUSED (space to resume)");
                    }
                    PlotEvent::Resumed => {
                        progress.set_message("Plotting");
                    }
                    PlotEvent::Completed => {
                        progress.finish_with_message("Done");
                    }
                    PlotEvent::Error(msg) => {
                        progress.abandon_with_message(format!("Error: {}", msg));
                        return Err(anyhow::anyhow!("Plotting failed: {}", msg));
                    }
                    _ => {}
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    })();

    // Always restore terminal state
    terminal::disable_raw_mode()?;

    result?;
    handle.join()?;
    println!("\nPlotting complete!");

    Ok(())
}
