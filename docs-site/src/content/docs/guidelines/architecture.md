---
title: Architecture
description: Architecture
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-12
---

# Architecture

This document defines the active crate/domain boundaries in `Runenwerk` and where new code belongs.

For the canonical Runenwerk-wide architecture spine, see
[`../architecture/runenwerk-platform-architecture.md`](../architecture/runenwerk-platform-architecture.md).

For the reusable boundary doctrine underneath that specialization, see
[`authority-centered-boundary-architecture.md`](authority-centered-boundary-architecture.md).

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

Repository-family extraction and peer-framework dependency direction are governed by
[Repository Family Architecture](../architecture/repository-family-architecture.md) and
[ADR 0014](../adr/accepted/0014-repository-family-extraction-boundaries.md), not by the
physical location of legacy code in this workspace.

## Ownership Boundaries

- `domain/*` owns engine-agnostic domain contracts, data structures, and execution primitives.
- `engine` owns runtime composition and plugin integration points.
- `net/*` owns protocol/session/transport contracts and replay storage/runtime integration.
- `apps/*` owns process wiring, config loading, and external system integration.
- `adapters/*` owns interop glue to external runtimes and host engines.

If logic must remain reusable across engine hosts, keep it in the owning reusable domain
or peer framework. If it is Runenwerk-specific runtime/product glue, keep it in
Runenwerk's engine/app/integration surfaces according to the current owner architecture.

## Placement Rules

When adding code:

1. Choose the semantic owner first; current code location is evidence, not permanent ownership.
2. Reuse local helpers in that owner before adding new abstractions.
3. Expose narrow public interfaces instead of reaching into internals across crates.
4. Keep peer frameworks independent of Runenwerk unless a separate ADR accepts a direct dependency.
5. Add or update local docs when behavior or scope changes.

## Architecture Guardrails

- Prefer explicit types, deterministic control flow, and clear ownership.
- One semantic invariant set has one authority.
- Do not add silent failure paths or broad catch-all error handling.
- Do not move code across domains unless the ownership boundary itself is changing.
- Do not infer one universal ID, graph, registry, database, transaction, or runtime merely from repeated vocabulary.
- Keep docs and crate boundaries aligned with current accepted ownership and `Cargo.toml` workspace evidence.

See also:

- `AGENTS.md` for agent behavior rules.
- `domain-map.md` for crate-level ownership and dependency summary.
- `code-patterns.md` for implementation patterns used across domains.
- [`domain-program-architecture-pattern.md`](domain-program-architecture-pattern.md)
  for the optional durable domain-program/compiler/evaluator pattern when a domain actually needs it.
