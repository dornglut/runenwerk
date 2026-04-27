---
title: "Engine Plugins"
description: "Documentation for Engine Plugins."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Engine Plugins

This directory is the feature composition layer for `engine`.

## Contract

- Plugin modules are either:
  - folder modules (`<plugin>/mod.rs` with `<plugin>/README.md`)
  - file modules (`<plugin>.rs`) when scope is intentionally small
- Plugin `build` methods should only compose app/runtime state:
  - initialize resources
  - register systems
  - configure schedule ordering
- Cross-plugin helpers belong in `shared/`.
- Prefer typed schedules/sets from `engine::runtime`.

## Guides

- Canonical plugin guide index:
  - [`../../docs/reference/plugins/index.md`](../reference/plugins/index.md)

## Plugin Index

- `time/`
  - README: [`time/README.md`](time/README.md)
  - Guides: [`../../docs/reference/plugins/time/usage-guide.md`](../reference/plugins/time/usage-guide.md)
  - Purpose: frame time progression.
- `fixed_step.rs`
  - Guides: [`../../docs/reference/plugins/fixed-step/usage-guide.md`](../reference/plugins/fixed-step/usage-guide.md)
  - Purpose: fixed-step resource installation (`FixedTimeConfig`, `CatchupBudget`, `FixedTimeState`, `SimulationTick`).
- `replay.rs`
  - Guides: [`../../docs/reference/plugins/replay/usage-guide.md`](../reference/plugins/replay/usage-guide.md)
  - Purpose: replay recording/playback resources and fixed-step replay hooks.
- `input/`
  - README: [`input/README.md`](input/README.md)
  - Guides: [`../../docs/reference/plugins/input/usage-guide.md`](../reference/plugins/input/usage-guide.md)
  - Purpose: action mapping and frame input pulses.
- `scene/`
  - README: [`scene/README.md`](scene/README.md)
  - Guides: [`../../docs/reference/plugins/scene/usage-guide.md`](../reference/plugins/scene/usage-guide.md)
  - Purpose: scene lifecycle and runtime state publication.
- `render/`
  - README: [`render/README.md`](render/README.md)
  - Guides: [`../../docs/reference/plugins/render/usage-guide.md`](../reference/plugins/render/usage-guide.md)
  - Purpose: render graph/executor/shader orchestration.
- `net/`
  - README: [`net/README.md`](net/README.md)
  - Guides: [`../../docs/reference/plugins/net/usage-guide.md`](../reference/plugins/net/usage-guide.md)
  - Purpose: network runtime and replication bridge.
- `ui/`
  - README: [`ui/README.md`](ui/README.md)
  - Guides: [`../../docs/reference/plugins/ui/usage-guide.md`](../reference/plugins/ui/usage-guide.md)
  - Purpose: UI domain data and template/text flows.
- `grid/`
  - README: [`grid/README.md`](grid/README.md)
  - Guides: [`../../docs/reference/plugins/grid/usage-guide.md`](../reference/plugins/grid/usage-guide.md)
  - Purpose: gameplay-to-grid runtime config publication.
- `debug_metrics/`
  - README: [`debug_metrics/README.md`](debug-metrics/README.md)
  - Guides: [`../../docs/reference/plugins/debug-metrics/usage-guide.md`](../reference/plugins/debug-metrics/usage-guide.md)
  - Purpose: diagnostics overlay state and draw commands.
- `scheduler_diagnostics/`
  - README: [`scheduler_diagnostics/README.md`](scheduler-diagnostics/README.md)
  - Guides: [`../../docs/reference/plugins/scheduler-diagnostics/usage-guide.md`](../reference/plugins/scheduler-diagnostics/usage-guide.md)
  - Purpose: periodic runtime diagnostics logging.
- `shared/`
  - README: [`shared/README.md`](shared/README.md)
  - Guides: [`../../docs/reference/plugins/shared/usage-guide.md`](../reference/plugins/shared/usage-guide.md)
  - Purpose: shared helper utilities.
- `world/`
  - README: [`world/README.md`](world/README.md)
  - Guides: [`../../docs/reference/plugins/world/index.md`](../reference/plugins/world/index.md)
  - Purpose: authoritative chunked SDF world runtime, op-log edits, rebuild graph, streaming contracts, and render-prepare world feature payloads.

## Entry Points

- Plugin trait: `engine/src/plugin.rs`
- Plugin registry module: `engine/src/plugins/mod.rs`
- Default plugin stack helper: `engine::plugins::default_plugins()`
- Crate advanced docs: `engine/docs/reference/advanced-guide.md`
- Crate architecture docs: `engine/docs/reference/architecture.md`
