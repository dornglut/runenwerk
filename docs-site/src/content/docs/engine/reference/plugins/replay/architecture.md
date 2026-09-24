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
- Does not own: Scene simulation execution itself.

## Module Layout

- Primary module: engine/src/plugins/replay.rs
- Entry surface: ReplayPlugin
- Runtime schedule touchpoints: PreUpdate, FixedUpdate, FrameEnd

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
