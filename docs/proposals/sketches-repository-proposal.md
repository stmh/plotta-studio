# Proposal: Moving Sketches to a Dedicated Repository

## Executive Summary

This proposal outlines options for extracting the 12 sketches from `plotta-studio` into a dedicated repository, enabling independent development, better separation of concerns, and easier sharing of sketch examples.

## Current State

### Project Structure
```
plotta-studio/
├── Cargo.toml           # Workspace root
├── crates/              # 8 library/CLI crates
│   ├── drawing-core/    # Fundamental types (Drawing, Element, Shape, etc.)
│   ├── sketch-runner/   # Framework for running sketches (Sketch trait)
│   ├── drawing-text/    # Single-line font support
│   ├── drawing-utils/   # Utilities (hatching, frames, signatures)
│   ├── drawing-svg/     # SVG import/export
│   ├── drawing-plotter/ # AxiDraw control and path optimization
│   ├── plotta-cli/      # CLI interface
│   └── vsf-convert/     # Font conversion tool
├── sketches/            # 12 example sketches (to be extracted)
│   ├── sketch-001-radial/
│   ├── sketch-002-dvd-screensaver/
│   ├── sketch-003-text/
│   ├── sketch-004-hatched-circles/
│   ├── sketch-005-svg-viewer/
│   ├── sketch-006-hamburg/
│   ├── sketch-007-1000-lines/
│   ├── sketch-008-wool-ball/
│   ├── sketch-009-altoetting/
│   ├── sketch-010-schnellstrasse/
│   ├── sketch-011-clip-demo/
│   └── sketch-012-rotated-squares/
└── fonts/               # Font assets (Hershey, VSF, SVG)
```

### Sketch Dependencies

| Sketch | Dependencies |
|--------|-------------|
| sketch-001-radial | sketch-runner |
| sketch-002-dvd-screensaver | sketch-runner |
| sketch-003-text | sketch-runner, drawing-text |
| sketch-004-hatched-circles | sketch-runner, drawing-text, drawing-utils |
| sketch-005-svg-viewer | sketch-runner, drawing-svg, drawing-text |
| sketch-006-hamburg | sketch-runner, drawing-svg, drawing-utils |
| sketch-007-1000-lines | sketch-runner, drawing-utils |
| sketch-008-wool-ball | sketch-runner, drawing-utils |
| sketch-009-altoetting | sketch-runner, drawing-svg, drawing-utils |
| sketch-010-schnellstrasse | sketch-runner, drawing-svg, drawing-utils |
| sketch-011-clip-demo | sketch-runner, drawing-utils |
| sketch-012-rotated-squares | sketch-runner, drawing-utils, rand |

### Key Observations

1. **All sketches depend on `sketch-runner`** (which provides the `Sketch` trait and windowing)
2. **`sketch-runner` depends on `drawing-core` and `drawing-text`** (always)
3. **Some sketches embed SVG assets** via `include_str!()`
4. **Fonts are embedded** at compile time from the `/fonts/` directory
5. **Sketches are standalone binaries** - each has its own `Cargo.toml`

---

## Proposed Options

### Option A: Git Dependencies (Recommended)

**Description**: Keep library crates in `plotta-studio`, reference them via git in the new sketches repository.

**New Repository Structure**:
```
plotta-sketches/
├── Cargo.toml           # Workspace root with git dependencies
├── sketches/
│   ├── sketch-001-radial/
│   ├── sketch-002-dvd-screensaver/
│   └── ... (all 12 sketches)
└── assets/              # Shared assets (optional)
```

**Workspace Cargo.toml**:
```toml
[workspace]
resolver = "2"
members = ["sketches/*"]

[workspace.dependencies]
sketch-runner = { git = "https://github.com/stmh/plotta-studio", features = ["svg"] }
drawing-core = { git = "https://github.com/stmh/plotta-studio" }
drawing-text = { git = "https://github.com/stmh/plotta-studio" }
drawing-utils = { git = "https://github.com/stmh/plotta-studio" }
drawing-svg = { git = "https://github.com/stmh/plotta-studio" }
drawing-plotter = { git = "https://github.com/stmh/plotta-studio" }

# External dependencies
rand = "0.8"
```

