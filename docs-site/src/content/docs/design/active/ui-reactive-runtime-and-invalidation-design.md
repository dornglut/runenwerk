---
title: UI Reactive Runtime And Invalidation Design
description: Long-term reactive UI runtime model for Runenwerk, covering declarative snapshots, dependency tracking, invalidation, retained UI state, incremental evaluation, output deltas, and proof reports.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-source-projection-and-program-lowering-design.md
  - ./typed-app-program-counter-proof-design.md
  - ./ui-program-architecture.md
  - ./ui-program-architecture-owner-map.md
---

# UI Reactive Runtime And Invalidation Design

## Status

Active long-term UI design direction. This document defines the UI reactivity,
invalidation, retained-state, and incremental-evaluation model needed for a
mature Runenwerk UI framework. It does not authorize implementation by itself.

## Decision

Runenwerk UI should be declarative and reactive.

Declarative means:

```text
UI source/projection describes desired UI for a given model/source/host snapshot.
The app does not imperatively mutate retained controls or renderer primitives.
```

Reactive means:

```text
when model, source, package, theme, input, runtime state, time, or host data changes,
tracked dependencies are invalidated,
affected derived UI facts are recomputed,
retained runtime state is preserved or reset by policy,
and the host receives updated output facts or output deltas.
```

Reactivity is not callback mutation. Reactivity is reportable dependency-driven
update propagation.

## Update Cycle

The standard update cycle is:

```text
1. Input change arrives
   app model revision
   UiSource revision
   package/catalog revision
   theme revision
   host data revision
   runtime UI state revision
   time/animation tick

2. Change is converted into UiInvalidationCause

3. Dependency graph computes UiInvalidationScope

4. Runtime selects affected work
   projection
   source validation
   normalization
   interaction formation
   program formation
   artifact rebuild
   evaluator update
   layout
   style
   text
   hit testing
   focus/navigation
   accessibility
   render output

5. Runtime preserves or resets retained state by policy

6. Evaluator produces UiOutput or UiOutputDelta

7. Host applies output and side-effect proposals

8. Runtime emits UiUpdateReport
```

## Core Data Structures

### UiEvaluationRevision

Stable monotonically increasing revision for a UI evaluation pass.

Fields:

```text
revision id
program id
artifact id
host profile id
source revision refs
model revision refs
theme revision refs
package revision refs
runtime state revision refs
time tick if applicable
```

### UiDependencyGraph

Tracks dependencies between inputs and derived facts.

Dependency classes:

```text
model field dependency
source node dependency
package descriptor dependency
theme token dependency
style rule dependency
layout constraint dependency
text shaping dependency
input/focus dependency
runtime state dependency
host data dependency
time/animation dependency
surface dependency
```

### UiInvalidationCause

Reasons a UI update begins:

```text
AppModelChanged
UiSourceChanged
PackageCatalogChanged
ThemeChanged
HostDataChanged
InputChanged
FocusStateChanged
ScrollStateChanged
TextEditStateChanged
AnimationTick
SurfaceChanged
AccessibilityAction
ArtifactCacheInvalidated
ManualProofStep
```

### UiInvalidationScope

Describes what must be recomputed:

```text
Projection
SourceValidation
Normalization
InteractionFormation
ProgramFormation
ArtifactBuild
BindingEvaluation
Layout
Style
Text
HitTesting
FocusNavigation
Accessibility
Animation
RenderOutput
InspectionOnly
```

### UiDirtySet

Fine-grained dirty record:

```text
dirty source ids
dirty program node ids
dirty binding ids
dirty layout ids
dirty style ids
dirty text ids
dirty hit regions
dirty navigation scopes
dirty accessibility nodes
dirty render packets
dirty surface ids
```

### UiRuntimeStateStore

Retained runtime state keyed by stable source/program identity.

State classes:

```text
focus
hover
pressed
active
scroll
text edit
IME composition
drag/drop
capture
animation
transition
popup stack
accessibility focus
host-fed ephemeral data
package-owned control state
```

### UiStateRetentionPolicy

Defines how state survives changes.

Policies:

```text
PreserveIfSourceIdStable
PreserveIfProgramNodeCompatible
ResetOnControlKindChange
ResetOnSchemaVersionChange
ResetOnHostProfileChange
MigrateWithControlMigration
DropWithDiagnostic
```

