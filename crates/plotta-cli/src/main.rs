//! Plotta CLI - Command-line interface for plotter control

use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use plotta_cli::setup_diagram::{PlotterSetup, SetupDiagram};

/// Maximum allowed drawing file size (10 MB)
const MAX_FILE_SIZE: u64 = 500 * 1024 * 1024;
/// Maximum allowed number of strokes in a drawing
const MAX_STROKES: usize = 1_000_000;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use drawing_core::{Drawing, RenderContext};
use drawing_plotter::{
    plot_prepared_in_background, AxiDraw, PlotConfig, PlotEvent, PreparedDrawing,
};
use drawing_svg::{record_strokes_to_svg, RecordOptions};
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

        /// Plotter physical setup: top, bottom, left, right (default: top)
        /// Can also be set via PLOTTA_PLOTTER_SETUP environment variable
        #[arg(short = 's', long, env = "PLOTTA_PLOTTER_SETUP", default_value = "top")]
        plotter_setup: String,

        /// Skip confirmation prompt and start plotting immediately
        #[arg(short = 'y', long)]
        yes: bool,

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

        /// Enable position verification (diagnostic mode, adds latency)
        #[arg(long)]
        verify_position: bool,
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

        /// Enable position verification flag (accepted for API consistency, no effect in preview)
        #[arg(long)]
        verify_position: bool,
    },

    /// Record plot to SVG file (simulate without hardware)
    Record {
        /// Path to JSON drawing file
        file: PathBuf,

        /// Output SVG file path
        #[arg(short, long)]
        output: PathBuf,

        /// Show pen-up travel paths as dashed lines
        #[arg(long)]
        show_travel: bool,

        /// Show direction arrows at stroke starts
        #[arg(long)]
        show_direction: bool,

        /// Stroke width in mm (default: 0.3)
        #[arg(long, default_value = "0.3")]
        stroke_width: f64,

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
    verify_position: bool,
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
        // Motion planning settings (use defaults for now, can add CLI flags later)
        max_acceleration: defaults.max_acceleration,
        junction_deviation: defaults.junction_deviation,
        motion_planning_enabled: defaults.motion_planning_enabled,
        // Position verification (diagnostic mode)
        verify_position,
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
            plotter_setup,
            yes,
            draw_speed,
            travel_speed,
            pen_down_delay,
            pen_up_delay,
            pen_down_pos,
            pen_up_pos,
            pen_rate_raise,
            pen_rate_lower,
            verify_position,
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
                verify_position,
            );
            let setup = plotter_setup
                .parse::<PlotterSetup>()
                .map_err(|e| anyhow::anyhow!(e))?;
            cmd_plot(cli.port.as_deref(), &file, config, setup, yes)
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
            verify_position,
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
                verify_position,
            );
            cmd_preview(&file, config)
        }
        Commands::Record {
            file,
            output,
            show_travel,
            show_direction,
            stroke_width,
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
                false, // verify_position not applicable for Record
            );
            let record_options = RecordOptions {
                show_travel,
                show_direction,
                stroke_width,
                ..Default::default()
            };
            cmd_record(&file, &output, config, record_options)
        }
    }
}

/// Validate and load a drawing file with size limits
fn load_drawing_with_validation(file: &PathBuf) -> Result<Drawing> {
    // Check file size before loading
    let metadata = fs::metadata(file)
        .with_context(|| format!("Failed to read file metadata: {}", file.display()))?;

    let file_size = metadata.len();
    log::info!(
        "Loading drawing file: {} ({:.2} MB)",
        file.display(),
        file_size as f64 / 1024.0 / 1024.0
    );

    if file_size > MAX_FILE_SIZE {
        anyhow::bail!(
            "Drawing file too large: {} bytes (max {} MB)",
            file_size,
            MAX_FILE_SIZE / 1024 / 1024
        );
    }

    log::debug!("Parsing JSON drawing file...");
    let start = Instant::now();
    let drawing = Drawing::load(file)
        .with_context(|| format!("Failed to load drawing from {}", file.display()))?;
    log::info!(
        "Drawing loaded in {:.2}s: {} elements, {:.0}x{:.0} mm",
        start.elapsed().as_secs_f64(),
        drawing.elements.len(),
        drawing.width,
        drawing.height
    );

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

    // Prepare the drawing - flattens, optimizes, and calculates stats
    log::info!("Preparing drawing...");
    let prepare_start = Instant::now();
    let prepared = PreparedDrawing::new(&drawing, &config, &ctx);
    log::info!(
        "Drawing prepared in {:.2}s",
        prepare_start.elapsed().as_secs_f64()
    );

    validate_stroke_count(prepared.stroke_count())?;

    let stats = &prepared.stats;

    println!("Drawing: {}", file.display());
    println!("  Size: {:.0} x {:.0} mm", prepared.width, prepared.height);
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
    println!(
        "  Motion planning: {}",
        if config.motion_planning_enabled {
            "enabled (smooth acceleration)"
        } else {
            "disabled (constant velocity)"
        }
    );
    println!("  Estimated time: {}", stats.format_time());

    Ok(())
}

