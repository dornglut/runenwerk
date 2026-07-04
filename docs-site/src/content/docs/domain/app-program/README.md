---
title: App Program
description: Current crate boundary for typed app-program contracts.
status: active
owner: app-program
layer: domain
canonical: true
last_reviewed: 2026-07-04
related_docs:
  - ../../workspace/planning/typed-app-program-ui-proof-001-planning.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
  - ../../reports/investigations/typed-app-program-current-state-investigation.md
---

# App Program

`domain/app_program` owns the first Typed App Program proof crate.

The production crate owns UI-independent app-program structure:

```text
IDs
model snapshots
actions and payload summaries
route-action mapping
pure reducer inputs and outcomes
inert effect plans
view projections
deterministic replay traces
reports and diagnostics
```

The first proof is the headless counter app. Counter semantics are demo-owned fixture semantics, not platform meaning.

## Boundary

`app_program` production code must not depend on UI, editor, game, engine, net, material graph, procgen, renderer backend, or `foundation/meta` crates.

UI crates are proving consumers only through tests/examples/dev-dependencies. The current proving consumer is:

```text
domain/app_program/examples/headless_counter_ui.rs
domain/app_program/tests/headless_counter_replay.rs
```

## Non-Goals

This crate does not own editor integration, game runtime integration, engine subsystem execution, networking, multiplayer, asset IO, file IO, thread pools, async runtimes, hot reload, localization, telemetry, renderer resources, world mutation, `AppRecipe`, `PluginSuite`, or a shared plugin framework.
