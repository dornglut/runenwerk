---
title: Graph Domain
description: Current documentation for Runenwerk's authored port-graph substrate.
status: active
owner: graph
layer: domain
canonical: true
last_reviewed: 2026-09-25
publication: primary
related_docs:
  - ../../adr/accepted/0010-graph-substrate-canvas-boundary.md
---

# Graph Domain

`domain/graph` owns Runenwerk's local authored port-graph substrate.

It defines the graph, node, port, and edge structure used by Runenwerk domains that need authored graph documents or structural validation without coupling those domains to editor surfaces, runtime rendering, material semantics, ability semantics, or scene-specific inspection behavior. It is not the organization-wide reusable graph/relationship framework authority.

## Ownership

This crate owns:

- graph identity;
- node identity;
- port identity;
- edge identity;
- port direction;
- port type compatibility;
- graph definitions;
- graph validation;
- traversal helpers;
- directed cycle policy.

## Non-goals

This crate does not own:

- organization-wide reusable graph or relationship semantics over caller-owned identities; standalone RunenGraph owns that framework boundary;

- graph canvas layout;
- pan, zoom, marquee, or hit testing behavior;
- material graph semantics;
- ability or skill graph semantics;
- render graph runtime execution;
- editor surface mounting;
- persistence format policy beyond the value contracts exposed by this crate.

## Boundary

Runenwerk-authored port-graph structure belongs here.

Standalone reusable caller-key graph/relationship semantics belong to RunenGraph and are not implied by this crate. `domain/graph` is not a RunenGraph implementation, predecessor, source authority, or implicit consumer.

Graph presentation belongs in editor/UI surface code.

Domain-specific graph meaning belongs in semantic crates such as `domain/material_graph` or future crates such as `domain/ability_graph` if those domains become real workspace members.
