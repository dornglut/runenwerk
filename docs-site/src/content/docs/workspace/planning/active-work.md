---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
  - ../../design/active/ui-component-platform-spatial-canvas-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../reports/investigations/phase-17-spatialcanvas-source-investigation.md
  - ../../reports/investigations/surface2d-future-pressure-branch-review.md
  - ../../reports/closeouts/phase-16-surface2d-closeout.md
  - ../../reports/investigations/phase-16-surface2d-source-investigation.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

`PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas planning/design intake is the current focus.

ID: `PT-UI-COMPONENT-PLATFORM-017`

Title: SpatialCanvas

State: active planning and design intake only. Phase 17 implementation is not authorized.

Lifecycle state: `active-planning`

Owner: workspace planning owns the active-planning state. Candidate implementation owners are `ui_controls`, `ui_runtime`, and `ui_static_mount` only after a later implementation promotion accepts exact files and scope.

Authority files: `AGENTS.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `CRATES.md`, `TESTING.md`, `GLOSSARY.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/operating-model.md`, `docs-site/src/content/docs/workspace/authority-model.md`, `docs-site/src/content/docs/workspace/documentation-structure.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/workspace/planning/README.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, `docs-site/src/content/docs/workspace/planning/completed-work.md`, `docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md`, `docs-site/src/content/docs/reports/investigations/phase-17-spatialcanvas-source-investigation.md`, `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`, `docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md`, `docs-site/src/content/docs/reports/closeouts/phase-16-surface2d-closeout.md`, and `docs-site/src/content/docs/reports/investigations/surface2d-future-pressure-branch-review.md`.

Evidence classes: `E2` repository file and PR/branch metadata inspection, `E3` source/test inspection, `E5` local command evidence for current git/GitHub state, and `E8` accepted workspace/planning/design authority.

Complete investigation gate: complete for opening Phase 17 planning/design intake. The investigation report is `docs-site/src/content/docs/reports/investigations/phase-17-spatialcanvas-source-investigation.md`. It does not authorize implementation.

Complete design gate: proposed intake only. The design is `docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md`. Implementation remains blocked until the complete design gate is reviewed and active planning records exact owner files, scope, validation, evidence expectation, and stop conditions.

Implementation contract: none authorized.

Allowed files/crates: docs and planning files for this intake only.

Non-owned files/crates: all product code, all Phase 17 implementation files, renderer backends, `ui_tree` retained-node implementation unless later accepted, `ui_render_data`/`ui_render_primitives` unless evidence proves required, `ui_input` unless evidence proves a missing normalized fact, `ui_surface` unless a separate mapping decision is accepted, `ui_composition`, `ui_graph_editor` as generic owner, domain `spatial`/`spatial_index`, editor/game/product command owners, graph/timeline implementation owners, new crates, plugin framework work, and `foundation/meta`.

Principle compliance matrix: recorded in the SpatialCanvas design for planning intake. It must be accepted again before implementation. Current status: KISS direct Surface2D -> SpatialCanvas -> runtime/static proof path proposed; DRY references Surface2D rather than duplicating it; YAGNI blocks new crates, renderer backends, plugin framework, `foundation/meta`, and speculative spatial index work; SOLID separates descriptors, proof, frame projection, and static proof; Separation of Concerns keeps product/editor/game semantics outside; Avoid Premature Optimization requires deterministic budget evidence before index/render machinery; Law of Demeter requires descriptors/reports/proof frames instead of internals.

Module decomposition map: proposed in the SpatialCanvas design. It is not implementation authorization. A later active-implementation record must name exact files and must not allow a monolithic `spatialcanvas.rs` or `spatial_canvas.rs` unless the design proves one cohesive responsibility and names a split trigger.

Maintainability review status: planning intake records expected decomposition and stop conditions. Implementation maintainability review remains blocked until exact files are accepted.

Feature support matrix: recorded in `docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md`.

Future-use-case pressure matrix: recorded in `docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md` and informed by `docs-site/src/content/docs/reports/investigations/surface2d-future-pressure-branch-review.md`.

Hierarchy/composition matrix: recorded in `docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md`.

Ergonomics/usability: recorded in `docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md`.

Validation expectation: this docs/intake PR must pass `python tools/docs/validate_docs.py` and `git diff --check`. No cargo validation is expected because this PR must not touch product code.

Known blockers: Phase 17 implementation is blocked until design review accepts the exact contract, owner files, forbidden files, validation envelope, evidence expectation, module decomposition, and stop conditions. No Phase 16 product blocker remains. PR #64 extracted the stale Surface2D future-pressure material into `docs-site/src/content/docs/reports/investigations/surface2d-future-pressure-branch-review.md`; current branch inspection shows `origin/surface2d-phase-16` is absent.

Next action: review the SpatialCanvas investigation and design intake. Do not start implementation until active planning is explicitly promoted to `active-implementation`.

Evidence: PR #62 merged docs-only principle/decomposition/merge-readiness workflow hardening at merge commit `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`. PR #61 squash-merged Phase 16 Surface2D at merge commit `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. PR #63 merged Phase 16 closeout at `53349154809bf779dba349269afeb1f3c3deb646`. PR #64 merged Surface2D future-pressure extraction at `05c51375986cf08e360884ebf44702ec62662c1e`. Current GitHub inspection reports no open PRs. Current branch inspection shows `origin/surface2d-phase-16` is absent. Phase 17 source investigation inspected `ui_controls`, `ui_runtime`, `ui_static_mount`, `ui_render_data`, `ui_render_primitives`, `ui_input`, `ui_surface`, `ui_tree`, `ui_composition`, `ui_graph_editor`, `editor_viewport`, `scene`, `spatial`, and `spatial_index`.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, stop conditions, principle compliance status, and module decomposition status are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Evidence classes:
Complete investigation gate:
Complete design gate:
Implementation contract:
Allowed files/crates:
Non-owned files/crates:
Principle compliance matrix:
Module decomposition map:
Maintainability review status:
Feature support matrix:
Future-use-case pressure matrix:
Hierarchy/composition matrix:
Ergonomics/usability:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
