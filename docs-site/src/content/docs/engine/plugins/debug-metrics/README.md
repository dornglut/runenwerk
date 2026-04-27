---
title: "Debug Metrics Plugin"
description: "Documentation for Debug Metrics Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Debug Metrics Plugin

## Purpose

Provides an on-screen diagnostics overlay for frame timing and runtime state.

## Usage

- Plugin: `DebugMetricsPlugin`
- Toggle overlay action: `debug.metrics.toggle`
- Default key: `F10`

The plugin appends UI draw commands to the overlay UI draw list each frame when enabled.

## Ownership Boundaries

- Owns debug metrics visibility toggle and overlay rendering content.
- Consumes runtime timing/state (`DebugMetricsState`, startup state, scene labels).
- Does not own frame submission or UI extraction orchestration.

## Extension Points

- Add additional diagnostic lines/sections in `debug_metrics_overlay_system`.
- Rebind the toggle action through input binding changes.

## Guides

- Usage: [../../../docs/reference/plugins/debug-metrics/usage-guide.md](../../reference/plugins/debug-metrics/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/debug-metrics/advanced-guide.md](../../reference/plugins/debug-metrics/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/debug-metrics/architecture.md](../../reference/plugins/debug-metrics/architecture.md)


