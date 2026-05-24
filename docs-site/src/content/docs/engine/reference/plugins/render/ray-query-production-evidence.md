---
title: Renderer Ray Query Production Evidence
description: Production evidence packet for optional renderer ray-query support, fallback behavior, and diagnostics.
status: active
owner: engine
layer: engine-runtime / renderer optional ray-query production evidence
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../../design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md
related_roadmaps:
  - ../../../../workspace/roadmap-archive.yaml
  - ../../../../workspace/production-tracks.yaml
---

# Renderer Ray Query Production Evidence

This evidence packet closes `WR-075` for optional ray-query production support.
Ray-query is a capability-gated renderer path; raster, SDF raymarch, temporal
reconstruction, and visible non-RT fallback remain the portable baseline.

## Evidence Sources

- Accepted doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md`.
- Capability/resource inspection:
  `docs-site/src/content/docs/reports/closeouts/wr-073-renderer-ray-query-capability-and-acceleration-resources/closeout.md`.
- Hybrid proof and fallback:
  `docs-site/src/content/docs/reports/closeouts/wr-074-renderer-hybrid-ray-sdf-raster-runtime-proof/closeout.md`.
- Runtime proof command:
  `cargo run -p engine --example render_hybrid_ray_sdf_raster_runtime_proof`.

## Capability Matrix

| State | Required evidence | Production behavior |
| --- | --- | --- |
| Supported | `RenderRayQueryCapabilityState::Supported`, ready BLAS/TLAS lineage, memory evidence, build version, and no backend-handle exposure | Ray-query invocation may be scheduled as optional hybrid work. |
| Unsupported | Typed unsupported reason and `native_fallback_visible = true` | Ray-query invocation is disabled and the renderer must expose the non-RT fallback path. |
| Disabled | Renderer policy reason and visible fallback | Ray-query invocation remains disabled even if hardware could support it. |
| Pending/degraded | Pending capability, unavailable timestamp/readback diagnostics, or readback-pending diagnostics | Evidence is retained as degraded; production claims must not treat missing data as success. |

## Acceleration Resource Evidence

Derived acceleration resources are renderer execution evidence, not source
truth. Public inspection may expose:

- stable debug labels;
- bottom-level and top-level resource kind;
- source lineage, product generation, and cache identity;
- memory estimates and build version;
- stale, missing-source, disabled, unsupported, or over-budget diagnostics.

Public inspection must not expose mutable backend handles. Scene, mesh,
material, SDF, temporal, camera, exposure, fallback legality, and product
freshness remain producer-owned.

## Fallback Evidence

The portable proof command reports:

```text
render hybrid ray/SDF/raster proof ready=true errors=0 warnings=5
raster_passes=1 sdf_ready=true temporal_ready=true ray_query_supported=true fallback_visible=true timing_passes=5 timing_labels=raster,sdf,temporal,ray_query,fallback
```

This output proves the production packet can distinguish a supported synthetic
ray-query path from the visible non-RT fallback path. The warning count is
expected because portable profiles explicitly report unsupported or unavailable
timestamp/ray-query states instead of hiding them.

## Timing And Diagnostics

Production evidence keeps pass labels separated:

- `raster`
- `sdf`
- `temporal`
- `ray_query`
- `fallback`

Unsupported timestamp queries, unavailable GPU timing data, readback-pending
data, unsupported ray-query capability, stale resources, missing lineage, and
over-budget acceleration resources are diagnostics. They are not collapsed into
empty success data.

## Validation

The production packet is reproducible with:

```text
cargo test -p engine render_ray_query
cargo test -p engine --example render_hybrid_ray_sdf_raster_runtime_proof
cargo run -p engine --example render_hybrid_ray_sdf_raster_runtime_proof
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Quality Claim

Completion quality is `runtime_proven` for optional ray-query production
evidence with mandatory fallback. This does not claim mandatory RT hardware,
vendor hardware certification, denoising quality, shader-table support, or final
`perfectionist_verified` status.