**Individual Sketch Cargo.toml** (example):
```toml
[package]
name = "sketch-001-radial"
version = "0.1.0"
edition = "2021"

[dependencies]
sketch-runner = { workspace = true }

[features]
default = []
hardware = ["sketch-runner/hardware"]
```

**Pros**:
- Minimal changes to existing code
- Easy to keep sketches in sync with library updates
- No need to publish crates to crates.io
- Clear separation: core library vs. examples

**Cons**:
- Requires internet access to build (git fetch)
- Version pinning requires specifying `rev` or `tag`
- Build times may be longer (clones repo each time)

---

### Option B: Publish Crates to crates.io

**Description**: Publish library crates to crates.io, use standard versioned dependencies.

**Workspace Cargo.toml**:
```toml
[workspace.dependencies]
sketch-runner = "0.1"
drawing-core = "0.1"
drawing-text = "0.1"
drawing-utils = "0.1"
drawing-svg = "0.1"
drawing-plotter = "0.1"
```

**Pros**:
- Standard Rust ecosystem approach
- Easy for others to use in their own projects
- Reliable versioning and compatibility
- Faster builds (uses crates.io cache)

**Cons**:
- Requires publishing and maintaining crate versions
- Must follow semver strictly
- Public API stability expectations
- More overhead for rapid iteration

---

### Option C: Git Submodules

**Description**: Include `plotta-studio` as a git submodule in the sketches repository.

**New Repository Structure**:
```
plotta-sketches/
├── Cargo.toml
├── plotta-studio/       # Git submodule
├── sketches/
│   └── ...
└── .gitmodules
```

**Workspace Cargo.toml**:
```toml
[workspace.dependencies]
sketch-runner = { path = "plotta-studio/crates/sketch-runner" }
drawing-core = { path = "plotta-studio/crates/drawing-core" }
# ... etc
```

**Pros**:
- Exact version control via submodule commit
- Works offline once cloned
- Easy to test local changes to both repos

**Cons**:
- Submodules add complexity for contributors
- Easy to forget to update submodule
- Two-repo checkout required

---

### Option D: Monorepo with Separate Workspaces

**Description**: Keep everything in one repo but use separate Cargo workspaces with feature flags.

**Structure**:
```
plotta-studio/
├── Cargo.toml           # Library workspace
├── crates/
├── fonts/
└── examples/            # Rename from sketches, not in main workspace

examples/
├── Cargo.toml           # Separate workspace
└── sketches/
```

**Pros**:
- Everything in one place
- Easy to develop both simultaneously
- No external dependencies

**Cons**:
- Doesn't actually separate repositories (not the goal)
- Workspace management complexity

---

## Recommended Approach: Option A (Git Dependencies)

### Implementation Plan

#### Phase 1: Prepare plotta-studio

1. **Tag a stable release** of plotta-studio (e.g., `v0.1.0`)
2. **Ensure public API stability** for crates used by sketches
3. **Document the Sketch trait** and public interfaces

#### Phase 2: Create New Repository

1. **Create `plotta-sketches` repository**
2. **Set up workspace Cargo.toml** with git dependencies
3. **Copy sketches** from `plotta-studio/sketches/`
4. **Copy necessary assets** (embedded SVGs stay with sketches)
5. **Update sketch Cargo.toml files** to use workspace dependencies
6. **Add README** with build instructions

#### Phase 3: Clean Up plotta-studio

1. **Remove `sketches/` directory** from plotta-studio
2. **Update workspace members** in root Cargo.toml
3. **Add reference** to plotta-sketches in documentation

#### Phase 4: CI/CD Setup

1. **Set up GitHub Actions** for plotta-sketches
2. **Configure build matrix** (with/without hardware feature)
3. **Add dependabot** for git dependency updates (optional)

### Migration Script

