---
title: SDF Runtime Evidence
description: Runtime and benchmark evidence contract for sparse SDF world rendering.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-22
related:
  - ../reference/plugins/render/render-flow-usage-guide.md
  - ../reference/plugins/render/public-api-reference.md
---

# SDF Runtime Evidence

WR-066 SDF runtime evidence uses
`inspect_render_sdf_production_evidence(...)` to keep the sparse SDF runtime
chain explicit:

- selected and resident SDF products, pages, bricks, clipmap windows, resident
  bytes, upload bytes, and residency budget status;
- distance mips, safe-step bounds, candidate-list counts, rejected candidates,
  and max ray-step evidence from SDF raymarch acceleration;
- near, mid, far, and summary visual evidence references;
- CPU pass timings, GPU timestamp timings when supported, and typed unsupported
  timing diagnostics when unavailable;
- hardware or capability profile identity, benchmark commands, and artifact
  paths.

Run the canonical SDF example evidence command with:

```text
cargo run -p engine --example sdf_render_flow -- --evidence
```

Run the standalone evidence command with:

```text
cargo run -p engine --example render_sdf_runtime_evidence -- --evidence
```

Run the planning and runtime evidence aggregation benchmarks with:

```text
cargo bench -p engine --bench render_flow_planning
```

Raw benchmark outputs, local visual proof references, and local hardware-profile
artifacts belong under:

```text
engine/benchmark-artifacts/render-sdf-runtime-evidence
```

Do not treat a benchmark run as a universal FPS promise. A valid report must
name its hardware or capability profile and keep unsupported timestamp or
capture states visible through diagnostics.
