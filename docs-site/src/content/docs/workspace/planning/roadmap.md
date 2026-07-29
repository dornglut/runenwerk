---
title: Roadmap
description: Maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-29
related_docs:
  - ../engineering-workflow.md
  - ./active-work.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ../../reports/investigations/runengpu-g4-context-program-realization-investigation.md
  - ../../reports/closeouts/pt-runengpu-g3-implementation-closeout.md
  - ../specs/pt-runengpu-g4a-context-admission.ron
  - ../specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
---

# Roadmap

This roadmap records high-level sequence and dependencies. GitHub issues and pull
requests own live delivery.

## Repository family

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

Runenwerk remains the integration and product repository. Framework repositories do
not depend on Runenwerk.

## Current priorities

1. Accept the G4 decision phase in issue `#182` / planning PR `#185` only after exact
   feature-head validation, documentation build, and critical review.
2. Activate only G4A context and adapter/device admission from the accepted
   specification.
3. Keep G4B blocked until accepted G4A and G4C blocked until accepted G4B.
4. Continue G5-G8 sequentially only through separately accepted issues and specs.
5. Extract RunenGPU and perform a clean Runenwerk cutover only after internal
   conformance and extraction-readiness gates pass.
6. Prove RunenRender internally on the accepted external RunenGPU boundary through its
   separately owned R phases.
7. Extract and cut over RunenRender only after its own conformance.
8. Resume RunenECS boundary repair as separately bounded non-conflicting work.

Accepted foundation:

```text
RunenSDF cutover complete
RunenGPU S0 complete
RunenGPU G1A complete
RunenGPU G2 complete at 709aa6aced020ee99405e1e1c3dde7703c77a4d4
RunenGPU G3 planning complete at 5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8
Runen family operational hardening complete at 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6
RunenGPU G3 implementation accepted at 39d6fe65a334502bdfba0b1a2ce3b365099fcf28
verified-head maintenance accepted at 6bbd341691a34763ef54c8ca059940cac8981265
```

The accepted commit after G3 changes only validation/workflow authority and does not
change RunenGPU or render architecture, source, dependency, manifest, or lockfile.

## RunenSDF

Maintained standalone authority:

```text
repository: dornglut/runen-sdf
package: runen-sdf
crate: runen_sdf
```

Issue `#133` completed the retirement-only Runenwerk cutover after proving zero real
consumers of the internal package. Runenwerk now contains no `domain/sdf` package,
source mirror, forwarding namespace, submodule, source include, or unused external
dependency.

Reusable field mathematics and CPU reference queries belong to `dornglut/runen-sdf`.
Runenwerk world/product integration remains in Runenwerk-owned domains and adapters.

## RunenGPU and RunenRender

Accepted dependency:

```text
RunenRender -> RunenGPU
```

Target layering:

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image plan
        -> RunenGPU generic resource/work contracts
            -> private WGPU backend
```

Non-render consumers may lower directly into RunenGPU work.

### Strategic position

The architecture intentionally sits between direct WGPU and production render graphs:

- broader than a WGPU wrapper because it owns normalized capabilities, logical
  resources, checked work, progress, pressure, lifecycle, recovery facts, and
  diagnostics;
- narrower than a production renderer because image formation, materials, views,
  visibility, lighting, reconstruction and presentation meaning remain RunenRender;
- independent of ECS, windows, product policy, scene semantics, and application
  domains;
- paired with direct WGPU comparison so reusable correctness value remains measurable.

No second backend, universal shader IR, macro package, shared core, or facade package is
created without concrete evidence.

### Accepted G3 contract

G3 planning and implementation now own:

```text
checked buffer byte ranges
checked texture mip/layer/aspect ranges
checked query ranges
QueryResolve destination usage
texture-view normalization
render attachment load/clear/store/discard
multisample resolve targets inside render operations
buffer-zero standalone clear
query-set-to-buffer standalone resolve
region-aware graph-entry initialization
RAW/WAR/WAW hazards
operation-derived capability requirements
immutable work fragments and nodes
typed import/export causality
deterministic prepared graph
explicit non-data order only
```

The accepted G3 merge is
`39d6fe65a334502bdfba0b1a2ce3b365099fcf28`. The temporary adapter and execution
sidecar are migration seams, not parallel G3 authority.

### G4 accepted decomposition

G4 is a three-slice program and must not be collapsed:

```text
G4A context and adapter/device admission
    async headless GpuContext
    context and device-generation identity
    normalized backend/portability/features/limits/formats/alignments
    deterministic requirement admission and degradation
    private instance/adapter/device/queue containment
    temporary host compatibility without G7 surface ownership