```bash
#!/bin/bash
# migrate-sketches.sh

# Create new repository structure
mkdir -p plotta-sketches/sketches

# Copy all sketches
cp -r plotta-studio/sketches/* plotta-sketches/sketches/

# Create workspace Cargo.toml
cat > plotta-sketches/Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = ["sketches/*"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/stmh/plotta-sketches"

[workspace.dependencies]
# Core plotta-studio dependencies
sketch-runner = { git = "https://github.com/stmh/plotta-studio", tag = "v0.1.0", features = ["svg"] }
drawing-core = { git = "https://github.com/stmh/plotta-studio", tag = "v0.1.0" }
drawing-text = { git = "https://github.com/stmh/plotta-studio", tag = "v0.1.0" }
drawing-utils = { git = "https://github.com/stmh/plotta-studio", tag = "v0.1.0" }
drawing-svg = { git = "https://github.com/stmh/plotta-studio", tag = "v0.1.0" }
drawing-plotter = { git = "https://github.com/stmh/plotta-studio", tag = "v0.1.0" }

# External dependencies
rand = "0.8"
EOF

# Create README
cat > plotta-sketches/README.md << 'EOF'
# Plotta Sketches

A collection of example sketches for the [plotta-studio](https://github.com/stmh/plotta-studio) pen plotter framework.

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- For hardware plotting: AxiDraw connected via USB

## Building

```bash
# Build all sketches
cargo build --release

# Build with hardware (AxiDraw) support
cargo build --release --features hardware
```

## Running a Sketch

```bash
# Run a specific sketch
cargo run -p sketch-001-radial

# Run with hardware support
cargo run -p sketch-001-radial --features hardware
```

## Controls

- **Space**: Fit drawing to window
- **R**: Reset view
- **S**: Save to drawing.json
- **E**: Export SVG
- **P**: Plot to AxiDraw (requires hardware feature)
- **Mouse**: Pan and zoom

## Available Sketches

| Sketch | Description |
|--------|-------------|
| sketch-001-radial | Concentric circles with radial rays |
| sketch-002-dvd-screensaver | Animated bouncing DVD logo |
| sketch-003-text | Font rendering demo |
| sketch-004-hatched-circles | Hatching pattern demo |
| sketch-005-svg-viewer | SVG import showcase |
| sketch-006-hamburg | Hamburg city map |
| sketch-007-1000-lines | Line clipping demo |
| sketch-008-wool-ball | Bezier curve patterns |
| sketch-009-altoetting | Altoetting city map |
| sketch-010-schnellstrasse | Road map SVG |
| sketch-011-clip-demo | Inverted clipping demo |
| sketch-012-rotated-squares | Hidden line removal demo |

## License

MIT
EOF

echo "Migration complete. Review and update individual sketch Cargo.toml files."
```

### Updating Individual Sketches

Each sketch's `Cargo.toml` needs to be updated to use workspace dependencies:

**Before** (in plotta-studio):
```toml
[dependencies]
sketch-runner = { path = "../../crates/sketch-runner", features = ["svg"] }
drawing-utils = { path = "../../crates/drawing-utils" }
```

**After** (in plotta-sketches):
```toml
[dependencies]
sketch-runner = { workspace = true }
drawing-utils = { workspace = true }
```

---

## Version Pinning Strategy

For stability, use git tags:

```toml
sketch-runner = { git = "https://github.com/stmh/plotta-studio", tag = "v0.1.0" }
```

For development/latest:

```toml
sketch-runner = { git = "https://github.com/stmh/plotta-studio", branch = "main" }
```

For specific commit:

```toml
sketch-runner = { git = "https://github.com/stmh/plotta-studio", rev = "abc123" }
```

---

## Future Considerations

1. **Crates.io Publishing**: Once the API stabilizes, consider publishing crates for easier consumption
2. **Template Repository**: Create a sketch template for easy bootstrapping of new sketches
3. **Sketch Gallery**: Build a website showcasing sketch outputs
4. **Community Contributions**: Allow others to submit sketches via PRs

---

## Conclusion

**Recommendation**: Proceed with **Option A (Git Dependencies)** as it provides the best balance of:
- Separation of concerns (library vs. examples)
- Ease of implementation
- Flexibility for future changes
- No need to manage crate publishing

The migration can be completed in a few hours, with minimal disruption to existing workflows.
