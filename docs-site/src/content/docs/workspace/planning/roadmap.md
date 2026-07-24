---
title: Roadmap
description: Maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-24
related_docs:
  - ../engineering-workflow.md
  - ./active-work.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-render-s0-identity-consumer-lifecycle.md
---

# Roadmap

This roadmap records high-level sequence and dependencies. GitHub issues and pull requests own live delivery.

## Repository family

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

Runenwerk remains the integration and product repository. Framework repositories do not depend on Runenwerk.

## Current priorities

1. Implement corrected owner-scoped RunenGPU G1A under issue `#131` from accepted post-RunenSDF `main`.
2. Continue internal RunenGPU boundary proof through one decision-complete implementation slice at a time.
3. Extract RunenGPU and perform a clean Runenwerk cutover only after G2-G8 and conformance gates pass.
4. Prove RunenRender internally on RunenGPU, then extract and cut over RunenRender.
5. Resume RunenECS boundary repair as separately bounded work.

The RunenSDF cutover gate is complete. GPU implementation now owns the serialized active queue; read-only investigation may still proceed independently.

## RunenSDF

The maintained standalone framework is:

```text
repository: dornglut/runen-sdf
maintained revision: ffa970f3eb7fd9ebaa1cfc67665e3e3128cd0676
source-transfer revision: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
package: runen-sdf
crate: runen_sdf
```

Issue `#133` completed the retirement-only cutover after an exact-head census proved zero real consumers of the internal package. Runenwerk now contains no `domain/sdf` package, workspace member, lockfile package, local framework documentation mirror, compatibility namespace, submodule, source include, or unused external dependency.

Current authority is split deliberately:

- reusable signed-field mathematics and CPU reference queries: `dornglut/runen-sdf`;
- Runenwerk world/product integration and payloads: `domain/world_sdf` and Runenwerk-owned adapters.

The repository audit rejects return of the retired internal package and duplicate authority. Detailed evidence is recorded in the PT-RUNENSDF-004 closeout report.

## RunenGPU and RunenRender

Accepted dependency:

```text
RunenRender -> RunenGPU
```

Completed prerequisites:

- architecture correction through PR `#126`;
- deterministic S0 inventory through PR `#128`;
- original G1A implementation specification through PR `#130`;
- cross-flow identity review and corrected owner-scoped G1A specification.

The corrected G1A target is:

```text
RenderResourceId
    -> GpuWorkResourceId { private owner scope, nonzero local value }

RenderResourceIdSequence
    -> owner-controlled GpuWorkResourceIdAllocator
```

The owner scope is required because public resource handles can cross flow boundaries and independent flows currently allocate identical local sequence values. G1A must make foreign-flow handles structurally rejectable rather than allowing an accidental collision with an unrelated local resource.

G1A remains a behavior-preserving internal ownership correction. It does not create an external package, GpuPlugin, public graph identity, WGPU redesign, graph-semantic redesign, surface change, shader/pipeline redesign, or renderer behavior change.

The extraction sequence remains:

```text
G1A owner-scoped logical GPU work-resource identity
-> G2 capabilities and resource descriptors
-> G3 access, lifetime, hazard validation, and work fragments
-> G4 shader/pipeline admission and WGPU realization
-> G5 headless compute, uploads, and readback
-> G6 offscreen graphics and shared consumer proof
-> G7 surfaces, generations, and device outcomes
-> G8 diagnostics, shutdown, and conformance
-> GX external runen-gpu transfer and clean Runenwerk cutover
-> internal RunenRender proof on RunenGPU
-> external runen-render transfer and cutover
```

G1A is future-transferable but does not by itself make RunenGPU extraction-ready.

## RunenECS

RunenECS remains a separate workstream. Continue only through bounded internal boundary-repair changes that do not conflict with active GPU/render, SDF cutover, manifest, identity, or lifecycle work.

## RunenUI

RunenUI is governed in `dornglut/runen-ui`. Runenwerk eventually owns only the integration adapter between accepted renderer-neutral UI output and RunenRender contributions after both public boundaries stabilize.

## Sequencing rule

Structural changes sharing manifests, lockfiles, identities, dependency direction, lifecycle ownership, repository guards, or canonical planning authority are serialized or explicitly rebased. A completed planning or architecture change proves only its own scope and never implies external extraction readiness.
