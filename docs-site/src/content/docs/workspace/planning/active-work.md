---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-12
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
state, the only next RunenGPU implementation gate, and the immediate dependency sequence.
It does not mirror active branches, workflow runs, or temporary review blockers.

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
G4C2 realization decision closure    accepted at 2856304755a86cb93f46888159a475d5ed17c064
G4C2 program/binding realization     accepted at 891c0a2a70b9501d756a019a2ce2e9fbed690269
```

G4B owns the backend-neutral logical program layer: bounded WGSL admission, typed entry
points and binding declarations, explicit program interfaces, pipeline and binding
descriptors, deterministic pipeline descriptors, runtime binding compatibility, and the
agreement vocabulary consumed by private realization.

G4C1 owns private context/device-generation-bound realization for buffers, textures,
texture views, samplers, and query sets. Generic GPU resource identity remains separate
from renderer identity, and resource creation does not absorb G5 execution or retirement.

G4C2 owns private context/device-generation-bound realization for canonical WGSL programs,
bind-group layouts, pipeline layouts, and typed bind groups. The accepted boundary also
owns the single shared WGPU device-health and error-attribution authority used by G4C1 and
G4C2, direct pinned-Naga evidence, bounded authoritative realization registries,
single-flight publication, and typed resolution of G4B runtime bindings through G4C1.

Before G7, `SurfaceColor` remains presentation/render-attachment and copy state rather
than a G4C1 logical shader resource. G4C2 therefore rejects sampled/storage
`SurfaceColor` bindings while preserving ordinary output, present, and copy behavior.

## Current G4C boundary

The accepted object-reference bridge ladder is:

```text
G4C1  CurrentRenderResourceBridge            deleted by G4C2
G4C2  CurrentRenderPipelineBridge            current accepted bridge
G4C3  CurrentRenderExecutionBridge           next successor
G5    deletes CurrentRenderExecutionBridge
```

Exactly one object-reference migration bridge may remain at an accepted G4C boundary.
A successor deletes its predecessor, carried predecessor terminals may only shrink, and
new terminals are limited to objects newly realized by the owning phase and still needed
by uncut successor consumers.

`CurrentRenderDeviceQueue` is a separate crate-private backend-operation loan, not a
second object-reference bridge. At the accepted G4C2 boundary:

- G4C1 resource creation no longer uses the loan;
- G4C2 shader-module, layout, and bind-group creation no longer uses the loan;
- all G4C1/G4C2 realization needed for a render batch completes before the loan begins;
- temporary G4C3 pipeline creation plus current G5 encoding/upload/submission/copy/
  map/readback operations remain inside the residual raw-operation interval.

G4C3 removes pipeline creation from that loan. G5 later migrates the remaining operation
classes and deletes the loan; neither ownership transfer is part of G4C2.

## Only next RunenGPU gate

Issue `#213` is completed. Issue `#214` is the only next RunenGPU implementation gate.
It remains unactivated until its implementation base is resolved from then-current
accepted `main` and a fresh dependency, pipeline-creation, bridge, cache, consumer, and
raw-WGPU census confirms the canonical G4C3 specification still matches current source.

G4C3 owns:

- private compute- and render-pipeline realization;
- complete semantic pipeline request keys and transactional reuse;
- agreement between G4C2 observed stage IO and explicit pipeline state;
- migration of remaining renderer/current pipeline-realization consumers;
- deletion of renderer-owned reusable G4 pipeline/cache authority;
- deletion of `CurrentRenderPipelineBridge` and replacement with exactly one narrow
  `CurrentRenderExecutionBridge` for the still-current G5 encoder;
- removal of compute/render pipeline creation from `CurrentRenderDeviceQueue`.

G4C3 does not acquire G5 command encoding, submission, progress, readback, cancellation,
retirement, or shutdown ownership; G7 surface ownership; RunenRender image-formation
semantics; package extraction; persisted backend caches; or public raw-WGPU authority.

## Ordered continuation

Issue `#188` remains the non-implementable G4C umbrella.

```text
#212 G4C1 private resource realization                         accepted
    -> #230 G4C2 presentation-surface boundary correction      accepted
        -> #233 G4C2 realization decision closure              accepted
            -> #213 G4C2 program/layout/bind-group realization accepted
                -> #214 G4C3 pipeline realization/final cutover next gate
                    -> separately designed G5 execution/lifecycle
```

No implementation child consumes an unmerged predecessor or correction branch as
authority. Each implementation gate begins from an explicit accepted default-branch
revision and receives its own exact-current-source census before implementation starts.

## Later RunenGPU program

The remaining program stays sequential and separately authorized:

- G5: work encoding, uploads, queue submission, progress, pressure, completion,
  asynchronous readback, cancellation, delayed retirement, and pending-work shutdown;
- G6: representative offscreen compute/render proof, shared render/non-render consumers,
  direct-WGPU comparisons, and cold/warm cost characterization;
- G7: surfaces, affinity, generations, device/surface loss, reconstruction facts, and
  explicit presentation-surface usage/capability policy;
- G8: operational conformance, reproducibility facts, diagnostics, shutdown, cache, and
  residual reach-through audit;
- GX: clean transfer to `dornglut/runen-gpu` only after accepted G2-G8 evidence.

G5, G7, RunenRender implementation, and package extraction remain unauthorized until
their own accepted gates.

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
review are required. A green feature branch becomes accepted authority only after an
unchanged reviewed head is merged and the accepted default-branch revision is validated.
