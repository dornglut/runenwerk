---
title: Renderer Scale Evidence
description: Runtime and benchmark evidence contract for renderer scale production readiness.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-22
related:
  - ../reference/plugins/render/render-flow-usage-guide.md
  - ../reference/plugins/render/public-api-reference.md
---

# Renderer Scale Evidence

WR-063 scale evidence uses `inspect_render_scale_production_evidence(...)` to
keep the renderer scale chain explicit:

- addressable, selected, accepted, resident, resident-byte, and upload-byte
  evidence from GPU residency inspection;
- visible, culled, compacted, submitted, and indirect command counts from scale
  visibility inspection;
- CPU pass timings, GPU timestamp timings when supported, and typed unsupported
  timing diagnostics when unavailable;
- hardware or capability profile identity, benchmark commands, and artifact
  paths.

Run the focused evidence command with:

```text
cargo run -p engine --example render_scale_evidence -- --evidence
```

Run the planning and evidence aggregation benchmarks with:

```text
cargo bench -p engine --bench render_flow_planning
```

Raw benchmark outputs and local hardware-profile artifacts belong under:

```text
engine/benchmark-artifacts/render-scale-evidence
```

Do not treat a benchmark run as a universal FPS promise. A valid report must
name its hardware or capability profile and keep unsupported timestamp,
readback, storage compaction, or indirect-submission states visible.
