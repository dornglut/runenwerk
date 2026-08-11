---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-11
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
  - ../../design/active/runengpu-g4c2-presentation-surface-binding-boundary.md
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
state, the only authorized next RunenGPU gate, and the immediate dependency sequence.

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
G4C1 private resource realization    accepted at 4dbc6edc46c3a4bf82c91c77e79eff67da44edc9
G4C2 surface-binding correction      accepted at daff5372d4b517ae54f3f20e2ee329738da071bc
```

G4B owns the backend-neutral logical program layer: bounded WGSL admission, typed entry
points and binding declarations, program interfaces, observed-interface agreement
vocabulary, bind-group and pipeline-layout descriptors, specialization, deterministic
compute/render pipeline descriptors, runtime binding compatibility, and capability-
complete fixed binding arrays as optional extensions.

G4C1 owns private context/device-generation-bound realization for buffers, textures,
texture views, samplers, and query sets; private process-local logical resource owner
scopes; transactional bounded authoritative realization records; current renderer
resource-consumer migration; and the one narrow `CurrentRenderResourceBridge` retained
only for audited uncut successor consumers. Renderer identity does not become generic GPU
resource identity, and G5 execution/lifecycle plus G7 surfaces remain separate.

Repository-owned accepted-main proof for G4C1 at exact squash
`4dbc6edc46c3a4bf82c91c77e79eff67da44edc9` is:

```text
CI push/main             31498863341 / #513  success
Documentation push/main  31498862417 / #301  success
```

The presentation-surface correction is accepted at exact squash
`daff5372d4b517ae54f3f20e2ee329738da071bc`, with accepted-main proof:

```text
CI push/main             31529433596 / #520  success
Documentation push/main  31529432560 / #308  success
```

Issues `#212` and `#230` are closed completed.

## Only authorized RunenGPU continuation

A final implementation-readiness review of active G4C2 found four remaining contract
choices that would otherwise force an implementation agent to invent architecture.
Issue `#233` is therefore the only active RunenGPU architecture/delivery gate. G4C2
implementation issue `#213` is temporarily blocked until this correction is accepted and
accepted-main verified.

Current status:

- `#233` — active G4C2 realization decision closure;
- branch — `docs/g4c2-decision-closure` from exact accepted main `daff5372d4b517ae54f3f20e2ee329738da071bc`;
- `#213` — blocked pending accepted `#233`, accepted-main proof, and a fresh exact-current-main activation census;
- `#214` — blocked behind accepted `#213`;
- G5 — unauthorized pending accepted G4C3 and separate design.

The correction does not widen G4C2. It makes four implementation semantics explicit:

1. **Backend completion is asynchronous.** WGPU 27.0.1 validation/OOM/internal error
   scopes must resolve before a shader module, bind-group layout, pipeline layout, or bind
   group is published. RunenGPU owns no executor and hides no `block_on`; any synchronous
   driving policy belongs at a Runenwerk product/integration boundary.
2. **The direct Naga profile is exact.** For Naga 27.0.3 the accepted evidence path uses
   `ValidationFlags::all()` plus `Capabilities::default()`. No broader Naga capability is
   inferred from current coarse RunenGPU feature vocabulary; an extension requires an
   explicit accepted mapping first.
3. **Request identity precedes realized evidence.** Complete typed pre-realization request
   keys locate accepted program/layout/bind-group records. A live exact hit returns the
   accepted record without rerunning Naga or WGPU creation. Observed interface/stage-IO
   and WGPU acceptance facts remain in the realized record, not the lookup key.
4. **G4C2 authoritative growth is bounded explicitly.** One
   `GpuProgramBindingRealizationPolicy` shares a default 16,384-record bound across
   program/module, bind-group-layout, pipeline-layout, and bind-group authority. A unified
   `GpuRealizationPolicies` context override composes it with the G4C1 resource policy;
   only registry-only records may be reclaimed, live authority is never silently evicted,
   and reclamation does not acquire G5 completion/retirement semantics.

The already accepted presentation-surface correction remains unchanged: before G7,
`SurfaceColor` may remain a presentation/render attachment and retain separately owned
copy behavior, but it is not a G4C1 logical resource and cannot enter a G4C2 sampled or
storage bind group.

The bridge ladder remains unchanged:

```text
G4C1  CurrentRenderResourceBridge
    -> G4C2  CurrentRenderPipelineBridge
        -> G4C3  CurrentRenderExecutionBridge
            -> G5 deletes the execution bridge
```

Exactly one object-reference migration bridge remains at an accepted G4C boundary. A
successor deletes its predecessor, carried predecessor terminals only shrink, and no
G4C2 bridge gains a raw presentation-surface shader-binding terminal.

`CurrentRenderDeviceQueue` remains separately a crate-private backend-operation loan,
not a second object-reference bridge. G4C1 removed generic
buffer/texture/view/sampler/query-set creation through it; G4C2 removes
module/layout/bind-group creation; G4C3 removes pipeline creation; G5 migrates remaining
encoding/upload/submission/copy/map/readback operation users and deletes the loan.

## Ordered G4C continuation

Issue `#188` remains a non-implementable umbrella.

```text
#212 G4C1 private resource realization                        accepted
    -> #230 G4C2 presentation-surface boundary correction     accepted
        -> #233 G4C2 realization decision closure             active
            -> #213 G4C2 private program/layout/bind-group realization   blocked
                -> #214 G4C3 private pipeline realization and final cutover   blocked
                    -> separately planned G5 execution and lifecycle
```

No implementation child consumes an unmerged predecessor or correction branch. After
`#233` is accepted and accepted-main verified, `#213` must re-resolve its implementation
base from then-current accepted `main`, repeat its exact-current-main dependency/source
census, and create one implementation branch/PR only from that base.

### G4C2 — blocked by #233 decision closure

G4C2 will own canonical WGSL module creation, direct pinned Naga evidence, agreement with
G4B resource declarations, bind-group layouts, pipeline layouts, typed bind groups, and
their private bounded registries plus optional derived caches. Typed texture bindings
consume accepted G4C1 resource handles; acquired presentation-surface views are not a
pre-G7 shader-resource exception.

The implementation must use the exact accepted Naga profile, asynchronous scoped WGPU
acceptance before publication, pre-realization request keys for authoritative reuse, and
one explicit bounded program/binding realization policy. It must structurally reject
sampled/storage `SurfaceColor` bindings before bind-group realization, replace and delete
`CurrentRenderResourceBridge` with `CurrentRenderPipelineBridge`, migrate G4C2-owned
realization, and leave G7 surface capability/affinity/generation work untouched. It does
not acquire G5 execution ownership or hide executor/blocking policy in RunenGPU.

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
- G7: surfaces, affinity, generations, device/surface loss, reconstruction facts, and
  any explicit presentation-surface usage/capability policy;
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

For every implementation or contract-correction slice:

```text
cargo validate
git diff --check
CI=true pnpm --dir docs-site build
```

Repository-owned exact-head CI and Documentation Build plus independent complete-diff
review are required. A green branch does not become accepted authority until merge and
accepted-main verification.
