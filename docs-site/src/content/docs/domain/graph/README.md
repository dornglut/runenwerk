---
title: Graph Domain
description: Current documentation for the domain-neutral graph substrate crate.
status: active
owner: graph
layer: domain
canonical: true
last_reviewed: 2026-05-09
related_docs:
  - ../../adr/accepted/0010-graph-substrate-canvas-boundary.md
---

# Graph Domain

`domain/graph` owns Runenwerk's domain-neutral graph substrate.

It defines reusable graph truth for systems that need graph-shaped authoring or validation without coupling those systems to editor surfaces, runtime rendering, material semantics, ability semantics, or scene-specific inspection behavior.

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

- graph canvas layout;
- pan, zoom, marquee, or hit testing behavior;
- material graph semantics;
- ability or skill graph semantics;
- render graph runtime execution;
- editor surface mounting;
- persistence format policy beyond the value contracts exposed by this crate.

## Boundary

Graph model truth belongs here.

Graph presentation belongs in editor/UI surface code.

Domain-specific graph meaning belongs in semantic crates such as `domain/material_graph` or future crates such as `domain/ability_graph` if those domains become real workspace members.
