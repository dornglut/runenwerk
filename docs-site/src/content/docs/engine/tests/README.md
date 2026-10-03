---
title: "Engine Integration Tests"
description: "Documentation for Engine Integration Tests."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-10-03
publication: primary
---

# Engine Integration Tests

Integration coverage is organized by behavior area.

## Suites

- `runtime_core/`
  - One boundary-owned integration target for App lifecycle, runtime ownership, fixed-step/simulation, frame pacing, time, native-window hooks, and RunenECS consumer behavior.
- `render_contracts/`
  - One boundary-owned integration target for ordinary renderer/procedural contracts, planning, inspection, scale, SDF, temporal, and resource-model behavior. 
- `architecture_guards/`
  - One boundary-owned integration target for source/ownership guardrails covering GPU, input, render, runtime, and world architecture.
- `ui_plugins.rs`
  - Scene/UI runtime interaction and overlay state assertions.
- `replay_plugin.rs`
  - Replay recording, archive loading, and seek validation path.
- `network_plugins.rs`
  - Net plugin integration with role-specific runtime behavior.
- `network_plugins/`
  - Split modules for focused net behavior:
    - `basic_flow.rs`
    - `runtime_and_replication.rs`
    - `delta_and_reconnect.rs`

## Run

- Full integration tests:
  - `cargo test -p engine --tests`
- Architecture guardrails only:
  - `cargo test -p engine --test architecture_guards`

## Related

- Crate docs hub: [`../index.md`](../index.md)
- Usage guide: [`../reference/usage-guide.md`](../reference/usage-guide.md)
- Architecture guide: [`../reference/architecture.md`](../reference/architecture.md)
