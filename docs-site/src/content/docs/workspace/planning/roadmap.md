---
title: Roadmap
description: Maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-28
related_docs:
  - ../engineering-workflow.md
  - ./active-work.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-runenrender-application-domain-fit.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../../reports/closeouts/pt-runen-family-operational-hardening-closeout.md
  - ../../reports/closeouts/pt-runengpu-g3-implementation-closeout.md
  - ../specs/pt-runengpu-g3-access-work-graph.ron
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

1. Complete exact-head CI and independent review for issue `#177` / draft PR `#181`.
2. Merge the coherent G3 migration/deletion slice without adding G4/G5/G7 work.
3. Plan and execute G4-G8 sequentially through their existing owners after G3 merge.
4. Extract RunenGPU and perform a clean Runenwerk cutover only after internal
   conformance and extraction-readiness gates pass.
5. Prove RunenRender internally on the accepted external RunenGPU boundary.
6. Extract and cut over RunenRender only after its own R-phase conformance.
7. Resume RunenECS boundary repair as separately bounded non-conflicting work.

Accepted foundation:

```text
RunenSDF cutover complete
RunenGPU S0 complete
RunenGPU G1A complete
RunenGPU G2 complete at 709aa6aced020ee99405e1e1c3dde7703c77a4d4
RunenGPU G3 planning complete at 5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8
Runen family operational hardening complete at 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6
RunenGPU G3 corrected code candidate on branch at 905c506e33202405d1bea8c160a05ac92c326c43
```

The G3 candidate starts from accepted base
`1c645b2bbfcece44dd6ae151cc97559793afa2c2`. Its first reviewed head was
`38abac6bd234d9db3a4544aedbf2dba149538e36`; draft PR `#181` remains open and
unmerged pending a new review of the corrected head, so the roadmap asserts no G3
merge SHA.

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
            -> WGPU backend
```

Non-render consumers may lower directly into RunenGPU work.

### Strategic position

The architecture intentionally sits between direct WGPU and production render graphs:

- broader than a WGPU wrapper because it owns normalized capabilities, logical
  resources, checked work, progress, pressure, lifecycle, recovery facts, and
  diagnostics;
- narrower than Unreal RDG because aliasing, multi-queue scheduling, pass fusion, and
  graph UI remain deferred;
- independent of ECS, windows, product policy, scene semantics, and application
  domains;
- paired with RunenRender only for image formation.

Direct WGPU remains the strongest substitute and mandatory narrow comparison.
Filament, rend3, Bevy, Godot, Unity, Unreal, bgfx, CUDA, and OptiX remain valid choices
for different constraints.

### Public API experience

The graph remains internal/advanced correctness authority. Ordinary callers submit
work through a simple path that validates automatically. Strings remain diagnostic,
not identity or dependency authority. Simple and inspectable paths use the same
preparation authority.

### G3 accepted contract

G3 planning accepted through issue `#174` and PR `#175` binds:

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

Operational hardening does not alter these semantics.

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
- G7 owns device generations, loss classification, and reconstruction reports;
- G8 proves recovery, capture, cache, pressure, performance, and no reach-through;
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
G3 access/work/graph implementation #177 / PR #181         complete on branch; in review
G4 context/device/program/layout/WGPU/cache admission       deferred
G5 progress/pressure/execution/readback/retirement          deferred
G6 offscreen graphics/shared consumers/direct baseline     deferred
G7 surfaces/generations/device loss/reconstruction          deferred
G8 operational conformance/diagnostics/shutdown/audit       deferred
GX external dornglut/runen-gpu transfer and clean cutover  blocked
```

Every implementation phase starts from current `main`, repeats its declaration and
consumer census, migrates every consumer of replaced authority, and deletes that
authority in the same accepted slice.

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

Near-term proof provider pressure:

```text
Procedural
Analytic
field-backed Solid sufficient for terrain
Overlay
```

Research candidates:

```text
Volume
Population
RegionalSummary
Liquid
```

Deferred pending concrete accepted consumers:

```text
Fiber
broad hardware-specialized providers
universal provider unification
```

RunenRender provider interfaces remain narrow. No universal provider trait may force
all query families on every implementation.

## Proof portfolio

### G5 correctness

- exact inclusive/exclusive prefix scan over exactly 4,097 elements with full readback comparison;
- headless fixed-seed 160×90 Game of Life for exactly 16 steps with CPU oracle, exact live count `2,063`, and FNV-1a-64 checksum `0xBD710B88594CD584`;
- deterministic integer compute-to-texture when accepted.

### G5 operations

- submission, upload, and readback saturation;
- native/web progress behavior;
- callbacks outside internal locks;
- lifecycle-point cancellation;
- shutdown with pending work and exactly one terminal outcome.

### G6 graphics and overhead

- offscreen known-pattern draw;
- compute-generated indirect draw;
- offscreen boids integration;
- direct-WGPU comparisons for narrow compute, image processing, and graphics proofs;
- cold/warm pipeline characterization.

### G7 lifecycle

- surface outdated/lost/out-of-memory outcomes;
- device generations and stale-value rejection;
- source-backed reconstruction;
- external reimport requirements;
- explicit non-reconstructable loss.

### G8 conformance

- cache compatibility and rejection;
- recovery and pending-work shutdown;
- reproducibility bundle;
- bounded capture/diagnostics;
- residual WGPU reach-through audit;
- retained standalone/downstream conformance.

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

Every G2-G7 and R-phase slice must:

- start from current `main` and an exact declaration/consumer inventory;
- establish one transferable owner boundary;
- preserve ergonomic and inspectable paths;
- migrate all consumers of replaced authority;
- delete replaced authority without aliases or parallel paths;
- keep product policy in Runenwerk;
- run focused checks plus exact-head `cargo validate` and `git diff --check`;
- update parent issues and durable planning;
- stop if a new ADR, package, compatibility path, persisted-format change, or
  premature later-phase owner is required.

## Strategic reevaluation gates

Reconsider RunenGPU if there is no independent non-render consumer, ordinary callers
require raw WGPU, or measured overhead outweighs reusable correctness value.

Reconsider RunenRender if a smaller existing renderer satisfies accepted proofs,
provider abstractions become universal/runtime-heavy, or prepared scenes require
systematic full rebuilds.

Reevaluation is an explicit architecture decision, not permission to create a hidden
bypass.
