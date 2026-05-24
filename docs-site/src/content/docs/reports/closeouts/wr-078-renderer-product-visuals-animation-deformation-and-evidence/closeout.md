---
title: WR-078 Renderer Product Visuals Animation Deformation And Evidence Closeout
description: Closeout evidence for animation/deformation producer handoff and cross-family product visual runtime evidence.
status: completed
owner: engine
layer: engine-runtime / product-render-integration
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/accepted/renderer-product-visual-producers-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
related_reports:
  - ../../benchmarks/render/product-visual-evidence.md
---

# WR-078 Renderer Product Visuals Animation Deformation And Evidence Closeout

## Result

`WR-078` is complete at `runtime_proven` quality for the product visual producer
track. The renderer now exposes cross-family evidence that aggregates
particle/VFX, world visual, and deformation handoff families without making the
renderer authoritative for product animation, VFX, vegetation, water,
atmosphere, weather, field, freshness, fallback, or authoring truth.

The implementation adds:

- `engine/src/plugins/render/inspect/product_visual_evidence.rs`: typed
  renderer-owned product visual evidence DTOs, family evidence conversion,
  deformation stream handoff evidence, fail-closed diagnostics, artifact path
  checks, benchmark command checks, and ownership-boundary checks.
- `engine/src/plugins/render/inspect/mod.rs`: public exports for the product
  visual evidence inspection API.
- `engine/tests/render_product_visual_evidence.rs`: focused tests for ready
  cross-family evidence, missing family evidence, fallback-only claims,
  renderer-owned product truth leaks, missing artifacts, and invalid
  deformation handoff streams.
- `engine/examples/render_product_visual_evidence.rs`: public example that
  prints the canonical deterministic product visual evidence summary.
- `engine/Cargo.toml`: example registration.
- `engine/benchmark-artifacts/render-product-visual-evidence/README.md`:
  raw/reproducible artifact folder for product visual evidence.
- `docs-site/src/content/docs/reports/benchmarks/render/product-visual-evidence.md`:
  human-readable evidence report and example output.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  public API notes for product visual evidence and deformation handoff
  ownership boundaries.

## Boundary Evidence

- Particle/VFX and world visual families remain product-owned prepared payloads
  consumed through renderer registered payload inspection.
- Deformation evidence uses `PreparedDeformationFeatureContribution` and
  `PreparedDeformationStream` only as renderer handoff evidence. Pose,
  skeleton, animation graph, cloth, product freshness, and fallback legality
  remain product-domain concerns.
- The renderer evidence report fails closed when representative particle/VFX,
  world visual, or deformation family evidence is missing.
- The report fails closed for missing prepared batches or streams, unconsumed
  handoff evidence, fallback-only claims, over-budget or unsupported states,
  missing benchmark commands, missing raw artifact paths, missing human report
  paths, and any renderer-owned product-truth claim.
- Runtime proof is a renderer execution evidence claim backed by public tests,
  a public example, deterministic artifact/report references, and docs. It is
  not a backend visual quality, hardware certification, or product authoring
  completeness claim.
- No ADR was required because the slice stays inside accepted renderer evidence
  and producer-boundary contracts and does not introduce a durable animation
  ABI or move product authority into renderer policy.

## Validation

Focused validation passed:

```text
cargo fmt --check
cargo test -p engine render_product_visual_particles
cargo test -p engine render_product_visual_world
cargo test -p engine render_product_visual_evidence
cargo test -p engine render_runtime_inspect
cargo test -p engine --example render_product_visual_evidence
cargo run -p engine --example render_product_visual_evidence
```

The new product visual evidence test filter passed four tests:

- ready evidence covers particle/VFX, world visual, and deformation families;
- missing required family evidence fails closed;
- fallback-only claims and renderer-owned product truth fail closed;
- missing artifact/report references and invalid deformation streams fail
  closed.

The runtime example printed:

```text
render product visual evidence runtime_proven=true errors=0 warnings=0 families=3
particle_batches=2 world_batches=2 deformation_streams=1 residency_requests=2 temporal_inputs=7
```

## Completion Quality

Completion quality: `runtime_proven`.

Known quality gaps:

- `perfectionist_verified` remains blocked until `PT-RENDER-PERFECTION`
  performs the final no-gap renderer audit.
- Product-domain animation, VFX, vegetation, water, atmosphere, weather, field,
  freshness, fallback legality, and authoring workflows remain outside renderer
  ownership.
- Hardware-specific visual quality, vendor certification, and backend-specific
  product visual captures remain outside this bounded product visual evidence
  slice unless future evidence explicitly proves them.
