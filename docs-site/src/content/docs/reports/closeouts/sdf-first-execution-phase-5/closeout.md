---
title: SDF-First Execution Phase 5 Closeout
description: Completion and drift-check record for the procgen readiness gate.
status: completed
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-13
related_designs:
  - ../../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
related_roadmaps:
  - ../../../workspace/sdf-first-execution-roadmap.md
  - ../../../workspace/repo-execution-priority-checklist.md
  - ../../../workspace/roadmap-index.md
related_domains:
  - ../../../domain/procgen/README.md
  - ../../../domain/product/README.md
  - ../../../domain/graph/README.md
  - ../../../domain/world-ops/README.md
  - ../../../domain/world-sdf/README.md
---

# SDF-First Execution Phase 5 Closeout

## Status

Complete as of 2026-05-13 for the procgen readiness gate.

This closeout accepts the procgen domain contract only. It does not add
`domain/procgen` code, a Cargo workspace member, generator algorithms, preview
execution, bake execution, worker pools, editor providers, or product runtime
implementation.

## Completion Evidence

- `docs-site/src/content/docs/domain/procgen/README.md` now defines the
  accepted future `domain/procgen` ownership boundary.
- The accepted descriptor shape is a typed `ProcgenDocument` over
  `domain/graph`, with procgen-owned node semantics, ratification, lowering,
  diagnostics, source maps, and cache lineage.
- The accepted procgen lifecycle covers prototypes, candidates, reservations,
  instance plans, realization, and runtime-state handoff without declaring a
  generic content-planning platform.
- The first future product track is bounded region terrain/material generation
  that lowers to deterministic `world_ops::OperationRecord` windows plus
  changed-region diagnostics.
- Runtime preview and offline bake share the same future product-job/lowering
  contract while differing by budget, retention, and command boundary.
- Generated bases remain separate from authored `world_ops` overlays, and
  regeneration conflicts require explicit diagnostics instead of silent edit
  replacement.
- Multiplayer policy is server-validated deterministic generation: clients may
  reproduce generation, but authoritative sessions validate and publish product
  generations or operation windows.

## Drift Corrections

- The SDF-first execution roadmap now records Phase 5 as complete and names
  Phase 6 / M6.2 as the next first procgen product track.
- The repository priority checklist now allows M6.2 procgen code only from the
  accepted procgen contract instead of saying readiness still blocks it.
- The roadmap index now lists Phase 5 among finished cross-track baselines.
- The editor procedural workflow plan now treats the procgen domain README as
  accepted while keeping provider/runtime execution deferred.
- The deferred procgen design draft now points to the accepted procgen domain
  contract as the implementation authority.
- The procgen contract now records promotion criteria for extracting shared
  content-planning contracts only after repeated non-procgen demand exists.

## Deferred Work

- `domain/procgen` crate/code, ratifiers, descriptors, lowering, deterministic
  replay tests, and product-job integration.
- Procgen graph canvas and preview providers.
- Bake-to-`world_ops`, bake-to-field-product commands, rollback, changed-region
  diagnostics, and concrete terrain/material/cave/stamp/scatter algorithms.
- Gameplay graph, particles, physics, animation, and world-process domains.

## Validation

All listed docs-only validation passed on 2026-05-13. Cargo tests and full gate
were not required because this phase changed documentation contracts only and
did not change code, manifests, or workspace behavior.

Validation run:

- `python3 tools/docs/validate_docs.py`
- `rg -n "procgen readiness.*gate|Phase 5|M6\\.2|domain/procgen" docs-site/src/content/docs`
- `./workflow closeout --task "SDF-first execution roadmap Phase 5 - Procgen Readiness Gate" --roadmap "docs-site/src/content/docs/workspace/sdf-first-execution-roadmap.md"`

Additional stale-language check:

- `rg -n 'no \`domain/procgen\`$|There is no \`domain/procgen\`|procgen readiness.*still|still gate M6\\.2|blocked on.*accepted procgen domain doc|Do not start M6\\.2 procgen code until|Phases 1 through 4 are complete|current work is Phase 5|immediate priority is no longer M6\\.2' docs-site/src/content/docs/workspace docs-site/src/content/docs/apps docs-site/src/content/docs/design docs-site/src/content/docs/domain`
