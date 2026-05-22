---
title: WR-063 Renderer Scale Evidence And Production Readiness Closeout
description: Closeout evidence for renderer scale production evidence, benchmark coverage, hardware/capability profile reporting, and public docs.
status: completed
owner: engine
layer: engine-runtime / renderer scale
canonical: false
last_reviewed: 2026-05-22
related:
  - ../../implementation-plans/wr-063-renderer-scale-evidence-and-production-readiness/plan.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../engine/benchmarks/render-scale-evidence.md
---

# WR-063 Renderer Scale Evidence And Production Readiness Closeout

`WR-063` is complete at `runtime_proven` quality for the bounded renderer scale
production-evidence contract. The renderer now exposes a production evidence
report that keeps addressable, resident, visible, compacted, submitted,
indirect, timing, capability-profile, benchmark-command, and artifact evidence
separate.

## Changed Files

- `engine/src/plugins/render/inspect/scale_production.rs`:
  added `inspect_render_scale_production_evidence`,
  `RenderScaleProductionEvidenceRequest`,
  `RenderScaleProductionEvidenceReport`,
  `RenderScaleProductionHardwareProfile`,
  `RenderScaleProductionEvidenceCounts`,
  `RenderScaleProductionTimingEvidence`, and fail-closed diagnostics for
  missing profile, timing, benchmark, artifact, and count-invariant evidence.
- `engine/src/plugins/render/inspect/mod.rs`:
  exports the scale production evidence DTOs.
- `engine/examples/render_scale_evidence.rs` and `engine/Cargo.toml`:
  added the canonical `cargo run -p engine --example render_scale_evidence --
  --evidence` command.
- `engine/benches/render_flow_planning.rs`:
  added `render_scale/production_evidence_report_4096`.
- `engine/tests/render_scale_production_evidence.rs`:
  added runtime-chain, count-invariant, and unsupported GPU timing diagnostic
  coverage.
- `engine/benchmark-artifacts/render-scale-evidence/README.md`:
  reserves the raw artifact folder for WR-063 benchmark/profile evidence.
- `docs-site/src/content/docs/engine/benchmarks/render-scale-evidence.md`:
  documents the scale evidence and benchmark workflow.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`
  and
  `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  document the production scale evidence API and command path.

## Evidence

The example command produced a ready report:

```text
render scale evidence ready=true errors=0 warnings=0 profile=example-wgpu-portable-scale
addressable=1000000 resident=4 visible=2 compacted=2 submitted=2 indirect_commands=1 cpu_ms=0.450 gpu_ms=0.240
```

The benchmark command completed and added:

```text
render_scale/production_evidence_report_4096
time: [575.01 us 585.20 us 600.88 us]
```

Existing `render_flow_planning` benchmark comparisons still reported local
Criterion regressions against older baselines for some pre-existing cases:
`render_flow/simple_fullscreen`, `render_flow/boids_preflight_cold`, and
`render_flow/procedural_boids_preflight_cold`. They remain visible benchmark
baseline drift, not hidden pass/fail evidence.

## Validation

Passed:

```text
cargo fmt
cargo test -p engine render_scale
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow
cargo test -p engine --example render_scale_evidence
cargo run -p engine --example render_scale_evidence -- --evidence
cargo bench -p engine --bench render_flow_planning
```

Workspace validators are run after the roadmap and production metadata updates
that close this row.

## Completion Quality

Completion quality: `runtime_proven`.

Known quality gaps:

- The canonical scale evidence command uses deterministic renderer inspection
  DTOs and a capability profile; it does not submit a finite swapchain frame for
  a million-instance scene.
- Benchmark output is local Criterion evidence and not a universal FPS promise.
- Hardware-specific timestamp, readback, storage-compaction, and indirect
  submission gaps must stay explicit through diagnostics on adapters that do
  not support them.
- `perfectionist_verified` remains blocked until `PT-RENDER-PERFECTION`
  performs the final no-gap renderer audit.

The bounded production contract is satisfied because the implementation exposes
the complete renderer scale evidence chain, exercises it through tests, example
command, and benchmark command, and leaves remaining quality gaps visible for
the final renderer perfection track.
