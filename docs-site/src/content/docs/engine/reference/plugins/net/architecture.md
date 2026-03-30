---
title: "Net Plugin Architecture"
description: "Documentation for Net Plugin Architecture."
---

# Net Plugin Architecture

## Ownership Boundary

- Owns: Role-based networking runtime composition.
- Does not own: Transport implementation internals in engine_net.

## Module Layout

- Primary module: engine/src/plugins/net/plugin.rs
- Entry surface: NetPlugin<TDriver>
- Runtime schedule touchpoints: PreUpdate, FixedUpdate, FrameEnd

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
