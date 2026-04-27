---
title: "Scheduler Diagnostics Plugin Usage Guide"
description: "Documentation for Scheduler Diagnostics Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Scheduler Diagnostics Plugin Usage Guide

## Purpose

Logs periodic scheduler/runtime diagnostics fields.

## Entry Points

- Module: engine/src/plugins/scheduler_diagnostics/mod.rs
- Entry: SchedulerDiagnosticsPlugin
- Local README: engine/src/plugins/scheduler_diagnostics/README.md

## Minimal Setup

```rust
use engine::plugins::scheduler_diagnostics::SchedulerDiagnosticsPlugin;

app.add_plugin(SchedulerDiagnosticsPlugin);
```

## Runtime Contract

- Schedule placement: RenderSubmit
- Ownership: Diagnostics cadence and structured log emission.
- Non-ownership: Scheduler execution internals.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
