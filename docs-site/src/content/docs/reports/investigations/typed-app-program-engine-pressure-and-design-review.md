---
title: Typed App Program Engine Pressure And Design Review
description: Critical review of the Typed App Program design against complete-design expectations, Runenwerk engine/runtime use cases, and no-half-measure future-use pressure.
status: active
owner: ui
layer: reports
canonical: false
last_reviewed: 2026-07-04
related_docs:
  - ./typed-app-program-current-state-investigation.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
  - ../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../guidelines/domain-program-architecture-pattern.md
  - ../../workspace/complete-design-gate.md
  - ../../domain/ui/architecture.md
---

# Typed App Program Engine Pressure And Design Review

## Purpose

Critically review whether `typed-app-program-and-ui-proof-design.md` is broad enough for Runenwerk's long-term goals and whether engine systems such as physics, asset loading, streaming, LOD, scheduler/runtime plans, render-resource preparation, and world streaming should be part of the typed app-program design.

This review is not implementation authorization. It hardens the design gate and records required boundaries before any code planning starts.

## Review Verdict

The design direction is correct, but the first version was too implicit about engine/runtime pressure.

The design is not allowed to become a narrow UI helper. It must be understood as a future cross-domain app-program architecture where UI is only the first proof.

However, engine/runtime systems must not be implemented in the first proof. They must be considered as future pressure and classified into owner boundaries.

Correct interpretation:

```text
Complete architecture target:
  Typed App Program handles model/action/reducer/effect/replay structure.

First proving slice:
  Headless Counter App Proof exercises the full app-program loop with no engine side effects.

Future engine integrations:
  asset loading, streaming, LOD, physics, scheduler, renderer, and world systems consume or contribute through host/domain effect proposals and domain-owned programs.

Forbidden shortcut:
  stuffing engine semantics into the UI proof, app core, foundation/meta, or a universal plugin framework.
```

## No-Half-Measure Assessment

### What Is Complete Enough

The design is complete in these areas:

- it defines the durable architectural spine;
- it separates app model/action/reducer/effect/replay from `UiProgram`;
- it keeps UI definitions and controls behavior-free;
- it reuses current `UiProgram`, route/event, host, binding, evaluator, and testing infrastructure;
- it defines a first proof that exercises the full model/action/reducer/effect/replay path;
- it rejects callbacks, hidden state, global stores, shared plugin extraction, `foundation/meta`, ECS-owned semantics, and renderer-owned product truth;
- it defines negative cases and reporting requirements;
- it blocks shared extraction until UI plus a non-UI proof pass.

### What Was Too Weak

The design was too weak or implicit in these areas:

- engine subsystem pressure was not explicit enough;
- asset loading and streaming were not classified;
- LOD and budget selection were not classified;
- physics and simulation effects were not classified;
- scheduler/runtime plan interaction was not classified;
- resource lifetime and async effect states were not classified;
- networking/remote execution was not classified;
- save/load/persistence and migration implications for app programs were not explicit enough;
- multi-host execution scenarios were not explicit enough.

This does not require implementation now. It requires explicit owner/boundary rules before implementation planning.

## Engine And Runtime System Classification

Typed App Program must consider engine systems as future integration pressure, but it must not own their domain semantics.

| System | Should Typed App Program Consider It? | Should First Proof Implement It? | Owner Boundary |
| --- | --- | --- | --- |
| Asset loading | Yes, as async/effect pressure | No | Asset/resource domain owns loading semantics; app program may emit inert load request effects later. |
| Streaming | Yes, as host/runtime pressure | No | World/streaming/runtime owners own chunk/page/asset residency. App program may observe status snapshots or request proposals later. |
| LOD | Yes, as projection/budget pressure | No | Renderer/world/streaming/domain owners choose LOD. App program may carry budget intent or view-model projection constraints, not LOD algorithms. |
| Physics | Yes, as command/effect pressure | No | Physics/simulation domains own simulation and mutation. App program may emit domain command proposals, not solve physics. |
| Scheduler | Yes, as runtime plan pressure | No | Runtime/scheduler/app owners execute systems. App program provides replay/effect facts, not scheduling policy. |
| Renderer resource preparation | Yes, as derived artifact pressure | No | Render domains own GPU/resource preparation. App program must not own renderer truth. |
| World streaming / spatial residency | Yes, as world host pressure | No | World/streaming owners manage spatial residency. App program may request/observe through host facts later. |
| Asset import | Yes, as domain-program pressure | No | Asset/import domain owns import program, validation, IO, and products. App program may route action/effect proposals later. |
| Persistence / save-load | Yes, as model/version pressure | No for first proof except deterministic snapshots | App/domain owns persistence; app program requires explicit model/action/schema versions. |
| Networking / remote execution | Yes, as host compatibility pressure | No | Network/remote host owners own transport/session. App program may require serializable traces and action payloads. |
| Undo/redo/history | Yes, as reducer trace pressure | No for generic engine history | Domain/app owns history policy; app program records reducer trace and can support later replay/history adapters. |
| Long-running async operations | Yes, as effect lifecycle pressure | No beyond explicit deferred design | Host/effect owners manage pending/completed/failed states. App program records requested effect and observed completion event later. |

