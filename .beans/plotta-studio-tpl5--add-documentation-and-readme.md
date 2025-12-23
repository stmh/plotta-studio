---
# plotta-studio-tpl5
title: Add documentation and README
status: todo
type: task
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-23T19:00:00Z
parent: plotta-studio-tpl1
---

Document the template usage and update main README.

## Template README

Create `templates/README.md`:

```markdown
# Plotta Studio Templates

Templates for creating new sketches using cargo-generate.

## Prerequisites

Install cargo-generate:
\`\`\`bash
cargo install cargo-generate
\`\`\`

## Available Templates

### Basic Sketch
A minimal sketch with setup and key handling:
\`\`\`bash
cargo generate --path ./templates/sketch-basic --name my-sketch
\`\`\`

### Animated Sketch
A sketch that updates every frame:
\`\`\`bash
cargo generate --path ./templates/sketch-animated --name my-animation
\`\`\`

### GUI Sketch
A sketch with egui parameter controls:
\`\`\`bash
cargo generate --path ./templates/sketch-gui --name my-interactive
\`\`\`

## Options

All templates support these options:

| Option | Description | Default |
|--------|-------------|---------|
| `project-name` | Name for the new sketch | (required) |
| `description` | Project description | "A generative drawing" |
| `paper-size` | Paper size preset | "a4-landscape" |

## Example

\`\`\`bash
cargo generate --path ./templates/sketch-basic \\
  --name circular-pattern \\
  --define paper-size=a3-landscape
\`\`\`

This creates `sketches/circular-pattern/` with all boilerplate ready.

## Running Your Sketch

\`\`\`bash
cargo run -p my-sketch
\`\`\`
```

## Update Main README

Add section to main `README.md`:

```markdown
## Creating New Sketches

The easiest way to create a new sketch is with cargo-generate:

\`\`\`bash
# Install cargo-generate (once)
cargo install cargo-generate

# Create a new sketch
cargo generate --path ./templates/sketch-basic --name my-sketch

# Run it
cargo run -p my-sketch
\`\`\`

See [templates/README.md](templates/README.md) for all available templates.
```

## Files to Create/Modify
- `templates/README.md` (new)
- `README.md` (update)
