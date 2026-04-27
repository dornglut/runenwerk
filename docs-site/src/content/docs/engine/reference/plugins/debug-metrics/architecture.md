---
title: "Debug Metrics Plugin Architecture"
description: "Documentation for Debug Metrics Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Debug Metrics Plugin Architecture

## Ownership Boundary

- Owns: Debug overlay state and draw-list publication.
- Does not own: Render submission execution and input transport.

## Module Layout

- Primary module: engine/src/plugins/debug_metrics/mod.rs
- Entry surface: DebugMetricsPlugin
- Runtime schedule touchpoints: Startup, RenderPrepare

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
