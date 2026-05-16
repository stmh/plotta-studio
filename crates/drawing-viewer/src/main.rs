//! drawing-viewer — open one or more saved Drawing JSON files in the
//! sketch-runner window and cycle through them with the arrow keys.
//!
//! Reuses sketch-runner's pan/zoom/fit/SVG-export keys (Space, R, E, S).
//!
//! Extra controls:
//! - Left / Right arrow: previous / next file
//! - Home / End: first / last file

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use sketch_runner::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "View saved Drawing JSON files")]
struct Cli {
    /// One or more JSON files to view. Use shell globs to pass multiple.
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

struct ViewerSketch {
    files: Vec<PathBuf>,
    index: usize,
}

impl ViewerSketch {
    fn new(files: Vec<PathBuf>) -> Self {
        Self { files, index: 0 }
    }

    fn current_path(&self) -> &PathBuf {
        &self.files[self.index]
    }

    fn load_current(&self) -> Drawing {
        let path = self.current_path();
        match Drawing::load(path) {
            Ok(d) => {
                log::info!(
                    "[{}/{}] {}",
                    self.index + 1,
                    self.files.len(),
                    path.display()
                );
                d
            }
            Err(e) => {
                log::error!("failed to load {}: {}", path.display(), e);
                // Show an empty A4 portrait so the window stays usable.
                Drawing::new(210.0, 297.0)
            }
        }
    }

    fn step(&mut self, delta: isize) {
        let n = self.files.len() as isize;
        if n <= 1 {
            return;
        }
        let next = ((self.index as isize + delta).rem_euclid(n)) as usize;
        self.index = next;
    }
}

impl Sketch for ViewerSketch {
    fn setup(&mut self, _ctx: &SketchContext) -> Drawing {
        self.load_current()
    }

    fn update(&mut self, _drawing: &mut Drawing, _ctx: &UpdateContext) -> bool {
        false
    }

    fn key_pressed(&mut self, key: &Key, drawing: &mut Drawing, _ctx: &SketchContext) -> bool {
        match key {
            Key::Named(NamedKey::ArrowRight) => {
                self.step(1);
                *drawing = self.load_current();
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.step(-1);
                *drawing = self.load_current();
                true
            }
            Key::Named(NamedKey::Home) => {
                self.index = 0;
                *drawing = self.load_current();
                true
            }
            Key::Named(NamedKey::End) => {
                self.index = self.files.len().saturating_sub(1);
                *drawing = self.load_current();
                true
            }
            _ => false,
        }
    }

    fn base_filename(&self) -> Option<String> {
        self.current_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.files.is_empty() {
        return Err(anyhow!("no files provided"));
    }
    for f in &cli.files {
        if !f.exists() {
            return Err(anyhow!("file not found: {}", f.display())).context("checking inputs");
        }
    }

    let sketch = ViewerSketch::new(cli.files);

    run_with_config(
        sketch,
        RunnerConfig::new("Drawing Viewer")
            .with_size(1100, 800)
            .with_animation(false),
    );

    Ok(())
}
