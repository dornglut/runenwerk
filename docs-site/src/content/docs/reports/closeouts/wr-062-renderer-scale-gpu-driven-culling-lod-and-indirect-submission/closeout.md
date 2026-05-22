---
title: WR-062 Renderer Scale GPU Driven Culling LOD And Indirect Submission Closeout
description: Closeout evidence for renderer-owned visible working-set, compaction, unsupported capability, and indirect submission diagnostics.
status: completed
owner: engine
layer: engine-runtime / renderer scale
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-062 Renderer Scale GPU Driven Culling LOD And Indirect Submission Closeout

## Outcome

`WR-062` is complete at `bounded_contract` quality. The renderer now has a
backend-neutral scale visibility inspection surface that distinguishes resident
candidates, visible candidates, culled candidates, compacted candidates,
submitted draw counts, and indirect command counts. Unsupported storage
compaction or indirect submission produces diagnostics and zero submitted work
instead of silently falling back to per-entity CPU submission.

The row does not claim runtime scale proof or `perfectionist_verified`.
Production examples, benchmarks, hardware profiles, and final scale readiness
evidence remain WR-063 scope.

## Implementation Evidence

Changed modules:

- `engine/src/plugins/render/inspect/scale_visibility.rs::RenderScaleVisibilityCapabilities`:
  records storage-compaction and indirect-submission capability state.
- `engine/src/plugins/render/inspect/scale_visibility.rs::RenderScaleVisibilityConfig`:
  records renderer-owned frustum, screen-size LOD, and compaction-budget
  thresholds without owning product semantic LOD.
- `engine/src/plugins/render/inspect/scale_visibility.rs::RenderScaleVisibilityCandidate`:
  carries renderer-resident candidate identity, bounds, screen-size, and byte
  evidence for visibility evaluation.
- `engine/src/plugins/render/inspect/scale_visibility.rs::inspect_render_scale_visibility`:
  evaluates visible, culled, compacted, submitted, and indirect command counts
  with explicit unsupported diagnostics and no CPU fallback.
- `engine/src/plugins/render/inspect/mod.rs`: exports the scale visibility DTOs.
- `engine/tests/render_scale_visibility.rs`: proves visible/culled/LOD/indirect
  counts, unsupported indirect fail-closed behavior, and compaction-budget
  submitted-work bounds.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  documents scale visibility and indirect-submission evidence.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents the public scale visibility DTOs and product-policy boundary.

## Validation

Focused validation:

```text
cargo fmt
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
cargo test -p engine render_flow
```

Workflow validation after roadmap and production metadata updates:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- WR-062 provides deterministic renderer inspection evidence, not hardware
  command-buffer execution proof.
- Production scale examples, hardware profiles, benchmarks, and final scale
  readiness evidence remain WR-063 scope.
- Renderer LOD bands are execution buckets only; product semantic LOD,
  streaming, fallback, freshness, authority, and visibility truth remain
  product-owned.
- `perfectionist_verified` remains blocked until the final renderer perfection
  audit verifies the completed production stack with no known quality gaps.

These gaps are sequencing boundaries for the renderer scale track, not hidden
defects in the bounded WR-062 implementation.
