---
title: PM-RENDER-RT-001 Hardware Ray Query Doctrine Closeout
description: Closeout evidence for accepting the optional renderer hardware ray-query and hybrid tracing doctrine.
status: completed
owner: engine
layer: engine-runtime / renderer ray-query doctrine
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# PM-RENDER-RT-001 Hardware Ray Query Doctrine Closeout

## Result

`PM-RENDER-RT-001` is complete at `bounded_contract` quality. The renderer
hardware ray-query doctrine is accepted and records the long-term contract for
optional capability detection, derived acceleration resources, hybrid
raster/SDF/ray-query paths, timing diagnostics, and mandatory non-RT fallback.

No product code, renderer runtime code, examples, benchmarks, shaders, or
backend handles changed for this milestone. This closeout accepts doctrine and
sequencing only; it does not authorize `WR-073`, `WR-074`, or `WR-075`
implementation without their own roadmap gates, implementation contracts,
validation, and closeouts.

## Accepted Doctrine

Accepted design:

```text
docs-site/src/content/docs/design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md
```

The design records:

- renderer ownership of backend capability reports, derived acceleration
  resources, hybrid pass scheduling, timing labels, debug labels, fallback
  diagnostics, and ray-query invocation evidence;
- producer ownership of scene, mesh, material, product, SDF, camera, exposure,
  temporal, freshness, semantic, and fallback-legality truth;
- typed contracts for supported, unsupported, disabled, and fallback capability
  states;
- invariants that keep ray-query optional, prevent backend acceleration
  resources from becoming source truth, and require visible non-RT fallback;
- downstream sequence for `WR-073`, `WR-074`, and `WR-075`.

## Governance Evidence

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "Accept renderer hardware ray query and hybrid tracing doctrine for PM-RENDER-RT-001" --scope "docs-site/src/content/docs/design/active/renderer-hardware-ray-query-and-hybrid-tracing-design.md docs-site/src/content/docs/workspace/production-tracks.yaml docs-site/src/content/docs/workspace/roadmap-deferred.yaml"
```

Governance decision:

- Bounded context owner: `engine/src/plugins/render`.
- Team Topologies owner: complicated-subsystem renderer platform consuming
  stream-aligned scene, mesh, material, product, SDF, temporal, camera, and
  exposure producers.
- ADR requirement: no ADR for doctrine acceptance because dependency direction
  preserves accepted Render Product Graph, scale, SDF, temporal, and renderer
  evidence boundaries. ADR is required before a later implementation persists a
  cross-domain acceleration-resource ABI, moves producer truth or fallback
  authority into the renderer, or makes RT hardware a baseline requirement.
- Next action after closeout: use the stack coordinator and only apply/promote
  the first legal implementation WR when its intake, dependencies, write
  scopes, and validations are ready.

## Metadata Evidence

Updated source files:

- `docs-site/src/content/docs/design/active/README.md`: removed the renderer
  hardware ray-query design from active designs.
- `docs-site/src/content/docs/design/accepted/README.md`: added the renderer
  hardware ray-query design to accepted designs.
- `docs-site/src/content/docs/design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md`:
  accepted and expanded the doctrine.
- `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`
  and
  `docs-site/src/content/docs/design/active/renderer-production-audit-and-perfectionist-verification-design.md`:
  retargeted related-design evidence to the accepted RT design.
- `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-RT-001`:
  marked the doctrine milestone completed and added this closeout as evidence.
- Downstream RT production milestones now require the accepted RT design path
  and this doctrine closeout before implementation.

## Validation

Design validation after metadata updates:

```text
task docs:validate
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- Ray-query capability detection and acceleration-resource inspection remain
  blocked until `WR-073` is applied, promoted, implemented, validated, and
  closed.
- Hybrid raster/SDF/ray-query runtime proof remains blocked until `WR-074` is
  applied, promoted, implemented, validated, and closed.
- Ray-query production evidence remains blocked until `WR-075` is applied,
  promoted, implemented, validated, and closed.
- This design-only milestone does not claim `runtime_proven` or
  `perfectionist_verified`.

These gaps are expected sequencing boundaries for `PT-RENDER-RT`, not hidden
completion defects in `PM-RENDER-RT-001`.
