---
title: Roadmap
description: Maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-22
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

1. Complete issue `#135`: prune duplicate repository workflow and documentation surfaces.
2. Implement RunenGPU G1A under issue `#131` on a fresh branch from current `main`.
3. Complete the RunenSDF clean-cutover consumer audit and integration/removal decision.
4. Continue internal RunenGPU boundary proof through coherent implementation slices.
5. Extract RunenGPU and perform a clean Runenwerk cutover.
6. Prove RunenRender internally on RunenGPU, then extract and cut over RunenRender.
7. Resume RunenECS boundary repair as separately bounded work.

## RunenGPU and RunenRender

Accepted dependency:

```text
RunenRender -> RunenGPU
```

Completed prerequisites:

- architecture correction through PR `#126`;
- deterministic S0 inventory through PR `#128`;
- G1A implementation specification through PR `#130`.

The sequence is:

```text
G1A logical GPU work-resource identity
-> further internal RunenGPU contracts and execution ownership
-> internal conformance
-> external runen-gpu transfer
-> Runenwerk dependency cutover and internal GPU-source deletion
-> internal RunenRender contracts lowering through RunenGPU
-> external runen-render transfer
-> Runenwerk dependency cutover and internal renderer-source deletion
-> reusable adapter review
-> advanced renderer work
```

G1A is a behavior-preserving ownership correction:

```text
RenderResourceId         -> GpuWorkResourceId
RenderResourceIdSequence -> GpuWorkResourceIdAllocator
```

It does not redesign the graph, WGPU execution, surfaces, shaders, renderer behavior, or product behavior.

## RunenSDF

Standalone RunenSDF transfer and conformance are complete at:

```text
repository: Crystonix/runen-sdf
commit: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
package: runen-sdf
crate: runen_sdf
```

Runenwerk still requires a clean-cutover decision:

1. audit every current code, test, manifest, adapter, document, and persisted consumer;
2. add the exact external dependency only where a real consumer exists;
3. migrate consumers;
4. remove `domain/sdf`, workspace membership, and stale lockfile authority;
5. prove no alias, forwarding package, source include, branch dependency, or duplicate implementation remains;
6. pass exact-head validation and focused integration evidence.

If no product consumer exists, remove the internal package without adding an unused dependency.

## RunenECS

RunenECS remains a separate workstream. Continue only through bounded internal boundary-repair changes that do not conflict with active GPU/render, SDF cutover, manifest, identity, or lifecycle work.

## RunenUI

RunenUI is governed in its own repository. Runenwerk eventually owns only the integration adapter between accepted renderer-neutral UI output and RunenRender contributions after both public boundaries stabilize.

## Sequencing rule

Read-only investigation and unrelated cleanup may proceed in parallel. Structural changes sharing manifests, lockfiles, identities, dependency direction, lifecycle ownership, or canonical architecture must be serialized or explicitly rebased.
