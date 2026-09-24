---
title: Mesh Material Production Evidence Benchmark Report
description: WR-069 renderer mesh/material production evidence benchmark and artifact summary.
status: active
owner: engine
layer: engine-runtime / renderer mesh material production evidence
canonical: false
last_reviewed: 2026-05-23
related:
  - ../../implementation-plans/wr-069-renderer-mesh-material-production-evidence/plan.md
  - ../../closeouts/wr-069-renderer-mesh-material-production-evidence/closeout.md
---

# Mesh Material Production Evidence Benchmark Report

## Command

```text
cargo bench -p engine --bench render_flow_planning
```

## Artifact References

- Raw artifact summary:
  `engine/benchmark-artifacts/render-mesh-material-production-evidence/summary.txt`
- Artifact folder:
  `engine/benchmark-artifacts/render-mesh-material-production-evidence/README.md`

## Evidence Summary

The WR-069 production evidence path consumes the renderer-owned material
handoff inspection from WR-067 and the pipeline/fallback inspection from
WR-068. The deterministic artifact summary records one material instance, one
material-consuming pass, one pipeline-backed pass, and nonzero rendered pixel
evidence. It is a portable artifact reference, not a source of material,
asset, model, scene, product, shader, or fallback truth.

## Benchmark Summary

The focused WR-069 benchmark case completed in the canonical render-flow
planning suite:

```text
render_mesh_material/production_evidence_report
                        time:   [1.8383 us 1.9200 us 2.0127 us]
```

The full benchmark command also records local baseline movement in pre-existing
render-flow cases. Those comparisons are local Criterion evidence, not a
universal FPS claim or hidden pass/fail threshold.
