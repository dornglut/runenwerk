---
title: "Replay Plugin Architecture"
description: "Documentation for Replay Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Replay Plugin Architecture

## Ownership Boundary

- Owns: Replay recording and playback lifecycle resources.
- Owns: Replay runtime-control ergonomics through `AppReplayExt`.
- Owns: the private integration-activation fact used to admit Replay runtime controls.
- Does not own: Scene simulation execution itself.
- Does not own: `SimulationTick` or `App::current_tick`; simulation identity remains separate.

## Module Layout

- Primary module: engine/src/plugins/replay.rs
- Entry surface: ReplayPlugin and AppReplayExt
- Runtime schedule touchpoints: PreUpdate, FixedUpdate, FrameEnd

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Public Replay resources are not capability-selection authority; only explicit `ReplayPlugin`
  selection activates Replay runtime controls.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