G4B program, interface, binding and pipeline contracts
    WGSL-first source keys and revisions
    program and typed entry-point descriptors
    typed binding keys and declarations
    explicit interfaces and layout descriptors
    specialization schemas and values
    deterministic compute/render pipeline descriptors
    compile-pass and compile-fail proof

G4C WGPU realization, cache compatibility and cutover
    generation-bound resources/programs/layouts/bind groups/pipelines
    private WGPU registries
    correctness-complete in-memory cache keys and rejection
    complete consumer migration
    deletion of renderer-owned realization/cache authority
    deletion of synthetic handles and temporary resource-owner bridge
    removal of G4-owned truth from the execution sidecar
```

Only G4A may become active after planning acceptance. G4B is blocked by accepted G4A;
G4C is blocked by accepted G4B.

G4 is GPU/backend decontamination and substrate work. It does not execute work, own
progress/completion/readback/retirement, own reusable surfaces/presentation/loss, or
implement RunenRender semantics.

### Operational requirements

The family requires:

- accepted work is never silently discarded;
- queues, staging, readback, history, and captures expose bounds and structured
  pressure;
- derived caches are non-authoritative, source-generation-bound, validated, and
  versioned when persisted;
- Runenwerk owns the tested compatibility manifest;
- frameworks report lifecycle/reconstruction facts while Runenwerk owns product
  recovery;
- Runenwerk may create a versioned namespaced reproducibility bundle;
- RunenGPU G5 binds native/web progress, callback/reentrancy, cancellation, terminal
  outcomes, and pending-work shutdown;
- G6 includes narrow direct-WGPU comparisons;
- G7 owns device generations after replacement, loss classification, surfaces and
  reconstruction reports;
- G8 proves recovery, capture, cache, pressure, performance, shutdown and no
  reach-through;
- RunenRender R1/R2 proves incremental prepared scenes;
- R3 accepts narrow provider capabilities only;
- R6 binds cache/history invalidation;
- R8 characterizes renderer cost and reproducibility.

### RunenGPU sequence

```text
G1A owner-scoped logical resource identity                 complete
G2 capabilities/resources/typed handles/prepared data      complete
G3 decision-complete planning                              complete
operational hardening #176 / PR #178                       complete at 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6
G3 access/work/graph implementation #177 / PR #181         complete at 39d6fe65a334502bdfba0b1a2ce3b365099fcf28
G4 decision phase #182 / PR #185                            planning acceptance
G4A context and adapter/device admission                    next; only active implementation slice after planning
G4B program/interface/binding/layout contracts              blocked by accepted G4A
G4C WGPU realization/cache/cutover                          blocked by accepted G4B
G5 progress/pressure/execution/readback/retirement          deferred
G6 offscreen graphics/shared consumers/direct baseline     deferred
G7 surfaces/generations/device loss/reconstruction          deferred
G8 operational conformance/diagnostics/shutdown/audit       deferred
GX external dornglut/runen-gpu transfer and clean cutover  blocked
```

Every implementation phase starts from exact current `main`, repeats its declaration
and consumer census, migrates every consumer of replaced authority, and deletes that
authority in the same accepted slice.

### G4C versus G5 sidecar split

After G4B, the temporary sidecar contains no program, source, entry-point, interface,
binding, layout, specialization, or pipeline descriptor truth. After G4C, it contains
no backend resource, shader module, layout, bind-group, pipeline, cache, generation, or
realization truth. Only G5-owned execution payload may remain. G5 deletes that residual
payload during encoding/submission cutover.

### RunenRender sequence

```text
S0 ownership inventory                                     complete
architecture/decomposition                                 accepted direction
R1/R2 prepared scene and contribution lifecycle            blocked on RunenGPU cutover
R3 provider capability proof                               deferred
R4/R5 image formation and quality ladder                   deferred
R6 cache/history invalidation                              deferred
R7 surfaces/integration through RunenGPU                    deferred
R8 operational/performance/capture conformance             deferred
RX external dornglut/runen-render transfer                 blocked
```

The current render tree is migration evidence. G4 changes it only where GPU context,
program/interface, realization or cache authority is removed. It is not renamed, moved,
wrapped or extracted wholesale. Separate RunenRender planning and R-phase issues remain
independently owned. RX is a later mechanical transfer/cutover, not the point where
renderer architecture is invented.

Near-term proof provider pressure:

```text
Procedural
Analytic
field-backed Solid sufficient for terrain
Overlay
```

Research candidates are `Volume`, `Population`, `RegionalSummary`, and `Liquid`.
`Fiber`, broad hardware-specialized providers, and universal provider unification
remain deferred. RunenRender provider interfaces remain narrow.

## Proof portfolio

### G4 deterministic

- context identity, generation affinity and synthetic admission/degradation;
- normalized backend/limit/format/alignment mapping;
- source key/revision/content consistency;
- typed binding/interface/layout compatibility;
- specialization and pipeline descriptor equality/hashing;
- cache-key completeness, stale/foreign rejection and fallback;
- compile-pass/fail, source/dependency/reach-through, migration and deletion guards.

### G4 environment-dependent

- headless WGPU adapter/device request;
- WGSL module and one compute/render pipeline realization;
- resource, view, sampler, query, layout and bind-group realization;
- actual format/alignment and cache behavior;
- explicit unsupported environment outcomes.

### G5 correctness and operations

- exact inclusive/exclusive prefix scan over 4,097 elements with full readback;
- fixed-seed 160x90 Game of Life for 16 steps with CPU oracle, live count `2,063`, and
  FNV-1a-64 `0xBD710B88594CD584`;
- deterministic compute-to-texture when admitted;
- submission/upload/readback saturation, native/web progress, callbacks outside locks,
  lifecycle-point cancellation, and pending-work shutdown.

### G6 graphics and overhead

- offscreen known-pattern draw;
- compute-generated indirect draw;
- offscreen boids integration;
- direct-WGPU comparisons for narrow compute, image processing, and graphics;
- cold/warm pipeline characterization.

### G7 and G8

- surface/device outcomes, generations and stale-value rejection;
- source-backed, external-reimport and non-reconstructable recovery matrix;
- cache compatibility, pending-work shutdown, reproducibility facts, bounded
  capture/diagnostics, residual WGPU reach-through audit, and standalone conformance.

### RunenRender

- procedural sky/SDF terrain;
- incremental technical-digital-twin style scene updates;
- synthetic integer volume;
- cache/history invalidation;
- full versus incremental preparation cost;
- comparison with a simpler renderer/direct path.

## Application-domain pressure

Highest-value domains for architecture proof are:

1. implicit CAD/computational fabrication;
2. scientific/medical volume visualization;
3. robotics synthetic sensors;
4. geospatial/environmental simulation;
5. technical digital twins;
6. VFX/offline procedural generation.

Domain systems remain outside the frameworks. Reports justify proof workloads, not
vertical-product ownership.

## Phase closure rules

Every G4A-G7 and R-phase slice must:

- start from exact current `main` and an exact declaration/consumer inventory;
- establish one transferable owner boundary;
- preserve ergonomic and inspectable paths;
- migrate all consumers of replaced authority;
- delete replaced authority without aliases or parallel paths;
- keep product policy in Runenwerk;
- run focused checks plus exact-head `cargo validate`, `git diff --check`, documentation
  build and repository-owned Actions;
- update parent issues and durable planning;
- stop if a new ADR, package, dependency, compatibility path, persisted-format change,
  or premature later-phase owner is required.

## Strategic reevaluation gates

Reconsider RunenGPU if there is no independent non-render consumer, ordinary callers
require raw WGPU, or measured overhead outweighs reusable correctness value.

Reconsider RunenRender if a smaller existing renderer satisfies accepted proofs,
provider abstractions become universal/runtime-heavy, or prepared scenes require
systematic full rebuilds.

Reevaluation is an explicit architecture decision, not permission to create a hidden
bypass.