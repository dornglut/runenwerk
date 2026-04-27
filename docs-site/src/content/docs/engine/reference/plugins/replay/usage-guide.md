---
title: "Replay Plugin Usage Guide"
description: "Documentation for Replay Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Replay Plugin Usage Guide

## Purpose

Captures simulation command frames/checkpoints and manages replay state resources.

## Entry Points

- Module: engine/src/plugins/replay.rs
- Entry: ReplayPlugin
- Local README: not present (file-based plugin module)

## Minimal Setup

```rust
use engine::plugins::replay::ReplayPlugin;

app.add_plugin(ReplayPlugin);
```

## Runtime Contract

- Schedule placement: PreUpdate, FixedUpdate, FrameEnd
- Ownership: Replay recording and playback lifecycle resources.
- Non-ownership: Scene simulation execution itself.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
