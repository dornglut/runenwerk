---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workflow-lifecycle.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../specs/pt-runengpu-g1-identities-errors.ron
  - ./active-work.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
---

# Decision Register

## Repository-family ownership

Date: 2026-07-19; amended 2026-07-21

Decision:

```text
RunenSDF --------------------+
RunenECS --------------------+
RunenUI ---------------------+--> Runenwerk integration --> applications
                             |
RunenRender --> RunenGPU ----+
other GPU consumers -> RunenGPU
```

Runenwerk remains the product/integration repository. Framework repositories do
not depend on Runenwerk. Rejected: submodules, source mirrors, universal shared
core, unchanged directory extraction, and indefinite compatibility packages.

Direct framework dependencies still require accepted evidence. The accepted
exception is `RunenRender -> RunenGPU` because RunenGPU owns an independently
useful lower-level execution responsibility shared with non-render compute.

## Parallel maturity model

Date: 2026-07-19; clarified 2026-07-21

Decision:

- RunenSDF continues its independently authorized transfer track.
- RunenECS performs ordered internal repairs after activation.
- RunenGPU is decomposed and proven before RunenRender structural extraction.
- RunenRender read-only design may overlap, but GPU realization uses the accepted
  RunenGPU boundary rather than a second temporary backend.
- RunenUI is managed separately.
- Shared manifests, lockfiles, root architecture, and canonical planning merges
  have one active owner at a time.

## PT-RUNENSDF-002 completion

Date: 2026-07-20

Decision: accept the in-workspace RunenSDF boundary correction through PR #116.

Accepted outcomes:

- no Runenwerk `geometry` dependency in `domain/sdf`;
- validated SDF-owned bounds and ray values;
- distinct signed value and conservative tracing step;
- explicit exact-distance capability;
- capability-sensitive metric and tracing queries;
- validated authored state and structured failures;
- explicit query terminals and fallible normals;
- all nine package tests migrated;
- no compatibility layer, source mirror, external repository, or deletion.

Not claimed: MSRV, benchmarks, dedicated property tooling, full workspace tests,
full workspace Clippy, runtime, or GPU evidence.

Closeout:
`../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md`.

## RunenSDF standalone transfer

Date: 2026-07-20

Decision: standalone repository creation, corrected source transfer, and
independent conformance remain owned by the separate SDF track. Runenwerk cutover
and deletion are later SDF phases.

The GPU/render architecture branch must not change SDF implementation or silently
replace newer SDF planning facts. Shared planning changes require rebase and
reconciliation before merge.

## RunenECS boundary

Date: 2026-07-19

Decision: do not extract current ECS directories unchanged. Candidate packages are
`runenecs`, `runenecs_macros`, and independently usable `runen_schedule`. ECS core
does not own Runenwerk geometry/spatial policy. Reflection is explicit. Runenwerk
owns lifecycle and network/replay/product policy. Only R1 is specified; no ECS
Rust implementation is authorized.

## Separate GPU execution from rendering

Date: 2026-07-21

Decision: create a dedicated RunenGPU framework and retain RunenRender as the
image-formation framework.

Required candidates:

```text
RunenGPU
  runengpu_core
  runengpu_wgpu

RunenRender
  runenrender_core
  runenrender_gpu
```

Dependency:

```text
RunenRender -> RunenGPU
```

RunenGPU owns devices, queues, capabilities, general resources, access/lifetimes,
GPU work, WGPU realization, headless execution, uploads/readback, low-level
surfaces, and backend outcomes.

RunenRender owns prepared render scenes, views/targets, providers/interactions,
materials/media, emitters, visibility, light transport, radiance caches,
reconstruction, overlays, output color intent, and render diagnostics.

Runenwerk retains lifecycle, windows, ECS/domain extraction, adapters, shader
source discovery/reload policy, product selection, diagnostics presentation, and
recovery.

Rejected:

- keep general compute in RunenRender;
- rename the entire renderer to RunenGPU;
- duplicate GPU contexts per domain;
- create `runenrender_wgpu` after this split;
- make RunenUI/RunenSDF cores depend on RunenGPU;
- copy the current render directory externally before internal proof.

Authority: ADR 0015.

## Old RunenRender R1 supersession

Date: 2026-07-21

Decision: retire the unimplemented `PT-RUNENRENDER-R1 — Neutral Renderer
Identities and Structured Identity Errors` specification.

Reason: current `Render*Id` values mix probable GPU-execution, renderer-semantic,
and Runenwerk producer/product identities. Implementing a renderer-local identity
rewrite before ownership classification would encode the wrong package boundary.

The retirement is not completion. No Rust implementation was delivered.

Replacement sequence:

```text
S0 complete ownership/command inventory
G1-G9 internal RunenGPU decomposition and proof
GX external RunenGPU transfer and cutover
R1-R8 internal RunenRender decomposition and proof
RX external RunenRender transfer and cutover
```

## PT-RUNENGPU-G1 planning

Date: 2026-07-21

Decision: record a planning-only G1 handoff for GPU-owned identities, structured
errors, and dependency guards.

No implementation is authorized until:

- S0 local file/consumer/identity/command inventory passes;
- every current `Render*Id` is classified;
- raw ID persistence/replay/network/cache use is audited;
- the spec is updated with exact current files and consumers;
- a separate activation decision is accepted.

G1 must not absorb resource/graph/shader/WGPU/surface/renderer behavior or
external source movement.

## RunenUI separation

Date: 2026-07-19; clarified 2026-07-21

Decision: RunenUI implementation remains outside this program. RunenUI core and
runtime do not depend on RunenGPU or RunenRender.

A future Runenwerk-owned adapter may translate accepted renderer-neutral paint
output into RunenRender overlay contributions after both sides expose stable public
seams. An optional standalone UI backend may use RunenGPU externally without
changing RunenUI core ownership.

## Historical supersession

PR #107 is closed unmerged. Commit
`b5e9624c594c9f1e3f2a0929bf84028f13fde860` is a rejected incomplete extraction
attempt and is not an implementation base.

The former renderer-only R1-R10 execution plan is superseded by the
RunenGPU/RunenRender sequence. Historical PR #112 remains evidence of the earlier
investigation and the fact that no implementation was authorized.
