---
title: Architecture
description: Architecture
---

# Architecture

This document defines the active crate/domain boundaries in `Runenwerk` and where new code belongs.

For the canonical governing architecture doctrine, see
[`runenwerk-architecture.md`](runenwerk-architecture.md).

This page remains the workspace boundary and placement guide.

## Top-Level Domains

- `foundation/`: low-level shared primitives reused across domains (for example typed ids)
- `domain/`: engine-agnostic reusable gameplay/runtime logic (`ecs`, `scheduler`, `scene`, editor domains)
- `engine/`: runtime loop, plugin system, rendering, input, scene, time integration
- `net/`: transport/session/replication infrastructure (`engine_net`, `engine_net_quic`)
- `apps/`: runnable applications and tooling (`runenwerk_editor`, other app binaries)
- `adapters/`: external engine/runtime bridges (for example Godot adapters)
- `assets/`: data assets consumed by engine/domain/apps/adapters
- `docs-site/`: documentation source tree

## Dependency Direction

Keep dependency flow unidirectional:

- `domain` -> no project-internal dependency on higher domains
- `domain` -> `foundation`
- `engine` -> `foundation` + `domain`
- `net` -> `foundation` + `domain` (and self-contained net crates)
- `apps` -> `foundation` + `domain` + `engine` + `net` contracts as needed
- `adapters` -> `foundation` + `domain` (+ targeted integration crates as needed)

Avoid sideways coupling between app crates via private internals.

## Ownership Boundaries

- `domain/*` owns engine-agnostic domain contracts, data structures, and execution primitives.
- `engine` owns runtime composition and plugin integration points.
- `net/*` owns protocol/session/transport contracts and replay storage/runtime integration.
- `apps/*` owns process wiring, config loading, and external system integration.
- `adapters/*` owns interop glue to external runtimes and host engines.

If logic must remain reusable across engine hosts, keep it in `domain/`. If it is engine-specific runtime glue, keep it in `engine/`.

## Placement Rules

When adding code:

1. Choose the owning domain first (`domain`, `engine`, `net`, `apps`, or `adapters`).
2. Reuse local helpers in that domain before adding new abstractions.
3. Expose narrow public interfaces instead of reaching into internals across crates.
4. Add or update local docs (`readme.md`, `usage-guide.md`, `architecture.md`) when behavior or scope changes.

## Architecture Guardrails

- Prefer explicit types, deterministic control flow, and clear ownership.
- Do not add silent failure paths or broad catch-all error handling.
- Do not move code across domains unless the ownership boundary itself is changing.
- Keep docs and crate boundaries aligned with `Cargo.toml` workspace members.

See also:

- `AGENTS.md` for agent behavior rules.
- `domain-map.md` for crate-level ownership and dependency summary.
- `code-patterns.md` for implementation patterns used across domains.
