---
title: Roadmap
description: Manually maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../engineering-workflow.md
  - ./active-work.md
  - ./completed-work.md
  - ./decision-register.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/closeouts/pt-runensdf-003-standalone-transfer-closeout.md
---

# Roadmap

This is the single maintained roadmap for Runenwerk. It records high-level
sequencing and dependencies; GitHub issues and pull requests own active delivery.

## Repository family

```text
RunenSDF -----+
RunenECS -----+--> Runenwerk adapters and product integration --> applications
RunenGPU -----+
RunenRender --+
RunenUI ------+    independent peer framework
```

Framework repositories do not depend on Runenwerk. Runenwerk owns application
lifecycle, cross-framework composition, adapters, editor/runtime integration,
product policy, and recovery.

## Current priorities

1. Finish workflow-orchestration retirement and repository cleanup under issue
   `#122` without disturbing framework extraction work.
2. Reconcile the GPU/render architecture branch with current `main`, the accepted
   SDF state, and the one-package-per-repository policy before implementation.
3. Perform any RunenSDF clean cutover only through an exact consumer audit and a
   separately reviewed change.
4. Start RunenECS or RunenGPU implementation only after their current-state
   inventories and bounded first contracts are accepted.

## RunenSDF

### Completed

- Boundary correction completed in Runenwerk through PR `#116`.
- Standalone repository transfer completed through Runenwerk PR `#118` and
  `Crystonix/runen-sdf` PR `#1`.
- Accepted standalone revision:

```text
repository: Crystonix/runen-sdf
commit: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
package: runen-sdf
crate: runen_sdf
```

### Next decision

`PT-RUNENSDF-004` is not recorded as completed on current Runenwerk `main`.
Before a clean cutover:

1. audit every code, test, manifest, adapter, documentation, and persisted consumer;
2. add the exact standalone revision only where a real consumer exists;
3. remove `domain/sdf`, its workspace membership, and stale lockfile authority;
4. prove no forwarding package, alias, source include, branch dependency, or
   duplicate implementation remains;
5. pass the complete Runenwerk baseline and integration evidence.

If no production consumer exists, remove the internal package without adding an
unused external dependency.

## RunenGPU and RunenRender

The accepted dependency direction is:

```text
RunenRender -> RunenGPU
```

RunenGPU owns general GPU execution: device/context, capabilities, resources,
access and lifetime validation, workloads, synchronization, uploads/readback,
low-level surfaces, and backend outcomes.

RunenRender owns image formation: prepared scenes, providers/interactions,
materials/media, emitters, visibility, transport, radiance caches,
reconstruction, overlays, color, and presentation intent.

Runenwerk retains windows, application lifecycle, ECS/domain extraction,
adapters, product policy, diagnostics presentation, shader discovery/reload
policy, and recovery. RunenUI and RunenRender remain independent peers.

The existing draft architecture PR `#119` predates the SDF closeout and workflow
cleanup. It must be rebased and critically reconciled before review readiness.
No GPU or renderer implementation is authorized by this roadmap alone.

## RunenECS

The accepted repair order remains:

```text
R1 entity identity and structured errors
R2 atomic structural mutation
R3 query and SystemParam unsafe boundaries
R4 explicit reflection and macro migration
R5 remove spatial and geometry ownership
R6 messaging split
R7 change, ownership, and networking separation
R8 neutralize runen_schedule
R9 standalone conformance and performance baseline
```

R1 is specified but not implemented. Repository extraction remains blocked until
the internal boundary repair and conformance sequence is complete.

## RunenUI

RunenUI is governed in its own repository. A future Runenwerk-owned adapter may
translate accepted renderer-neutral UI paint output into RunenRender overlay
contributions after both public boundaries stabilize. Neither framework depends
on the other.

## Sequencing rule

Read-only investigation and unrelated cleanup may run in parallel. Structural
changes that share workspace manifests, dependency boundaries, identity policy,
or canonical roadmap files must be serialized or explicitly rebased.
