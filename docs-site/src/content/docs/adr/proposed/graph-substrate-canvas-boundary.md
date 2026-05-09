---
title: Graph Substrate Canvas Boundary
description: Proposed decision separating domain-neutral graph truth from graph canvas and editor surface behavior.
status: draft
owner: workspace
layer: domain
canonical: true
last_reviewed: 2026-05-09
---

# ADR: Graph Substrate Canvas Boundary

## Status

Draft / proposed.

## Context

Runenwerk needs graph support for multiple future domains, including material graphs, ability graphs, entity query graphs, and scene relation graphs.

Starting with one concrete editor graph would overfit the model to that surface. Starting with a broad universal graph would create a god abstraction.

## Decision

Create a small domain-neutral `domain/graph` substrate first.

`domain/graph` owns graph truth only:

- graph, node, port, and edge identities
- graph, node, port, and edge definitions
- typed ports
- edge direction and compatibility rules
- traversal
- validation
- cycle policy

Graph canvas or editor surface code owns view and interaction behavior:

- pan and zoom
- selection and marquee selection
- node placement
- edge dragging
- hit testing
- presentation models
- surface intents
- workspace mounting

Domain-specific graph crates own semantic meaning:

- material graph semantics belong in `domain/material_graph`
- ability and skill graph semantics belong in a future `domain/ability_graph`
- editor-specific graph projections may use a future editor-owned module if needed

## Rejected Alternatives

Use the engine render graph as a reusable authoring graph. That graph is render-runtime specific and should not become the semantic owner for material, ability, entity, or scene authoring graphs.

Build the entity query graph or scene relation graph first. Those views are useful, but they would bias the substrate toward editor inspection instead of reusable graph truth.

## Consequences

Graph model tests can validate IDs, port compatibility, traversal, and cycle policy without any UI or editor dependencies.

Graph canvas work remains free to evolve as an editor/UI surface concern after the core UI substrate and table/query surface prove non-viewport hosting.
