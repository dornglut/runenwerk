---
title: WR-066 SDF World Runtime Evidence Closeout
description: Closeout evidence for sparse SDF runtime examples, visual proof references, benchmarks, timing, residency, and raymarch production evidence.
status: completed
owner: engine
layer: engine-runtime / renderer sdf runtime evidence
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-066-sdf-world-runtime-evidence/plan.md
---

# WR-066 SDF World Runtime Evidence Closeout

## Result

`WR-066` is complete at `runtime_proven` quality. The renderer now has a
closeout-ready SDF runtime evidence path that aggregates completed SDF
residency and raymarch evidence with executable example output, visual proof
references, benchmark commands, artifact paths, CPU timing, explicit GPU timing
diagnostics, hardware/capability profile identity, and fail-closed production
readiness diagnostics.

This row does not claim final `perfectionist_verified` quality. The final
no-gap renderer audit remains `PT-RENDER-PERFECTION` scope after the remaining
renderer dependency tracks close.

## Changed Modules

- `engine/src/plugins/render/inspect/sdf_production.rs`:
  added `RenderSdfProductionEvidenceRequest`,
  `RenderSdfProductionEvidenceReport`,
  `RenderSdfProductionHardwareProfile`, `RenderSdfRuntimeVisualEvidence`,
  count/timing DTOs, diagnostics, and
  `inspect_render_sdf_production_evidence`.
- `engine/src/plugins/render/inspect/mod.rs`:
  exported the SDF production evidence inspection surface.
- `engine/examples/sdf_render_flow/main.rs`:
  added the `--evidence` command path.
- `engine/examples/sdf_render_flow/rendering/evidence.rs`:
  added the canonical SDF runtime evidence report builder. It compiles the SDF
  render flow, derives example SDF residency and raymarch acceleration evidence,
  records CPU timing and explicit unsupported GPU timing diagnostics, and
  formats closeout-ready text output.
- `engine/examples/sdf_render_flow/rendering/mod.rs`:
  exported the SDF evidence module through the example rendering boundary.
- `engine/examples/render_sdf_runtime_evidence.rs`:
  added a standalone non-windowed SDF runtime evidence command.
- `engine/Cargo.toml`:
  registered the `render_sdf_runtime_evidence` example.
- `engine/tests/render_sdf_runtime_evidence.rs`:
  added focused fail-closed tests for ready runtime evidence, missing
  visual/benchmark evidence, and raymarch/residency count drift.
- `engine/benches/render_flow_planning.rs`:
  added `render_sdf/runtime_evidence_report_4096` to benchmark SDF production
  evidence aggregation.
- `engine/benchmark-artifacts/render-sdf-runtime-evidence/README.md`:
  added the raw artifact location for WR-066 SDF runtime evidence.
- `docs-site/src/content/docs/engine/benchmarks/render-sdf-runtime-evidence.md`:
  added the human-readable SDF runtime evidence benchmark and artifact guide.
- `docs-site/src/content/docs/engine/benchmarks/render-flow-planning.md`:
  documented the scale and SDF production evidence benchmark cases.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documented the SDF production evidence public DTOs and fail-closed contract.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  documented normal SDF runtime evidence commands and artifact placement.

