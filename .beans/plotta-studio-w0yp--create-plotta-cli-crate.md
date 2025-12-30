---
# plotta-studio-w0yp
title: Create plotta-cli crate
status: completed
type: task
priority: normal
created_at: 2025-12-28T15:31:47Z
updated_at: 2025-12-28T20:07:23Z
parent: plotta-studio-7wzz
blocking:
    - plotta-studio-vd8d
---

Create the new CLI crate at `crates/plotta-cli/`.

## Crate Setup

Create `crates/plotta-cli/Cargo.toml`:
```toml
[package]
name = "plotta-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "plotta"
path = "src/main.rs"

[dependencies]
drawing-core = { path = "../drawing-core" }
drawing-plotter = { path = "../drawing-plotter" }
clap = { workspace = true, features = ["derive"] }
indicatif = "0.17"
anyhow = { workspace = true }
serde_json = { workspace = true }
```

## CLI Structure

Create `crates/plotta-cli/src/main.rs` with clap setup:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "plotta")]
#[command(about = "CLI for plotter control")]
struct Cli {
    /// Serial port path (auto-detects if not provided)
    #[arg(long, global = true)]
    port: Option<String>,

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
    /// Plot a drawing from JSON file
    Plot {
        /// Path to JSON drawing file
        file: PathBuf,
    },
    /// Show drawing stats without plotting
    Preview {
        /// Path to JSON drawing file
        file: PathBuf,
    },
}
```

## Add to Workspace

Add `"crates/plotta-cli"` to workspace members in root `Cargo.toml`