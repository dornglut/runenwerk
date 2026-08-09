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

Issue `#224` is the only authorized RunenGPU documentation correction: it finalizes
G4C1 realization and migration semantics without authorizing source implementation.

Current status is deliberately narrow:

- `#224` — only authorized RunenGPU documentation correction.
- `#212` — census retained; the existing implementation branch is retained but untouched
  and implementation is blocked by `#224`.
- `#213` — blocked.
- `#214` — blocked.

The retained implementation branch is
`codex/runengpu-g4c1-resource-realization`. It remains zero commits and zero changed
files from accepted main `810f3e31174a84dd494c11eea1616092142e11bc`; do not modify or
replace it. Only after #224 is accepted and its squash commit has accepted-main CI and
Documentation Build proof may that still-empty branch move to the new accepted main and
re-establish #212 as the sole active implementation slice.

When later authorized, G4C1 will own private context/device-generation-bound realization
of:

- buffers;
- textures;
- texture views;
- samplers;
- query sets;
- transactional resource registries and discardable in-memory compatibility caches;
- migration and deletion of renderer-owned resource realization that the slice replaces.

It may retain exactly one narrow crate-private `CurrentRenderResourceBridge` for exact
audited current G4C2/G4C3/G5 consumers. That bridge lends purpose-typed,
affinity-validated resource references only; the consumer's semantic operation remains
in its existing phase. G4C2 must replace and delete it with a successor carrying only
proven residual terminals. It does not parse WGSL, create layouts/bind groups or
pipelines, implement G5 execution, own G7 surfaces, or absorb RunenRender semantics.

`CurrentRenderDeviceQueue` is separately a crate-private backend-operation loan, not a
second object-reference bridge and not part of `CurrentRenderResourceBridge`. G4C1
removes generic buffer/texture/view/sampler/query-set creation through it; G4C2 then
removes module/layout/bind-group creation; G4C3 removes pipeline creation; G5 migrates
the remaining operation users and deletes it. Its source-guarded operation classes and
exact call sites only shrink.

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

Each later implementation child requires its own current-main census, one implementation
branch and PR, exact-head validation, independent complete-diff review, accepted merge,
and accepted-main verification. No child consumes an unmerged predecessor branch.

Branch discipline for active RunenGPU implementation is one remote implementation branch
per active PR. Corrections stay on that branch; experiments remain local. Temporary
remote staging refs require a concrete exceptional need and are deleted immediately after
transfer or rejection.

### G4C2 — blocked by accepted G4C1

G4C2 will own canonical WGSL module creation, direct pinned Naga evidence, agreement with
G4B resource declarations, bind-group layouts, pipeline layouts, typed bind groups, and
their private registries/caches. It replaces and deletes `CurrentRenderResourceBridge`
with `CurrentRenderPipelineBridge`, which carries only proven residual G4C1 resource
terminals plus G4C2 program/layout/bind-group terminals for current pipeline creation and
unchanged encoding. It does not acquire G5 execution ownership.

### G4C3 — blocked by accepted G4C2

G4C3 will own compute/render pipeline realization, complete compatibility keys, final
stage-IO agreement, migration of remaining realization consumers, and deletion of
renderer-owned G4 realization/cache authority. It replaces and deletes
`CurrentRenderPipelineBridge` with one narrow `CurrentRenderExecutionBridge` carrying
only accepted resource/bind-group/pipeline terminals for the unchanged G5 encoder; G5
deletes that bridge.

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
