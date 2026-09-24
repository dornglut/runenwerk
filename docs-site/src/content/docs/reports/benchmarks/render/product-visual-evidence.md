---
title: Product Visual Evidence Benchmark Report
description: WR-078 renderer product visual producer evidence benchmark and artifact summary.
status: active
owner: engine
layer: engine-runtime / renderer product visual evidence
canonical: false
last_reviewed: 2026-05-24
related:
  - ../../implementation-plans/wr-078-renderer-product-visuals-animation-deformation-and-evidence/plan.md
  - ../../closeouts/wr-078-renderer-product-visuals-animation-deformation-and-evidence/closeout.md
---

# Product Visual Evidence Benchmark Report

## Command

```text
cargo bench -p engine --bench render_flow_planning
```

## Artifact References

- Raw artifact folder:
  `engine/benchmark-artifacts/render-product-visual-evidence/README.md`
- Public example:
  `cargo run -p engine --example render_product_visual_evidence`

## Evidence Summary

The WR-078 product visual evidence path consumes renderer-owned inspection
evidence for representative particle/VFX, world visual, and deformation
handoff families. The deterministic example records prepared particle/VFX
batches, world visual batches, deformation streams, residency request counts,
temporal input counts, benchmark commands, raw artifact paths, and this human
report path.

This is renderer execution evidence only. Product animation graphs, VFX
semantics, vegetation/water/weather/field authoring, product freshness,
fallback legality, and source ownership remain product-domain concerns.

## Runtime Example Summary

```text
render product visual evidence runtime_proven=true errors=0 warnings=0 families=3
particle_batches=2 world_batches=2 deformation_streams=1 residency_requests=2 temporal_inputs=7
```