## Engine Pressure Requirements For The Design

The design must explicitly preserve these extension points:

```text
AppEffectPlan
  must represent inert proposals for host/domain work.

AppReplayTrace
  must distinguish pure reducer transitions from host-observed effect completions.

AppModelSnapshot
  must be versioned and serializable enough for deterministic proof and migration.

HostCompatibilityMatrix
  must report whether a host supports requested action/effect capabilities.

AppViewProjection
  must support host-fed status snapshots, not direct engine queries.

Shared extraction
  must remain blocked until at least two domains prove the same structural primitive.
```

## Required Effect Taxonomy Before Implementation Planning

The design should not implement all effects now, but implementation planning must reserve names and boundaries for:

```text
NoEffect
HostCommandProposal
DomainCommandProposal
AssetLoadRequest
AssetImportRequest
StreamingResidencyRequest
WorldQueryRequest
PhysicsCommandProposal
SimulationCommandProposal
NavigationRequest
FocusRequest
AsyncRequest
RemoteRequest
PersistenceRequest
```

The first proof may only use:

```text
NoEffect
```

But the report and data model must not make `NoEffect` the only possible effect shape.

## Required Host Fact Taxonomy Before Implementation Planning

Future hosts need to provide status/fact snapshots without letting the app program query engine internals directly.

Reserve conceptual space for:

```text
HostCapabilityFacts
HostResourceFacts
HostAssetStatusFacts
HostStreamingStatusFacts
HostWorldResidencyFacts
HostSchedulerFacts
HostPhysicsStatusFacts
HostRenderBudgetFacts
HostNetworkSessionFacts
HostPersistenceFacts
```

First proof may use only:

```text
HeadlessHost capability facts
```

## Boundary Rules For Engine Systems

### Asset Loading

Typed App Program may later express:

```text
request asset X by stable ref
observe asset status snapshot
reject unsupported asset loading capability
record effect request in replay trace
```

It must not own:

```text
file IO
import pipelines
asset decoding
resource cache policy
GPU upload
asset database mutation
```

### Streaming

Typed App Program may later express:

```text
request or declare desired residency
observe streaming status
record host compatibility for streaming effects
```

It must not own:

```text
chunk streaming algorithms
spatial residency policy
cache eviction
network streaming transport
world paging
```

### LOD

Typed App Program may later carry:

```text
view projection budget facts
host-provided LOD status
render/world budget compatibility
```

It must not own:

```text
mesh/SDF LOD generation
streaming LOD selection
renderer LOD policy
simulation LOD policy
```

### Physics And Simulation

Typed App Program may later emit:

```text
PhysicsCommandProposal
SimulationCommandProposal
```

It must not own:

```text
physics stepping
collision detection
constraint solving
rigid/soft body semantics
simulation rollback policy
world mutation execution
```

### Scheduler And Runtime Plan

Typed App Program may later contribute:

```text
pure replay trace
requested effect list
runtime compatibility facts
```

It must not own:

```text
system scheduling
threading
job graph execution
ECS storage layout
runtime freeze points
```

Runtime scheduling belongs to app/runtime/platform owners and only after typed app composition is proven.

### Renderer And Render Resources

Typed App Program may later produce or reference:

```text
UI output facts
view projection facts
render budget intent
```

It must not own:

```text
GPU resources
render graph execution
render pass semantics
material compiler semantics
renderer resource lifetime
```

### Persistence And Migration

Typed App Program must require:

```text
AppProgramVersion
AppModelVersion
AppActionVersion
schema references
migration diagnostics
replay compatibility checks
```

