---
title: "Scheduler Diagnostics Plugin Architecture"
description: "Documentation for Scheduler Diagnostics Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Scheduler Diagnostics Plugin Architecture

## Ownership Boundary

- Owns: Diagnostics cadence and structured log emission.
- Does not own: Scheduler execution internals.

## Module Layout

- Primary module: engine/src/plugins/scheduler_diagnostics/mod.rs
- Entry surface: SchedulerDiagnosticsPlugin
- Runtime schedule touchpoints: RenderSubmit

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
