---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-09
related_docs:
  - ../engineering-workflow.md
  - ./roadmap.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runengpu-shader-authoring-artifact-boundary.md
  - ../../design/active/runengpu-g4b-contracts-g4c-delivery-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/2026-08-03-runengpu-g4b-g4c-finalization.md
  - ../specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
  - ../specs/pt-runengpu-g4c1-resource-realization.ron
  - ../specs/pt-runengpu-g4c2-program-binding-realization.ron
  - ../specs/pt-runengpu-g4c3-pipeline-cutover.ron
---

# Active Work

GitHub issues and pull requests own live delivery. This page records durable accepted
state, the only authorized next RunenGPU slice, and the immediate dependency gates.

## Accepted RunenGPU foundation

```text
S0 inventory                         complete
G1A logical work-resource identity   complete
G2 capabilities and resources        accepted at 709aa6aced020ee99405e1e1c3dde7703c77a4d4
G3 checked access and work graph     accepted at 39d6fe65a334502bdfba0b1a2ce3b365099fcf28
verified-head maintenance            accepted at 6bbd341691a34763ef54c8ca059940cac8981265
G4 planning                          accepted at 62c3949d31a7c03f1f554f8108120d9767139123
G4A context admission                accepted at 501b9fd58e56d33708573e47faf0e5026b5a1ff2
shader authoring boundary            accepted at 23bc982703f93d15ac39dd71d61bae9e23854141
G4B program/interface contracts      accepted at 2095afd624979a9f386254d44e082b7eeb0a18a1
```

G4B owns the backend-neutral logical program layer: bounded WGSL admission, typed entry
points and binding declarations, program interfaces, observed-interface agreement
vocabulary, bind-group and pipeline-layout descriptors, specialization, deterministic
compute/render pipeline descriptors, runtime binding compatibility, and capability-
complete fixed binding arrays as optional extensions.

Repository-owned accepted-main validation for G4B succeeded at exact commit
`2095afd624979a9f386254d44e082b7eeb0a18a1` through CI run `31320663399` and
Documentation Build `31320663084`.

## Only authorized RunenGPU continuation

Issue `#212` owns G4C1 private resource realization.

Its current state is an **active start gate**, not source implementation. Before creating
an implementation branch or modifying Rust, repeat the exact-current-main ownership and
consumer census required by #212 and record the result on the issue.

G4C1 owns private context/device-generation-bound realization of:

- buffers;
- textures;
- texture views;
- samplers;
- query sets;
- transactional resource registries and discardable in-memory compatibility caches;
- migration and deletion of renderer-owned resource realization that the slice replaces.

It may retain exactly one narrow crate-private `CurrentRenderResourceBridge` for audited
current consumers until G4C2. It does not parse WGSL, create layouts/bind groups or
pipelines, implement G5 execution, own G7 surfaces, or absorb RunenRender semantics.

The start census must explicitly revisit current texture-preview/resource-preparation
shape where material-prepared data carries shader-binding coordinates that are irrelevant
to generic texture residency. Any cleanup must remain resource-owned G4C1 work and must
not pull G4C2 shader-interface realization forward.

## Ordered G4C continuation

Issue `#188` remains a non-implementable umbrella.

```text
#212 G4C1 private resource realization
    -> #213 G4C2 private program/layout/bind-group realization
        -> #214 G4C3 private pipeline realization and final cutover
            -> separately planned G5 execution and lifecycle
```

Each child requires its own current-main census, one implementation branch and PR,
exact-head validation, independent complete-diff review, accepted merge, and accepted-main
verification. No child consumes an unmerged predecessor branch.

Branch discipline for active RunenGPU implementation is one remote implementation branch
per active PR. Corrections stay on that branch; experiments remain local. Temporary
remote staging refs require a concrete exceptional need and are deleted immediately after
transfer or rejection.

### G4C2 — blocked by accepted G4C1

G4C2 owns canonical WGSL module creation, direct pinned Naga evidence, agreement with G4B
resource declarations, bind-group layouts, pipeline layouts, typed bind groups, and their
private registries/caches. It deletes `CurrentRenderResourceBridge` and may retain only
the narrow `CurrentRenderPipelineBridge` required for G4C3.

### G4C3 — blocked by accepted G4C2

G4C3 owns compute/render pipeline realization, complete compatibility keys, final
stage-IO agreement, migration of remaining realization consumers, and deletion of
renderer-owned G4 realization/cache authority. It may retain only one narrow
`CurrentRenderExecutionBridge` for G5.

## Later RunenGPU program

The remaining program stays sequential and separately authorized:

- G5: work encoding, uploads, queue submission, progress, pressure, completion,
  asynchronous readback, cancellation, delayed retirement, and pending-work shutdown;
- G6: representative offscreen compute/render proof, shared render/non-render consumers,
  direct-WGPU comparisons, and cold/warm cost characterization;
- G7: surfaces, affinity, device replacement, loss, and reconstruction facts;
- G8: operational conformance, reproducibility facts, diagnostics, shutdown, cache and
  residual reach-through audit;
- GX: clean transfer to `dornglut/runen-gpu` only after accepted G2-G8 evidence.

G5, G7, RunenRender implementation, and package extraction remain unauthorized.

## RunenRender boundary

RunenRender remains architecture/design only until accepted external RunenGPU cutover
and a separately authorized R-phase issue.

Its permanent semantic spine is:

```text
RenderSceneStore
    -> RenderSceneCommit(RenderSceneSnapshot + RenderChangeSet)

RenderSceneSnapshot + RenderRequest + RenderInputSet
    -> RenderMethod
        -> RenderPlan
            -> AdmittedRenderPlan
                -> RenderWorkSet
                    -> RunenGPU
```

G4 removes GPU/backend realization authority from the current render tree. It does not
implement image formation or extract RunenRender.

## Acceptance discipline

For every implementation slice:

```text
cargo validate
git diff --check
CI=true pnpm --dir docs-site build
```

Repository-owned exact-head CI and Documentation Build plus independent complete-diff
review are required. A green branch does not become accepted authority until merge and
accepted-main verification.
