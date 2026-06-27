---
title: Runenwerk Workspace
description: Short orientation map for workspace documentation.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-27
---

# Runenwerk Workspace

This page is a short orientation map.

## Active workflow

- [Start Here](start-here.md)
- [Operating Model](operating-model.md)
- [Authority Model](authority-model.md)
- [Documentation Structure](documentation-structure.md)
- [Workspace Routines](routines/README.md)
- [Planning Records](planning/README.md)

## Top-level areas

- `foundation/`: shared low-level primitives.
- `domain/`: engine-agnostic contracts, models, and invariants.
- `engine/`: runtime app loop and plugins.
- `net/`: transport, session, replication, and replay crates.
- `apps/`: runnable applications and tools.
- `adapters/`: external runtime and host integrations.
- `assets/`: data assets consumed by runtime code.
- `docs-site/`: documentation source.

## Core references

- [Architecture Boundaries](../guidelines/architecture.md)
- [Programming Principles](../guidelines/programming-principles.md)
- [Domain Overview](../domain/00-overview.md)
- [Engine Docs](../engine/index.md)
- [Net Docs](../net/README.md)