/// Record a drawing to SVG file (simulate plotting without hardware)
fn cmd_record(
    file: &PathBuf,
    output: &PathBuf,
    config: PlotConfig,
    record_options: RecordOptions,
) -> Result<()> {
    let drawing = load_drawing_with_validation(file)?;
    let ctx = create_render_context()?;

    // Prepare the drawing - flattens, optimizes, and calculates stats
    log::info!("Preparing drawing...");
    let prepare_start = Instant::now();
    let prepared = PreparedDrawing::new(&drawing, &config, &ctx);
    log::info!(
        "Drawing prepared in {:.2}s",
        prepare_start.elapsed().as_secs_f64()
    );

    validate_stroke_count(prepared.stroke_count())?;

    // Generate SVG from optimized strokes
    log::info!("Generating SVG...");
    let svg = record_strokes_to_svg(
        &prepared.optimized,
        prepared.width,
        prepared.height,
        &record_options,
    );

    // Write to file
    fs::write(output, &svg)
        .with_context(|| format!("Failed to write SVG to {}", output.display()))?;

    let stats = &prepared.stats;

    println!("Recorded to: {}", output.display());
    println!("  Size: {:.0} x {:.0} mm", prepared.width, prepared.height);
    println!(
        "  Strokes: {} ({} reversed for shorter travel)",
        stats.stroke_count, stats.reversed_strokes
    );
    println!("  Pen-down distance: {:.1} mm", stats.pen_down_distance);
    println!("  Travel distance: {:.1} mm", stats.travel_distance);
    println!("  Estimated plot time: {}", stats.format_time());
    if record_options.show_travel {
        println!("  Travel lines: shown (dashed)");
    }
    if record_options.show_direction {
        println!("  Direction arrows: shown");
    }

    Ok(())
}

/// Key input result
enum KeyInput {
    /// No key pressed
    None,
    /// Space key - toggle pause
    Space,
    /// Q key or Ctrl+C - cancel
    Cancel,
}

/// Check for key press (non-blocking)
fn check_for_key() -> KeyInput {
    // Poll for events with a short timeout
    if event::poll(Duration::from_millis(0)).unwrap_or(false) {
        if let Ok(Event::Key(key_event)) = event::read() {
            // Only respond to key press events (not release)
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Char(' ') => return KeyInput::Space,
                    KeyCode::Char('q') | KeyCode::Char('Q') => return KeyInput::Cancel,
                    KeyCode::Char('c')
                        if key_event
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        return KeyInput::Cancel
                    }
                    _ => {}
                }
            }
        }
    }
    KeyInput::None
}

/// Format duration as human-readable string
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        format!("{}m {:02}s", mins, remaining_secs)
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {:02}m", hours, mins)
    }
}

/// Wait for user to press Enter
fn wait_for_enter() -> Result<()> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(())
}

