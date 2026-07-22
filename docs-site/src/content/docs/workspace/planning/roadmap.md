---
title: Roadmap
description: Manually maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-22
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
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-render-s0-identity-consumer-lifecycle.md
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

## Completed repository work

### Workflow simplification

Issue `#122` completed through PRs `#123` and `#124`. `cargo validate` and
exact-head CI are the canonical baseline. The obsolete workflow orchestration,
machine state, generated planning, truth-certificate, batch, and quiet/full gate
systems are removed.

### RunenSDF standalone transfer

The in-workspace boundary correction completed through PR `#116`. Standalone
transfer and conformance completed through Runenwerk PR `#118` and
`Crystonix/runen-sdf` PR `#1` at:

```text
repository: Crystonix/runen-sdf
commit: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
package: runen-sdf
crate: runen_sdf
```

Current Runenwerk `main` does not yet record a completed clean cutover removing
`domain/sdf`. That remains a separate consumer-audit and integration/removal
decision.

### GPU/render architecture correction

Issue `#125` completed through PR `#126`. Accepted direction:

```text
RunenRender -> RunenGPU
```

RunenGPU and RunenRender each begin with one public package. WGPU belongs inside
RunenGPU. RunenRender owns image formation and lowers through RunenGPU.

### GPU/render S0 inventory

Issue `#127` and PR `#128` establish the current-source inventory required before
implementation:

```text
primary render/macro files       174
move to RunenGPU                  25
move to RunenRender               10
stay in Runenwerk                 60
stay with another domain          28
redesign before movement          50
delete after replacement           1
```

S0 classifies current identities, direct consumers, shader/macro boundaries,
surface lifecycle ownership, validation commands, and every primary file.
Environment-dependent GPU proof remains intentionally deferred to later phases.

## Current priorities

1. Complete and merge the S0 documentation in PR `#128`.
2. Write exactly one G1A implementation specification against the merged current
   `main` for the logical GPU work-resource identity.
3. Implement G1A only after that specification is reviewed and explicitly
   authorized.
4. Resolve the RunenSDF clean cutover through exact current consumer evidence.
5. Continue RunenECS work only as separately bounded changes that do not conflict
   with GPU/render identities, manifests, or lifecycle ownership.

## RunenGPU and RunenRender sequence

```text
S0 current-source inventory
-> G1-G8 internal RunenGPU proof
-> GX external RunenGPU clean cutover
-> R1-R8 internal RunenRender proof on RunenGPU
-> RX external RunenRender clean cutover
-> A1 reusable adapter review
-> V1+ advanced renderer work
```

### S0 result

S0 is complete for deterministic current-source and direct-consumer evidence. It
identifies one bounded first implementation candidate:

```text
G1A
RenderResourceId -> GpuWorkResourceId
```

`RenderResourceId` is a graph-local logical GPU work-resource identity allocated
per `RenderFlow`. G1A must provide an opaque nonzero identity, fallible deterministic
allocation, explicit exhaustion, structured errors, no arbitrary safe raw
construction, no stable-format claim, and no compatibility alias.

G1A explicitly excludes:

```text
RenderFlowId
RenderPassId
RenderFeatureId
RenderFrameProducerId
RenderSurfaceId
WGPU realization
render graph redesign
surface/window lifecycle
shader/pipeline redesign
external repository creation
RunenRender implementation
```

No G1A implementation is authorized by S0 alone. The next artifact is a precise
implementation specification naming every current consumer and changed file.

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

The accepted repair order remains R1-R9. R1 is specified but not implemented.
Repository extraction remains blocked until the internal boundary repair and
conformance sequence completes.

## RunenUI

RunenUI is governed in its own repository. A future Runenwerk-owned bridge may
translate accepted renderer-neutral paint output into a RunenRender overlay
contribution after both public boundaries stabilize. RunenUI owns UI state, layout,
text, accessibility, and hit testing; RunenRender does not.

## Sequencing rule

Read-only investigation and unrelated cleanup may run in parallel. Structural
changes that share manifests, lockfiles, dependency direction, identities,
lifecycle, or canonical architecture must be serialized or explicitly rebased.

No roadmap item authorizes implementation by itself. An implementation PR requires
a current issue/design, explicit scope, acceptance criteria, and exact-head
validation.
