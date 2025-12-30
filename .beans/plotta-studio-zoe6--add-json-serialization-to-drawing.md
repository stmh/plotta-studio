---
# plotta-studio-zoe6
title: Add JSON serialization to Drawing
status: todo
type: task
priority: normal
created_at: 2025-12-28T15:31:35Z
updated_at: 2025-12-28T15:32:10Z
parent: plotta-studio-7wzz
blocking:
    - plotta-studio-w0yp
---

Add `Serialize` and `Deserialize` derives to the `Drawing` struct and all nested types in `drawing-core`.

## Changes Required

In `crates/drawing-core/`:
1. Add `serde` dependency with `derive` feature to `Cargo.toml`
2. Add `#[derive(Serialize, Deserialize)]` to:
   - `Drawing`
   - `Element` (and variants)
   - `Group`
   - `Stroke`
   - `Point` (if not already)
   - `Color` (if not already)
   - Any other types that `Drawing` contains

## Verification

- Create a simple test that serializes a `Drawing` to JSON and deserializes it back
- Ensure round-trip produces equivalent drawing