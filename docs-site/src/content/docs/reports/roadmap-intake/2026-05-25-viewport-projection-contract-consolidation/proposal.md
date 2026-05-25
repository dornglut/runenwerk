---
title: Roadmap Intake WR-106
description: Deferred governance intake for long-term viewport and procedural projection contract consolidation.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-25
---

# Roadmap Intake WR-106

Idea: Viewport and procedural projection contract consolidation
Suggested title: Viewport And Procedural Projection Contract Consolidation
Initial planning state: `blocked_deferred`

This is a governance-only intake proposal. It must not implement renderer,
editor, UI, example, or promotion beyond `blocked_deferred`. Its purpose is to
record the long-term ownership and evidence questions before any camera,
projection, viewport, or surface-fit contract changes are promoted.

## Governance Notes

- `WR-101` is completed and remains the evidence source for the current
  renderer procedural camera proof.
- Product-surface foundation and workspace viewport-expression designs remain
  authoritative for producer, product, and presentation separation.
- `WR-102` remains the home for richer boid/flock behavior, overlap handling,
  multiple flocks, split/merge policy, and semantic flock dynamics.
- `WR-104` remains the home for game runtime UI, HUD, and world-space projected
  UI governance.
- Renderer derived projection helpers must not become camera source truth.
  `PreparedViewFrame` must remain camera-free.
- UI primitives must remain camera-free and must embed typed product or viewport
  surfaces rather than owning projection semantics.
- Future implementation must be split into bounded follow-on WRs before
  promotion.
- This proposal is rebased to `WR-106` because `WR-105` is already used by the
  UI Lab runtime evidence platform closure row.

## Ownership Boundary

- `engine/src/plugins/render/procedural/camera.rs`, module
  `engine::plugins::render::procedural::camera`, owns renderer-derived
  procedural projection helpers only.
- `domain/editor/editor_viewport/src/camera.rs`, module
  `domain::editor::editor_viewport::camera`, owns editor camera semantics and
  projection modes.
- `apps/runenwerk_editor/src/runtime/resources.rs` and
  `apps/runenwerk_editor/src/runtime/systems/picking.rs` own editor runtime
  packing and picking integration.
- `domain/ui` stays camera-free and only embeds typed product or viewport
  surfaces.

## Public API And ADR Boundary

No immediate public API changes are allowed by this intake.

A later accepted design is required before adding or changing public contracts
such as a renderer `SurfaceFitPolicy`, new procedural camera or ray contracts,
editor projection contract changes, or shared CPU/GPU editor viewport
projection helpers.

An ADR is required before moving camera source truth into `PreparedViewFrame`,
making UI own camera semantics, or introducing a shared cross-domain camera
ABI. No ADR is required if future work preserves producer-owned camera intent,
renderer-derived uniforms, and camera-free UI embedding.

## Open Questions

- Which accepted design update splits this governance intake into bounded
  renderer and editor implementation WRs?
- Should renderer presentation fit become a generic `SurfaceFitPolicy`-style
  contract, remain part of procedural camera policy, or be split into separate
  world-projection and surface-presentation contracts?
- What accepted editor viewport camera contract resolves the dormant
  `ViewportProjection::Orthographic` gap without leaking editor concepts into
  renderer or UI?
- Which shared CPU/GPU editor viewport projection helpers or tests prevent
  picking/render drift?
- Which future design, if any, justifies an ADR for cross-domain camera ABI,
  `PreparedViewFrame` camera truth, or UI projection ownership?

## Validation

Proposal-only validation:

```text
task docs:validate
task planning:validate
task roadmap:validate
task roadmap:check
task puml:validate
```

Reserve roadmap and production rendering for a later accepted
`task roadmap:apply-intake` step:

```text
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task docs:validate
task puml:validate
```

Future implementation fitness functions must include renderer aspect and
surface-fit evidence, editor CPU/GPU projection drift guards, example migration
evidence after ownership is assigned, and guards proving no viewport-specific
camera truth enters `PreparedViewFrame` or generic UI primitives.

## Deferred Boundary

This proposal depends on `WR-101` because renderer hardening has already
provided the current procedural camera proof. It should not be applied or
promoted as implementation work until ownership, ADR need, follow-on WR split,
write scopes, and validation commands are accepted.

The roadmap edge is `WR-101 -> WR-106` with label `procedural camera proof`.

The active design anchor is
`docs-site/src/content/docs/design/active/viewport-camera-and-projection-contract-platform-design.md`.

## Apply Command

```text
task roadmap:apply-intake -- --proposal docs-site/src/content/docs/reports/roadmap-intake/2026-05-25-viewport-projection-contract-consolidation/proposal.yaml
```
