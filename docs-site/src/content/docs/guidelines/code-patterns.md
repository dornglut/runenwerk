---
title: Code Patterns
description: Code Patterns
---

# Code Patterns

This file captures implementation patterns already used in this repository. Reuse these before introducing new abstractions.

## Runtime Composition Pattern

Use plugin-based composition through `engine::App` and crate-local plugins:

- Prefer new feature integration as a plugin module under `engine/src/plugins/<feature>/`.
- Register systems into existing runtime stages (`Startup`, `PreUpdate`, `FixedUpdate`, `Update`, `RenderPrepare`, `RenderSubmit`, `FrameEnd`) instead of creating ad-hoc loops.
- Keep plugin resources/events/components scoped to the owning plugin module when possible.

## Domain-First Placement Pattern

Place code in the owning domain first:

- `domain/*` for reusable engine-agnostic runtime building blocks.
- `engine/*` for engine-generic runtime features.
- `net/*` for protocol/session/transport/replay contracts.
- `apps/*` for binary wiring and external integrations.
- `adapters/*` for external runtime/engine integration glue.

If the code must stay reusable across different host engines (custom engine, Bevy, Godot), keep it in `domain/*`.

## Contract-Over-Concrete Pattern

Networking code should depend on contracts, not game internals:

- Use `engine_net` protocol/session/replication traits as boundaries.
- Put transport-specific runtime behavior in `engine_net_quic` rather than in game or engine code.
- Keep payload interpretation in the owning gameplay domain/app module (not in transport or engine-generic crates).

## Explicit State and Scheduling Pattern

- Prefer explicit resources/components and deterministic fixed-step systems.
- Use typed ECS access (`Res`, `ResMut`, `Query`, `Commands`) over hidden global mutable state.
- Apply deferred ECS commands explicitly at stage boundaries where required.

## Error and Result Pattern

- Return explicit `Result` types from boundary APIs.
- Surface errors to callers with context.
- Avoid broad "best effort" swallowing in runtime, network, or replay paths.

## Documentation Pattern

When behavior changes in a crate/domain:

1. Update that area's `README.md`.
2. Update usage/architecture docs for that area when behavior contracts changed.
3. Update root docs (`architecture.md`, `domain-map.md`) if boundaries changed.

Keep docs aligned with actual workspace members in root `Cargo.toml`.
