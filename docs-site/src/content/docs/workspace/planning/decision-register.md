---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../engineering-workflow.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ./active-work.md
  - ./roadmap.md
  - ./completed-work.md
---

# Decision Register

## Repository-family ownership

Date: 2026-07-21

Decision:

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

Runenwerk remains the product and integration repository. Framework repositories
do not depend on Runenwerk.

ADR 0015 accepts one direct framework dependency:

```text
RunenRender -> RunenGPU
```

Rejected: submodules, source mirrors, universal shared core, unchanged directory
extraction, indefinite compatibility packages, and speculative multi-package
framework skeletons.

## Initial package identity

Date: 2026-07-21

Decision:

```text
product       repository                 package       crate
RunenSDF      Crystonix/runen-sdf        runen-sdf     runen_sdf
RunenECS      Crystonix/runen-ecs        runen-ecs     runen_ecs
RunenGPU      Crystonix/runen-gpu        runen-gpu     runen_gpu
RunenRender   Crystonix/runen-render     runen-render  runen_render
RunenUI       Crystonix/runen-ui         runen-ui      runen_ui
```

Each framework begins with one public package. Additional packages require proven
independent dependency, backend, release, ABI, platform, or compile-time pressure.
Internal module separation is preferred first.

## Workflow simplification completion

Date: 2026-07-21

Decision: accept issue `#122` completion through PRs `#123` and `#124`.

Accepted outcomes:

- `cargo validate` is the canonical local and CI baseline;
- repository tooling validates itself before the product workspace;
- GitHub issues/PRs own live work and one maintained roadmap owns high-level
  sequencing;
- production-track, execution-lock, truth-certificate, batch, generated-prompt,
  quiet/full gate, and generated planning systems are retired;
- the permanent Rust repository audit rejects reintroduction of retired paths and
  command namespaces.

## RunenSDF boundary correction

Date: 2026-07-20

Decision: accept the in-workspace RunenSDF boundary correction through PR `#116`.

Accepted outcomes:

- no Runenwerk geometry dependency in `domain/sdf`;
- validated SDF-owned bounds and rays;
- distinct signed value and conservative tracing step;
- explicit exact-distance capability;
- capability-sensitive queries and structured failures;
- no compatibility layer or external cutover in that phase.

Closeout:
`../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md`.

## RunenSDF standalone transfer

Date: 2026-07-21

Decision: accept standalone repository transfer through Runenwerk PR `#118` and
`Crystonix/runen-sdf` PR `#1`.

Accepted revision:

```text
repository: Crystonix/runen-sdf
commit: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
package: runen-sdf
crate: runen_sdf
```

This decision does not claim that current Runenwerk `main` has completed the later
clean cutover removing `domain/sdf`. That remains a separate current-consumer and
integration/removal decision.

Closeout:
`../../reports/closeouts/pt-runensdf-003-standalone-transfer-closeout.md`.

## RunenECS boundary

Date: 2026-07-19

Decision: do not extract current ECS directories unchanged.

RunenECS owns ECS lifecycle, storage/query semantics, deferred mutation, system
access, explicit reflection, and ECS-local scheduling integration. General spatial
indexing, engine lifecycle, rendering extraction, networking, replay, world policy,
and product scheduling remain outside.

R1 remains the next bounded internal repair. No external source movement is
authorized.

## RunenGPU ownership

Date: 2026-07-21

Decision: create RunenGPU as the lower-level GPU execution framework before
RunenRender extraction.

RunenGPU owns:

- contexts, capabilities, resources, access/lifetimes, hazards, workloads;
- shader/pipeline realization;
- uploads/readback, submission/completion;
- low-level surfaces and backend/device outcomes;
- WGPU as the initial internal backend;
- GPU diagnostics and provenance.

RunenGPU does not own image formation, domain algorithms, ECS, UI, windows/event
loops, shader file watching, or product recovery.

No RunenGPU implementation is authorized before S0 inventory and one current-main
G1 specification.

## RunenRender ownership

Date: 2026-07-21

Decision: RunenRender owns image formation and depends on RunenGPU.

RunenRender owns:

- prepared render scenes and contributions;
- views and logical targets;
- providers/interactions;
- materials, media, emitters, environments;
- visibility, transport, radiance caches, history, reconstruction;
- overlays, color, presentation intent;
- lowering into RunenGPU workloads.

RunenRender does not own WGPU directly, general GPU execution, ECS extraction,
RunenSDF mathematics, RunenUI semantics, native windows, shader file watching, or
Runenwerk lifecycle.

The old `runenrender_core`/`runenrender_wgpu` target and renderer-first identity
phase are superseded.

## GPU/render sequencing

Date: 2026-07-21

Decision:

```text
S0 complete inventory
-> G1-G8 internal RunenGPU proof
-> GX external RunenGPU clean cutover
-> R1-R8 internal RunenRender proof
-> RX external RunenRender clean cutover
-> reusable adapter review
-> advanced renderer work
```

Only S0 is next. Later implementation specifications are written one at a time
from current `main` after prerequisite closeout.

## RunenUI separation

Date: 2026-07-21

Decision: RunenUI remains independently governed.

RunenUI owns semantic UI, state/actions, focus/accessibility, layout/style/text,
hit testing, and renderer-neutral paint output.

A future Runenwerk-owned bridge may translate paint output into a RunenRender
overlay contribution. RunenUI does not depend on RunenRender or RunenGPU by
default.

## Historical supersession

- PR `#107` is a rejected incomplete extraction attempt and is not an
  implementation base.
- PR `#119` is closed unmerged and superseded by issue `#125` because it used stale
  repository state, speculative package splits, and retired planning authorities.
- Historical production-track and generated planning records are evidence only and
  do not authorize work.