## Architecture Evidence

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "WR-066 SDF World Runtime Evidence" --scope "PM-RENDER-SDF-004 engine/examples engine/benches engine/src/plugins/render SDF runtime examples visual proof benchmarks GPU timing residency raymarch diagnostics production evidence"
```

Governance decision:

- DDD bounded context owner: `engine/src/plugins/render` owns renderer execution
  evidence, SDF residency/raymarch inspection DTOs, production evidence
  aggregation, timing diagnostics, benchmark harnesses, and example render
  flows.
- Runtime example owner: `engine/examples/sdf_render_flow` owns the canonical
  executable SDF evidence command and user-visible example wiring.
- Source truth owner: `domain/world_sdf`, product publication, and SDF product
  producers own SDF payload truth, query policy, fallback legality, collision
  semantics, generation authority, and source freshness.
- Translation boundary: WR-066 consumes renderer-derived residency and raymarch
  evidence. Example payloads and visual artifact references are runtime
  evidence inputs, not canonical SDF product truth.
- ADR decision: no ADR required. The implementation adds renderer-owned
  evidence aggregation, examples, benchmark cases, and docs without changing
  dependency direction or moving SDF product/collision/query/fallback policy
  into renderer code.

## Evidence

The implementation provides explicit runtime evidence for:

- selected and resident SDF product counts;
- resident page, brick, clipmap, resident-byte, upload-byte, and budget status
  evidence;
- SDF distance mip count, candidate-list count, total and rejected candidates,
  and max ray-step evidence;
- near, mid, far, and summary visual evidence references with step counts;
- CPU pass timing evidence and explicit unsupported GPU timestamp diagnostics;
- hardware/capability profile identity;
- benchmark command and artifact paths;
- fail-closed diagnostics for missing visual evidence, missing benchmark
  commands, missing timing evidence, missing GPU timing diagnostics, broken
  residency/raymarch count invariants, missing distance mips, and missing
  candidate lists.

Canonical evidence command:

```text
cargo run -p engine --example sdf_render_flow -- --evidence
```

Output summary:

```text
sdf_runtime_evidence flow=sdf_render_flow_3d passes=4 ready=true errors=0 warnings=1
residency selected=4 resident=4 pages=10 bricks=10 clipmaps=4 resident_bytes=1280 upload_bytes=1280
raymarch distance_mips=4 candidate_lists=4 candidates=16 rejected=0 max_steps=160
timing source=cpu_encode_submit cpu_samples=3 cpu_total_ms=0.6500 gpu_samples=0 gpu_total_ms=0.0000 gpu_diagnostics=1
```

Standalone evidence command:

```text
cargo run -p engine --example render_sdf_runtime_evidence -- --evidence
```

Output summary:

```text
render sdf runtime evidence ready=true errors=0 warnings=1 profile=standalone-sdf-runtime
resident=4 pages=10 bricks=10 clipmaps=4 distance_mips=4 candidate_lists=1 visual=4 cpu_ms=0.550 gpu_diagnostics=1
```

## Validation

Focused validation passed:

```text
cargo fmt
cargo test -p engine render_sdf
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
cargo test -p engine --example sdf_render_flow
cargo test -p engine --example render_sdf_runtime_evidence
cargo run -p engine --example sdf_render_flow -- --evidence
cargo run -p engine --example render_sdf_runtime_evidence -- --evidence
cargo bench -p engine --bench render_flow_planning
```

The `render_sdf` filter ran the new
`engine/tests/render_sdf_runtime_evidence.rs` coverage:

- `render_sdf_runtime_evidence_reports_ready_runtime_chain`
- `render_sdf_runtime_evidence_fails_closed_without_visual_and_benchmark_evidence`
- `render_sdf_runtime_evidence_fails_closed_on_raymarch_count_drift`

Benchmark notes from the local Criterion run:

- `render_sdf/runtime_evidence_report_4096`: 134.92 us to 137.52 us.
- Existing local baselines reported regressions for several render-flow
  planning/preflight cases. These are recorded as local Criterion baseline
  changes and not hidden as universal performance claims.
- `render_scale/production_evidence_report_4096` reported no detected
  performance change.

Final planning validation after roadmap and production metadata updates:

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

Completion quality: `runtime_proven`.

Known quality gaps:

- The canonical evidence commands are deterministic, non-windowed closeout
  evidence paths. They prove flow compilation, renderer DTO consumption, visual
  evidence references, timing diagnostics, and benchmark linkage; they do not
  promise universal FPS.
- Timestamp-query evidence is capability-gated. The portable examples record
  unsupported timestamp queries as typed diagnostics; hardware with timestamp
  support can provide measured GPU pass timings in later local evidence runs.
- Final `perfectionist_verified` remains blocked until
  `PT-RENDER-PERFECTION` audits all completed renderer production tracks and
  proves there are no remaining known quality gaps.

These are visible production-track quality limits, not hidden defects in the
WR-066 runtime-proven contract.
