---
title: Phase 10 Local Validation
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-26
---

# Phase 10 Local Validation

Run this gate before merging the full Phase 10 completion branch.

```text
cargo fmt --all --check
cargo check -p ui_render_data
cargo check -p ui_controls
cargo check -p ui_runtime
cargo check -p engine
cargo test -p ui_render_data render_output
cargo test -p ui_controls control_render
cargo test -p ui_runtime runtime_render_output
cargo test -p engine render_output_proof
git diff --check
```
