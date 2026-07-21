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
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/closeouts/pt-runensdf-003-standalone-transfer-closeout.md
---

# Roadmap

This is the single maintained roadmap for Runenwerk. It records high-level
sequencing and dependencies. GitHub issues and pull requests own live delivery.

## Repository family

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters and product integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

Framework repositories do not depend on Runenwerk. Runenwerk owns application
lifecycle, cross-framework composition, adapters, editor/runtime integration,
product policy, diagnostics presentation, and recovery.

Each framework begins with one public package. Package splitting requires proven
independent dependency or release pressure.

## Completed repository work

### Workflow simplification

Issue `#122` completed through PRs `#123` and `#124`.

Delivered:

- one canonical `cargo validate` baseline;
- permanent exact-head CI;
- one engineering workflow and one maintained roadmap;
- removal of production-track, execution-lock, truth-certificate, batch, generated
  prompt, quiet/full gate, and generated planning systems;
- permanent repository audit preventing their return.

### RunenSDF standalone transfer

Completed:

- in-workspace boundary correction through Runenwerk PR `#116`;
- standalone repository transfer and conformance through Runenwerk PR `#118` and
  `Crystonix/runen-sdf` PR `#1`;
- accepted standalone revision:

```text
repository: Crystonix/runen-sdf
commit: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
package: runen-sdf
crate: runen_sdf
```

Current Runenwerk `main` does not yet record a completed clean cutover removing
`domain/sdf`. That remains a separate consumer-audit and integration/removal
decision.

## Current priorities

1. Complete issue `#125`: correct RunenGPU/RunenRender architecture on current
   `main`, superseding closed PR `#119`.
2. Perform GPU/render S0 inventory before writing any implementation phase.
3. Resolve RunenSDF clean cutover only through exact current consumer evidence.
4. Continue RunenECS R1 only as a separately bounded change that does not conflict
   with GPU/render identities, manifests, or lifecycle ownership.

## RunenGPU and RunenRender

Accepted repository identities:

```text
product       repository                 package       crate
RunenGPU      Crystonix/runen-gpu        runen-gpu     runen_gpu
RunenRender   Crystonix/runen-render     runen-render  runen_render
```

Accepted dependency direction:

```text
RunenRender -> RunenGPU
```

RunenGPU owns validated GPU contexts, capabilities, resources, access/lifetimes,
hazards, workloads, submission, uploads/readback, low-level surfaces, WGPU
realization, backend outcomes, and GPU diagnostics.

RunenRender owns prepared render scenes, views, providers/interactions,
materials/media, emitters, visibility, transport, radiance caches, history,
reconstruction, overlays, color, presentation intent, and lowering into RunenGPU
workloads.

Runenwerk retains application lifecycle, windows/event loops, ECS/domain
extraction, shader source discovery/reload policy, adapters, product quality,
diagnostics presentation, and recovery.

RunenUI and RunenSDF remain independent. Cross-framework bridges are
Runenwerk-owned until independent reuse proves extraction.

### Required sequence

```text
S0 complete current-source inventory
-> G1-G8 internal RunenGPU proof
-> GX external RunenGPU clean cutover
-> R1-R8 internal RunenRender proof on RunenGPU
-> RX external RunenRender clean cutover
-> A1 reusable adapter review
-> V1+ advanced renderer work
```

Only S0 is next. No G1 or R1 implementation specification is active.

### S0 required result

S0 must produce:

- complete file, shader, macro, test, example, benchmark, and artifact inventory;
- complete dependency and downstream consumer graph;
- every identity, allocator, raw use, and stable-format classification;
- graph/resource/frame/producer and context/device/surface/window/shutdown traces;
- shader/pipeline/reload/macro ownership map;
- exact move/stay/redesign/delete disposition;
- validation and environment-dependent GPU command inventory;
- one bounded first implementation candidate and stop conditions.

Unknown ownership blocks implementation.

## RunenSDF next decision

Before a Runenwerk clean cutover:

1. audit every code, test, manifest, adapter, document, and persisted consumer;
2. determine whether Runenwerk has a real external dependency consumer;
3. pin the accepted standalone revision only where needed;
4. remove `domain/sdf`, workspace membership, and stale lockfile authority;
5. prove no forwarding package, alias, source include, branch dependency, or
   duplicate implementation remains;
6. pass exact-head `cargo validate` and focused integration evidence.

If no product consumer exists, remove the internal package without adding an unused
external dependency.

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
R8 neutral scheduling boundary
R9 standalone conformance and performance baseline
```

R1 is specified but not implemented. Repository extraction remains blocked until
the internal boundary repair and conformance sequence is complete.

## RunenUI

RunenUI is governed in its own repository.

A future Runenwerk-owned bridge may translate accepted renderer-neutral paint
output into a RunenRender overlay contribution after both public boundaries
stabilize. RunenUI owns UI state, layout, text, accessibility, and hit testing;
RunenRender does not.

## Sequencing rule

Read-only investigation and unrelated cleanup may run in parallel. Structural
changes that share manifests, lockfiles, dependency direction, identities,
lifecycle, or canonical architecture must be serialized or explicitly rebased.

No roadmap item authorizes implementation by itself. An implementation PR requires
a current issue/design, explicit scope, acceptance criteria, and exact-head
validation.
