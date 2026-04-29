---
title: "Runenwerk Workspace"
description: "High-level map of the active Runenwerk workspace."
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# Runenwerk Workspace

This page is a lightweight orientation map for the active Runenwerk repository.

## Top-Level Areas

- `foundation/`: shared low-level primitives used across domains (for example typed ids)
- `domain/`: engine-agnostic gameplay/runtime contracts and logic
- `engine/`: runtime app loop and plugins
- `net/`: networking and replication crates
- `apps/`: runnable applications and tools
- `adapters/`: external engine/runtime integration bridges
- `assets/`: data assets consumed by runtime code
- `docs-site/`: documentation source

## Start Here

- Workspace agent rules: [`agents.md`](agents.md)
- Workspace routines: [`routines/README.md`](routines/README.md)
- Workspace roadmap index: [`roadmap-index.md`](roadmap-index.md)
- Prompt templates: [`prompt-templates/README.md`](prompt-templates/README.md)
- Documentation templates: [`templates/README.md`](templates/README.md)
- Docs root index: [`../index.mdx`](../index.mdx)
- Canonical architecture doctrine: [`../guidelines/runenwerk-architecture.md`](../guidelines/runenwerk-architecture.md)
- Workspace architecture boundaries: [`../guidelines/architecture.md`](../guidelines/architecture.md)
- Crate documentation status: [`crate-docs-status.md`](crate-docs-status.md)
- Domain docs: [`../domain/00-overview.md`](../domain/00-overview.md)
- Engine docs: [`../engine/index.md`](../engine/index.md)
- Net docs: [`../net/README.md`](../net/README.md)