Persistence itself remains app/domain-owned.

## Additional Considerations Missing From The First Design

Before implementation planning, add or explicitly accept these considerations:

1. **Schema versioning**: model/action/effect schemas need versions and rejection behavior.
2. **Replay determinism**: replay must not depend on wall-clock time, IO, scheduler order, random state, or host-specific resource availability unless those are explicit inputs.
3. **Effect completion events**: async effect completion must enter as an explicit event, not as hidden callback mutation.
4. **Host capability denial**: unsupported effects must produce diagnostics and fail closed.
5. **Authorization/policy**: host route/action/effect authorization must be separate from reducer logic.
6. **Resource references**: asset/resource references must use stable resource refs, not file paths or runtime handles.
7. **Error model**: route resolution, reducer, projection, effect planning, host compatibility, and replay each need distinct diagnostic namespaces.
8. **State ownership**: app model snapshots must not duplicate ECS/world/asset source truth; they can store projection state or app-owned state only.
9. **Multi-host compatibility**: same app program may be compatible with headless/editor/game hosts differently; this must be reportable.
10. **Security/sandboxing**: app effects must not be arbitrary code execution or dynamic service calls.
11. **Performance/budgeting**: replay traces and snapshots need size/budget diagnostics before becoming product-grade.
12. **Tooling/AI review**: reports must be readable without running the whole engine.

## Complete-Design Gate Review

| Gate Area | Current Status | Required Hardening |
| --- | --- | --- |
| Target capability | Mostly complete | Add explicit engine/runtime pressure classification. |
| Owner map | Good for UI proof | Add engine owner boundary matrix. |
| Vocabulary | Good baseline | Reserve effect/fact taxonomy for asset/streaming/physics/scheduler pressure. |
| Data model | Good baseline | Add schema/revision/migration and effect completion implications. |
| Feature support | Good for UI proof | Add engine/runtime pressure rows. |
| Future-use pressure | Too UI/editor/game-focused | Add asset loading, streaming, LOD, physics, scheduler, persistence, remote execution. |
| Validation | Good docs/future-code baseline | Add future checks for deterministic replay and effect rejection. |
| Stop conditions | Good baseline | Add engine-specific stop conditions. |
| Implementation contract | Correctly deferred | Must not proceed until hardening is incorporated or accepted as companion authority. |

## Required Stop Conditions To Add To Implementation Planning

Stop if implementation tries to:

```text
perform asset IO in app-program proof
query engine runtime directly from reducer
store ECS/world/physics state as AppModelSnapshot source truth
execute physics/simulation/render/streaming effects inside UI crates
use runtime handles or file paths as durable app actions
make host capabilities implicit
make async effect completion a hidden callback
make LOD or streaming policy app-program-owned
make scheduler/runtime plan app-program-owned
make renderer resources app-program-owned
skip schema/version diagnostics for app actions/effects
```

## Recommendation

PR #66 should remain draft until either:

1. the main design document is updated to include this engine/runtime pressure explicitly; or
2. this review is treated as companion authority and linked from the PR body as required hardening for the implementation-planning gate.

Preferred path:

```text
Keep PR #66 docs-only.
Keep investigation and design.
Add this review as design-gate hardening.
Run docs validation and diff check.
Merge only after review agrees the implementation-planning gate must consume this review.
```

## Answer To The Engine Question

Engine systems are not completely separate from Typed App Program.

They are separate in ownership and execution, but they are relevant as future integration pressure.

Correct relationship:

```text
Typed App Program owns structure:
  model snapshots
  actions
  reducers
  effect plans
  replay traces
  compatibility reports

Engine/domain systems own meaning and execution:
  physics
  asset loading
  streaming
  LOD
  scheduler
  renderer resources
  world mutation
  network transport
```

Typed App Program should be designed to emit and replay inert proposals and host facts involving those systems later. It must not implement or own those systems in the first proof.

## Final Review Verdict

The design is directionally right and not a throwaway UI helper.

It is not yet strong enough to be called complete without the engine/runtime pressure recorded here.

After this review is accepted or folded into the design, the next valid step remains implementation planning for:

```text
Typed App Program UI Proof 001 — Headless Counter App Proof
```

No implementation should begin before that planning contract names exact files, owners, validation, reports, stop conditions, and engine/system non-ownership rules.
