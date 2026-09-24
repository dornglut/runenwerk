---
title: Code Patterns
description: Implementation patterns used across Runenwerk.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ./programming-principles.md
  - ./architecture.md
---

# Code Patterns

This file captures implementation patterns already used in this repository. Reuse these before introducing new abstractions.

Use [Runenwerk Programming Principles](programming-principles.md) as the review lens for all patterns.

## Domain-first placement pattern

Place code in the owning domain first:

- `foundation/*` for reusable low-level primitives.
- `domain/*` for reusable engine-agnostic contracts and logic.
- `engine/*` for engine-generic runtime features.
- `net/*` for protocol/session/transport/replay contracts.
- `apps/*` for binary wiring and concrete application policy.
- `adapters/*` for external runtime and host integration glue.

This applies Separation of Concerns and Law of Demeter: callers should use the direct owner or contract instead of reaching through unrelated internals.

## Runtime composition pattern

Use plugin-based composition through `engine::App` and crate-local plugins:

- prefer feature integration as a named plugin module;
- register systems into existing runtime stages instead of creating ad-hoc loops;
- keep plugin resources, events, and components scoped to the owner when possible.

This keeps runtime wiring simple without putting runtime policy into domain crates.

## Contract-over-concrete pattern

Cross-boundary code should depend on explicit contracts:

- traits;
- command descriptors;
- DTOs;
- schemas;
- events;
- ratification reports;
- diagnostics.

Avoid reaching into private internals across crates. If deep knowledge is required, add or improve a contract at the owning boundary.

## Explicit state and scheduling pattern

- Prefer explicit resources/components and deterministic fixed-step systems.
- Use typed ECS access over hidden global mutable state.
- Apply deferred commands explicitly at stage boundaries where required.

## Error and result pattern

- Return explicit `Result` types from boundary APIs.
- Surface errors with context.
- Avoid broad best-effort swallowing in runtime, network, persistence, import, or replay paths.

## Documentation pattern

When behavior changes in a crate/domain:

1. Update that area's README or usage docs.
2. Update architecture docs when ownership or contracts change.
3. Update root summaries only when repository-wide boundaries change.

Keep docs aligned with actual workspace members.

## Abstraction rule

Add abstractions only when they reduce real duplication, protect a real boundary, or improve discoverability. Do not add speculative registries, universal objects, or generic extension surfaces before they are needed.
