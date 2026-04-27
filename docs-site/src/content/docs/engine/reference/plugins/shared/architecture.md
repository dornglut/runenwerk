---
title: "Shared Plugin Utilities Architecture"
description: "Documentation for Shared Plugin Utilities Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Shared Plugin Utilities Architecture

## Ownership Boundary

- Owns: Reusable plugin utility helpers.
- Does not own: Feature-specific domain logic.

## Module Layout

- Primary module: engine/src/plugins/shared/mod.rs
- Entry surface: module surface only; no standalone Plugin implementation
- Runtime schedule touchpoints: No direct schedule registration

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
