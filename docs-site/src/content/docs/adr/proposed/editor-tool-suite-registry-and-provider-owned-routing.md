---
title: Editor Tool Suite Registry And Provider-Owned Routing
description: Historical proposed decision for stable tool-surface keys, typed tool-suite registration, and provider-owned local routing.
status: superseded
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-17
related_designs:
  - ../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../design/active/editor-ui-workspace-tool-surface-architecture.md
  - ../../design/active/material-lab-and-material-preview-design.md
related_adrs:
  - ../accepted/0001-use-domain-owned-commands.md
  - ../accepted/0004-separate-description-from-execution.md
  - ../accepted/0005-projections-are-derived-state.md
  - ../superseded/0006-editor-surface-provider-plugin-seam.md
  - ../accepted/0010-graph-substrate-canvas-boundary.md
---

# ADR: Editor Tool Suite Registry And Provider-Owned Routing

## Status

Superseded by
[`../superseded/0012-capability-workbench-clean-break.md`](../superseded/0012-capability-workbench-clean-break.md).

## Context

Material Lab and future graph/product tools expose repeated shell,
persistence, provider-registration, workspace-profile, routing, product, and
validation tax.

The existing provider seam already exists. ADR 0006 accepts
`ToolSurfaceInstanceId`, app-owned provider registry composition,
deterministic provider resolution, fail-closed diagnostics, and provider-local
route ownership. Current code still carries enum-backed surface persistence and
a Material Graph-specific graph-canvas routing branch in
`domain/editor/editor_shell/src/commands/map_interactions.rs::command_for_graph_canvas_action`.

Runenwerk needs stable keys and provider-owned routing before more serious
graph/product tools land. Without that, Procgen, Gameplay, Animation,
Particles, Physics, SDF, and future tool families will repeat the Material
Lab-shaped shell path.

## Decision

Introduce a typed Tool Suite Registry.

Persist registry-owned tool surfaces by stable key rather than by an
ever-growing surface enum for new suite-owned surfaces.

Route provider-local interactions through provider-owned mappers. The shell
continues to carry structural route context, surface instance identity, and
provider identity, but providers map local graph, preview, inspector, or tool
interactions into typed command proposals.

Keep compiled-in suites first. The registry is an explicit app/workbench
composition input, not a global mutable registry.

Defer external and dynamic plugin loading.

## Rejected Alternatives

### Expanding Enums And Match Arms

Rejected because it preserves the repeated tax across shell enums, persistence
enums, workspace layout matches, provider registration, graph routing, and app
workflow branches.

### Dynamic Plugins Now

Rejected because ABI, sandboxing, permissions, security, migration, package
trust, unload/reload, and compatibility are not ready for the first
extensibility step.

### Universal EditorTool God Abstraction

Rejected because it centralizes semantic ownership and blurs source truth,
surface hosting, app IO, product formation, and runtime execution.

### Shell-Owned Semantic Routing

Rejected because `editor_shell` should own workspace and host contracts, not
material, texture, procgen, gameplay, animation, particle, physics, or other
tool semantics.

### App Forks For Standalone Tools

Rejected because standalone material and UI editors should be different
Workbench Host compositions, not forks of `apps/runenwerk_editor`.

## Consequences

New product-grade tools become easier to add because they can contribute stable
surface definitions, provider families, route descriptors, command descriptors,
product/preview descriptors, and validation requirements through one typed
suite contract.

Persistence migration is required. Existing enum-backed layouts need
compatibility adapters, unknown stable keys must fail closed with diagnostics,
and registry activation alone must not rewrite saved workspace state.

Guard tests are required. They should reject duplicate stable keys, unknown
keys without diagnostics, provider ambiguity, and new shell-level graph-tool
route branches.

Standalone editors become Workbench Host compositions over installed suites,
shared app-neutral contracts, and app-owned adapters.

External plugins remain a future decision. This ADR does not accept dynamic
loading, marketplace packaging, a WASM/process/plugin ABI, or arbitrary
third-party editor mutation.
