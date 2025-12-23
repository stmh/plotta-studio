---
# plotta-studio-tpl1
title: Sketch templates with cargo-generate
status: todo
type: epic
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
---

Create cargo-generate templates for quickly scaffolding new sketches, reducing boilerplate and making it easy to start new projects.

## Investigation

### Current Workflow
- Copy `sketches/sketch-001` directory
- Rename in `Cargo.toml`
- Manual and error-prone

### cargo-generate Benefits
- Single command to create new sketch
- Placeholder substitution (name, author, etc.)
- Can include multiple templates (simple, animated, with-gui)
- Industry standard for Rust project scaffolding

### Template Location Options
1. **In-repo templates**: `templates/` directory
2. **Separate repo**: `plotta-studio-templates` repo
3. **Both**: Local for development, published for users

### Recommended: In-repo with remote support
```bash
# Local development
cargo generate --path ./templates/sketch

# For users (after publishing)
cargo generate plotta-studio/templates --name my-sketch
```

## Template Structure

```
templates/
├── sketch-basic/
│   ├── cargo-generate.toml
│   ├── Cargo.toml.liquid
│   └── src/
│       └── main.rs.liquid
├── sketch-animated/
│   ├── cargo-generate.toml
│   ├── Cargo.toml.liquid
│   └── src/
│       └── main.rs.liquid
└── sketch-gui/
    ├── cargo-generate.toml
    ├── Cargo.toml.liquid
    └── src/
        └── main.rs.liquid
```

## Template Variables

```toml
# cargo-generate.toml
[template]
name = "plotta-sketch"
description = "A new plotta-studio sketch"

[placeholders]
project-name = { prompt = "Project name?" }
author = { prompt = "Author?", default = "" }
paper-size = { prompt = "Paper size?", choices = ["a4-landscape", "a4-portrait", "a3-landscape", "a3-portrait", "custom"], default = "a4-landscape" }
```

## Usage

```bash
# Install cargo-generate
cargo install cargo-generate

# Create new sketch
cargo generate --path ./templates/sketch-basic --name my-new-sketch

# Or with all options
cargo generate --path ./templates/sketch-basic \
  --name my-new-sketch \
  --define author="Your Name" \
  --define paper-size="a3-landscape"
```
