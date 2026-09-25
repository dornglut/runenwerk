---
title: Graph Substrate Canvas Boundary
description: Decision separating Runenwerk-authored port-graph authority from graph canvas and editor surface behavior.
status: accepted
owner: workspace
layer: domain
canonical: true
last_reviewed: 2026-09-25
related_docs:
  - ../../domain/graph/README.md
related_designs:
  - ../../design/active/semantic-graph-ir-and-compilation-design.md
preserved_from:
  - ../superseded/proposed-graph-substrate-canvas-boundary.md
---

# ADR: Graph Substrate Canvas Boundary

## Status

Accepted.

This ADR preserves and accepts the former proposed graph substrate boundary
decision. The original information is intentionally retained here instead of
being left only in history.

## Context

Runenwerk needs graph support for multiple domains, including material graphs,
ability graphs, entity query graphs, and scene relation graphs.

Starting with one concrete editor graph would overfit the model to that surface.
Starting with a broad universal graph would create a god abstraction.

`domain/graph` now exists as Runenwerk's local authored structural port-graph
substrate. The accepted long-term boundary is that this crate owns structural
graph/document semantics shared by Runenwerk domains, not organization-wide
reusable graph-framework authority, semantic graph meaning, or editor canvas
state.

## Decision

`domain/graph` owns Runenwerk-authored port-graph structure only:

- graph, node, port, and edge identities;
- graph, node, port, and edge definitions;
- typed ports;
- edge direction and compatibility rules;
- traversal;
- validation;
- cycle policy.

The graph, node, port, and edge identities above are authored document identities.
Owning domain formats may persist them. Their existence does not make this crate the
Runen-family reusable relationship framework.

Graph canvas or editor surface code owns view and interaction behavior:

- pan and zoom;
- selection and marquee selection;
- node placement;
- edge dragging;
- hit testing;
- presentation models;
- surface intents;
- workspace mounting.

Domain-specific graph crates own semantic meaning:

- material graph semantics belong in `domain/material_graph`;
- ability and skill graph semantics belong in a future `domain/ability_graph`;
- editor-specific graph projections may use an editor-owned module if needed.

Standalone RunenGraph owns reusable graph and relationship semantics over
caller-owned identities under Engineering ADR 0010. This ADR does not make
`domain/graph` a RunenGraph implementation, predecessor, source authority, or
implicit consumer. Any future RunenGraph adoption requires separate consumer-owned
work and must preserve one invariant set per semantic authority.

## Rejected Alternatives

Use the engine render graph as a reusable authoring graph. That graph is
render-runtime specific and should not become the semantic owner for material,
ability, entity, or scene authoring graphs.

Build the entity query graph or scene relation graph first. Those views are
useful, but they would bias the substrate toward editor inspection instead of
the authored port-graph requirements shared by Runenwerk domains.

## Consequences

Graph model tests can validate IDs, port compatibility, traversal, and cycle
policy without any UI or editor dependencies.

Graph canvas work remains free to evolve as an editor/UI surface concern after
the core UI substrate and table/query surface prove non-viewport hosting.

Semantic graph implementation must start from one concrete owning domain and one
formed product target. `domain/graph` must not grow into a semantic graph
language, universal graph runtime, or editor-owned runtime authority.

<!-- BEGIN RUNENWERK:UI_COMPONENT_PLATFORM:adr-consumption -->
## Component Platform consumption note

`PT-UI-COMPONENT-PLATFORM` consumes this ADR without changing its decision. The component platform makes the accepted interaction/canvas boundaries reusable and story-proven through `ControlPackage`s, `Surface2D`, `SpatialCanvas`, `NodeCanvas`, `PortGraphCanvas`, `ProgressionTreeView`, `TrackSurface`, Gallery proof, Workbench adoption, and UI Designer consumption. This ADR remains the authority for the underlying boundary; Component Platform docs define reusable package contracts and proof requirements.
<!-- END RUNENWERK:UI_COMPONENT_PLATFORM:adr-consumption -->