### UiOutputDelta

Incremental output for hosts that support delta application:

```text
added nodes
removed nodes
updated layout rects
updated visual packets
updated text packets
updated hit regions
updated focus state
updated accessibility facts
updated event routes
updated surface visibility
```

Hosts that cannot apply deltas may consume full `UiOutput`.

## Snapshot Semantics

Every evaluation operates over explicit input snapshots:

```text
AppModelSnapshot
UiSourceSnapshot
UiProgramSnapshot
UiRuntimeArtifactSnapshot
HostDataSnapshot
UiRuntimeStateSnapshot
ThemeSnapshot
PackageCatalogSnapshot
```

Snapshots make replay, proof, diagnostics, and live-preview rollback possible.

An event handler or route-action resolution sees the snapshot for the current
step. If newer host/app state arrives during the step, it starts a new evaluation
revision instead of mutating the current step underfoot.

## Binding Semantics

Bindings connect UI source/program facts to app/host data facts. They are not
arbitrary callbacks.

Binding kinds:

```text
ReadBinding
WriteProposalBinding
ComputedBinding
CollectionBinding
SelectionBinding
TextInputBinding
HostDataBinding
```

Binding rules:

```text
read bindings may observe snapshots
write bindings emit proposals or typed actions
computed bindings are deterministic and side-effect free
collection bindings must report insert/remove/move/update diffs
invalid binding paths produce diagnostics
bindings must record dependency edges
```

## Incremental Evaluation Strategy

Runenwerk should support three evaluation modes:

```text
FullEvaluation
IncrementalEvaluation
ProofEvaluation
```

Full evaluation recomputes all derived facts.

Incremental evaluation recomputes only affected scopes based on dependency and
dirty sets.

Proof evaluation may force deterministic full or bounded incremental evaluation
and must emit all intermediate reports.

Hot paths consume artifacts and dependency tables. They must not interpret
generic authoring graphs by default.

## Retained State Rules

Retained runtime state is allowed and required. The rule is that it must be
runtime state, not source truth.

Allowed retained state:

```text
focus
hover
pressed
scroll
text cursor/selection
IME composition
animation progress
popup stack
drag state
capture state
control-local ephemeral state
```

Forbidden retained state:

```text
app model truth
source truth
package catalog truth
renderer source truth
route/action semantics
material/procgen/gameplay meaning
```

## Reactive Update Reports

Every reactive update should produce `UiUpdateReport`:

```text
update id
evaluation revision
causes
snapshots consumed
invalidated scopes
dirty set summary
retained state decisions
artifact cache hits/misses
evaluator mode
output delta summary
diagnostics
source-map links
host application summary
```

## Counter Example

Counter increment flow:

```text
UiEventPacket(counter.increment)
-> RouteActionResolution accepted
-> CounterModel revision N -> N+1
-> UiInvalidationCause::AppModelChanged(counter.count)
-> Projection dirty: counter screen source
-> Binding/text dirty: counter.count label
-> If count becomes 10, screen branch changes to counter.win
-> Runtime applies state retention policy
-> UiOutputDelta replaces counter screen subtree with win screen subtree
-> Host receives frame/update facts
-> UiUpdateReport records transition
```

The win screen transition is reactive because it follows from model revision
change and projection dependency invalidation. It is not a UI-side mutation.

## Live Editing Integration

Live source edits use the same reactive path:

```text
UiSourceChanged
-> source validation
-> normalization
-> program diff
-> artifact invalidation
-> retained state policy
-> preview output update
-> diagnostics overlay
```

If source is invalid, the host keeps the last known good artifact and displays
source-map diagnostics.

## Rejected Runtime Shapes

Reject:

```text
callback-driven mutation as reactivity
hidden global subscriptions
controls mutating app/game state directly
renderer owning dirty state truth
ECS owning source/program semantics
unreported invalidation
unbounded full rebuilds in all hot paths
retained runtime state keyed only by unstable runtime ids
```

## Acceptance Criteria For First Implementation Slice

A first reactive slice must prove:

```text
app model revision change invalidates projected UI
counter count label updates
count 10 switches to win screen through projection
state retention/reset decisions are reported
route rejection remains fail-closed
headless replay can reproduce every update step
no product code builds event packets or render primitives manually
```
