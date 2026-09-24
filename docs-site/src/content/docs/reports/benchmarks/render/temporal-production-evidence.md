---
title: Renderer Temporal Production Evidence Benchmark
description: WR-072 temporal production evidence benchmark and artifact summary.
status: active
owner: engine
layer: engine-runtime / renderer temporal production evidence
canonical: false
last_reviewed: 2026-05-23
related:
  - ../../../../engine/reference/plugins/render/public-api-reference.md
  - ../../closeouts/wr-072-renderer-temporal-production-evidence/closeout.md
---

# Renderer Temporal Production Evidence Benchmark

WR-072 adds a bounded temporal production evidence path. The benchmark exercises
the renderer-owned evidence aggregation for temporal inputs, upscaling/ray
inputs, visual evidence references, timing diagnostics, benchmark commands, and
artifact paths.

## Raw Artifacts

- `engine/benchmark-artifacts/render-temporal-production-evidence/summary.txt`
- `engine/benchmark-artifacts/render-temporal-production-evidence/README.md`

## Command

```text
cargo bench -p engine --bench render_flow_planning
```

## Focused Case

The focused Criterion case is:

```text
render_temporal/production_evidence_report
                        time:   [3.2476 us 3.2756 us 3.3040 us]
```

The evidence path is intentionally portable. Unsupported GPU timestamp queries
are recorded as typed diagnostics instead of being collapsed into missing data
or success-shaped GPU timing.
