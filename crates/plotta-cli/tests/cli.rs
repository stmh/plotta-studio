//! Integration tests that run the built `plotta` binary.
//!
//! No plotter is attached when these run, so they only cover paths that must
//! fail before any connection attempt.

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use drawing_core::{Drawing, Element, Point};

/// Write a small valid drawing to a unique temp file.
fn sample_drawing(name: &str) -> PathBuf {
    let mut drawing = Drawing::new(100.0, 100.0);
    drawing.add(Element::line(
        Point::new(10.0, 10.0),
        Point::new(90.0, 90.0),
    ));
    let path = std::env::temp_dir().join(format!(
        "plotta-cli-test-{}-{}.json",
        name,
        std::process::id()
    ));
    drawing.save(&path).expect("save drawing");
    path
}

/// Run `plotta` with stdin not connected to a terminal.
fn run_plotta(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_plotta"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("run plotta")
}

#[test]
fn plot_rejects_invalid_port_path_before_connecting() {
    let file = sample_drawing("invalid-port");
    let out = run_plotta(&["plot", file.to_str().unwrap(), "--yes", "--port", "ttyUSB0"]);
    let _ = std::fs::remove_file(&file);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(
        stderr.contains("Invalid port path: 'ttyUSB0'"),
        "stderr: {stderr}"
    );
}
