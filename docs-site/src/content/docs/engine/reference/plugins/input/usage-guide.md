---
title: "Input Plugin Usage Guide"
description: "Documentation for Input Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Input Plugin Usage Guide

## Purpose

Maintains action-mapped input state and frame pulse clearing.

## Entry Points

- Module: engine/src/plugins/input/mod.rs
- Entry: InputFinalizePlugin
- Local README: engine/src/plugins/input/README.md

## Minimal Setup

```rust
use engine::plugins::input::InputFinalizePlugin;

app.add_plugin(InputFinalizePlugin);
```

## Runtime Contract

- Schedule placement: FrameEnd (CoreSet::FrameEnd)
- Ownership: Input action and pulse lifecycle.
- Non-ownership: Gameplay systems consuming input.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