/// Plot a drawing with progress bar
fn cmd_plot(
    port: Option<&str>,
    file: &PathBuf,
    config: PlotConfig,
    setup: PlotterSetup,
    skip_confirmation: bool,
) -> Result<()> {
    let drawing = load_drawing_with_validation(file)?;
    let ctx = create_render_context()?;

    // Prepare the drawing once - flattens, optimizes, and calculates stats
    log::info!("Preparing drawing...");
    let prepare_start = Instant::now();
    let prepared = PreparedDrawing::new(&drawing, &config, &ctx);
    log::info!(
        "Drawing prepared in {:.2}s",
        prepare_start.elapsed().as_secs_f64()
    );

    validate_stroke_count(prepared.stroke_count())?;

    // Clone stats before moving prepared to the background thread
    let stats = prepared.stats.clone();

    // Show setup diagram and confirmation unless --yes was passed
    if !skip_confirmation {
        // Render the setup diagram
        println!("Plotter Setup: {:?} ({})", setup, setup.description());
        println!();

        let diagram = SetupDiagram::new(setup, drawing.width, drawing.height);
        diagram.render_to_terminal();
        println!();

        // Print legend
        diagram.print_legend_for_setup();
        println!();
    }

    // Print stats (always shown)
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
    println!(
        "  Motion planning: {}",
        if config.motion_planning_enabled {
            "enabled (smooth acceleration)"
        } else {
            "disabled (constant velocity)"
        }
    );
    if config.verify_position {
        println!("  Position verification: ENABLED (diagnostic mode)");
    }
    println!("  Estimated time: {}", stats.format_time());
    println!();

    // Wait for confirmation unless --yes was passed
    if !skip_confirmation {
        println!("Press Enter to start plotting, Ctrl+C to cancel");
        wait_for_enter()?;
        println!();
    }

    println!("Press SPACE to pause/resume, Q to cancel");
    println!();

    // Use plot_prepared_in_background - no re-flatten or re-optimize needed!
    let handle = plot_prepared_in_background(prepared, config, port.map(String::from))?;

    let progress = ProgressBar::new(stats.stroke_count as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40}] {pos}/{len} strokes | {prefix}")
            .unwrap()
            .progress_chars("##-"),
    );
    progress.set_message("Plotting");
    progress.set_prefix(format!("~{} remaining", stats.format_time()));

    // Enable raw mode for keyboard input
    terminal::enable_raw_mode()?;

    let start_time = Instant::now();
    let estimated_total = stats.estimated_time;
    let total_strokes = stats.stroke_count;

    let mut was_cancelled = false;
    let result = (|| -> Result<()> {
        while handle.is_running() {
            // Check for key input
            match check_for_key() {
                KeyInput::Space => {
                    let is_paused = handle.toggle_pause();
                    if is_paused {
                        progress.set_message("PAUSED");
                    } else {
                        progress.set_message("Plotting");
                    }
                }
                KeyInput::Cancel => {
                    progress.set_message("Cancelling...");
                    handle.cancel();
                }
                KeyInput::None => {}
            }

            // Process plot events
            for event in handle.drain_events() {
                match event {
                    PlotEvent::Started { total_strokes: n } => {
                        progress.set_length(n as u64);
                    }
                    PlotEvent::StrokeComplete { index, .. } => {
                        let completed = index + 1;
                        progress.set_position(completed as u64);

                        // Calculate remaining time based on progress
                        if completed > 0 && total_strokes > 0 {
                            let elapsed = start_time.elapsed();
                            let progress_ratio = completed as f64 / total_strokes as f64;

                            // Use weighted average of elapsed-based and estimate-based remaining time
                            let remaining = if progress_ratio > 0.05 {
                                // After 5% progress, use actual elapsed time to estimate remaining
                                let estimated_total_from_elapsed =
                                    Duration::from_secs_f64(elapsed.as_secs_f64() / progress_ratio);
                                let remaining_from_elapsed =
                                    estimated_total_from_elapsed.saturating_sub(elapsed);

                                // Blend with original estimate (more weight to elapsed as we progress)
                                let weight = progress_ratio.min(0.8);
                                let remaining_from_estimate =
                                    estimated_total.saturating_sub(Duration::from_secs_f64(
                                        estimated_total.as_secs_f64() * progress_ratio,
                                    ));

                                Duration::from_secs_f64(
                                    remaining_from_elapsed.as_secs_f64() * weight
                                        + remaining_from_estimate.as_secs_f64() * (1.0 - weight),
                                )
                            } else {
                                // Early on, use original estimate
                                estimated_total.saturating_sub(Duration::from_secs_f64(
                                    estimated_total.as_secs_f64() * progress_ratio,
                                ))
                            };

                            progress
                                .set_prefix(format!("~{} remaining", format_duration(remaining)));
                        }
                    }
                    PlotEvent::Paused => {
                        progress.set_message("PAUSED");
                    }
                    PlotEvent::Resumed => {
                        progress.set_message("Plotting");
                    }
                    PlotEvent::Completed => {
                        let elapsed = start_time.elapsed();
                        progress.set_prefix(format!("completed in {}", format_duration(elapsed)));
                        progress.finish_with_message("Done");
                    }
                    PlotEvent::Cancelled => {
                        was_cancelled = true;
                        progress.finish_with_message("Cancelled");
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

    let total_elapsed = start_time.elapsed();
    if was_cancelled {
        println!(
            "\nPlotting cancelled after {}. Pen raised and returned home.",
            format_duration(total_elapsed)
        );
    } else {
        println!(
            "\nPlotting complete! Total time: {}",
            format_duration(total_elapsed)
        );
    }

    Ok(())
}
