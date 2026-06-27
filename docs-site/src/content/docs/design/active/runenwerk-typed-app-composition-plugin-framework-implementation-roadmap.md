---
title: Runenwerk Typed App Composition Plugin Framework Roadmap
description: Proposed proof-gated continuation path for typed app composition, plugin suites, extension points, host profiles, and domain contribution registries without premature shared extraction.
status: active
owner: workspace
layer: architecture
canonical: false
last_reviewed: 2026-06-27
related_docs:
  - ./runenwerk-typed-app-composition-plugin-framework-design.md
  - ./runenwerk-domain-workbench-north-star.md
  - ./ui-program-architecture.md
  - ./ui-program-architecture-owner-map.md
  - ./ui-component-platform-base-control-packages-design.md
  - ./ui-component-platform-ownership-realignment-design.md
  - ../../workspace/workflow-lifecycle.md
  - ../../guidelines/domain-program-architecture-pattern.md
  - ../../guidelines/programming-principles.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# Runenwerk Typed App Composition Plugin Framework Roadmap

## Status

Documentation folder status: `active`.

Workflow lifecycle state: `proposed-design` companion.

Target decision: `proposed-design -> accepted-direction` for the companion design only.

This document is not an active production track, not active implementation, and not a replacement for the current UI Component Platform roadmap.

If the companion design is accepted, this roadmap remains a reference until a planning update promotes a specific slice into `active-planning` or `active-implementation`.

## Decision summary

Use typed app composition as an architecture direction now, then prove it through narrow owner tracks before extracting shared primitives.

Correct sequence:

```text
1. Accept the design as architecture direction.
2. Use it to review/refactor Phase 11 UI base controls.
3. Prove contribution/preset/lowering ergonomics inside UI owner crates.
4. Add host compatibility proof only when Phase 11 evidence is credible.
5. Prove one non-UI domain such as MaterialNodes or ProcgenNodes.
6. Extract shared structure only after two proofs and an accepted extraction design.
```

## Roadmap principles

### Prove before extracting

Follow the Domain Program extraction rule:

```text
design the pattern
-> prove one domain
-> prove a second domain
-> extract only repeated domain-neutral primitives
```

### Keep work scriptless-readable

Every phase must be understandable by reading repository files. Local commands may add evidence, but they are not the authority for design acceptance or planning state.

### Preserve the current UI roadmap

The immediate UI Component Platform sequence remains:

```text
011 Base control packages
012 Generic interaction
013 Overlay / popup / layering
014 Minimum text editing
015 Generic text
016 Surface2D
017 SpatialCanvas
018 NodeCanvas
019 PortGraphCanvas
020 ProgressionTreeView
021 TrackSurface / Timeline
022 Transitions / effects
023 Adoption gates
024 Runtime-proven closeout
```

This roadmap informs those phases but does not collapse them into one meta-framework implementation.

### Keep ECS in the runtime lane

ECS system registration should remain ergonomic and runtime-focused. Static contribution catalogs and domain descriptors should be validated and frozen before runtime execution.

## Phase A — Accept architecture direction

### Goal

Accept the companion design as `accepted-direction`, not as implementation authorization.

### Scope

```text
runenwerk-typed-app-composition-plugin-framework-design.md
runenwerk-typed-app-composition-plugin-framework-implementation-roadmap.md
```

### Acceptance criteria

```text
- Design doc has lifecycle state and decision summary.
- Roadmap doc is explicitly not active implementation.
- No root docs are bloated.
- No active planning state is changed.
- No Rust implementation is included.
- No shared extraction is authorized.
```

### Output if accepted

A decision-register entry may record:

```text
State transition:
  proposed-design -> accepted-direction

Decision:
  Accept the typed app-composition/plugin framework design as cross-cutting architecture direction and reference.

Not authorized:
  production-track, active-implementation, foundation/meta, shared plugin extraction.

Follow-up:
  Use this direction to review/refactor Phase 11 base control package work. Reconsider production-track promotion only after UI and one non-UI proof.
```

## Phase B — UI control contribution proof

### Goal

Use Phase 11 as the first proof of contribution-style authoring and descriptor/catalog lowering.

### Scope

Implementation stays UI-local unless an accepted owner design says otherwise.

Target shape:

```text
BaseControlsPlugin
UiControls extension point or local equivalent
ControlContribution
ControlDef builder
control presets
field groups
theme groups
ControlCompiler / lowering layer
ControlCatalog projection
ControlInspection projection
```

Base controls:

```text
Label
Button
InspectorField
ColorPicker
ActionPrompt
ListView
TreeView
TableView
```

### Non-goals

```text
full pointer/keyboard runtime interaction
overlay/popup/layering
text editing
runtime mount eligibility
backend renderer behavior in ui_controls
generic shared plugin framework extraction
foundation/meta
```

### Acceptance criteria

```text
- Base controls are declared through contribution-style APIs or a staged equivalent.
- Per-control files declare intent, not catalog row construction.
- Descriptor/catalog/inspection output remains stable or intentionally migrated.
- Generated descriptor plumbing is centralized in lowering/compiler code.
- Phase 12 interaction remains future work.
```

### Validation envelope

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo test -p ui_controls control_package
cargo test -p ui_controls control_catalog
cargo test -p ui_controls control_layout
cargo test -p ui_controls control_render
cargo test -p ui_controls base_control
git diff --check
```

## Phase C — Host profile and compatibility proof

### Goal

Make host support explicit without forcing controls or surfaces to manually list every host.

### Target outputs

```text
UiHostProfile
UiHostContribution
UiHostCatalog
HostCompatibilityMatrix
HostCompatibilityDiagnostic
```

### Initial host profiles

```text
Editor
Game
WorldSpace
Headless
```

### Acceptance criteria

```text
- Host profiles are inspectable.
- Compatibility is derived from requirements/capabilities.
- Headless compatibility distinguishes descriptor/proof support from runtime interaction support.
- ui_surface and ui_composition vocabulary are used correctly.
- Runtime behavior is not changed unless explicitly authorized.
```

## Phase D — Small app recipe proof

### Goal

Prove app composition on a small recipe before applying it to the full editor.

Preferred first recipe:

```text
headless_ui_proofs
```

Reason: fewer runtime/backend side effects and direct value for AI/CI proof workflows.

### Target outputs

```text
AppRecipe
ProductProfile
PluginGraphReport
AppAssemblyReport
Frozen ControlCatalog snapshot or equivalent report
HostCompatibilityMatrix
ProofManifestReport
```

### Acceptance criteria

```text
- One small recipe can be read from files.
- The recipe expands into a plugin graph.
- Extension contributions are reported.
- Frozen catalogs or reports are produced.
- No generated view is required to understand the workflow.
```

## Phase E — Non-UI second-domain proof

### Goal

Prove that typed extension/contribution machinery is not UI-specific before extracting shared primitives.

Preferred first proof:

```text
MaterialNodes
```

Alternative:

```text
ProcgenNodes
```

### Material proof target

```text
BaseMaterialNodesPlugin
MaterialNodes extension point or local equivalent
MaterialNodeContribution
MaterialNodeDef builder
MaterialNodeRegistry
MaterialNodeCatalog
MaterialNodeLowering
Material node proof fixtures
```

### Procgen proof target

```text
TerrainProcgenPlugin
ProcgenNodes extension point or local equivalent
ProcgenNodeContribution
ProcgenNodeDef builder
ProcgenNodeRegistry
ProcgenNodeCatalog
DeterminismPolicy
SeedPolicy
BudgetClass
Procgen proof fixtures
```

### Acceptance criteria

```text
- The non-UI proof reuses only structural ideas, not UI semantics.
- Contribution authoring is cleaner than central hardcoded catalog growth.
- Descriptor/catalog output remains domain-owned.
- Proof metadata is explicit.
- No shared foundation extraction is performed in this phase.
```

## Phase F — Report and snapshot proof

### Goal

Make app composition and contribution lowering inspectable for humans, CI, and AI agents.

Minimum useful reports:

```text
PluginGraphReport
AppAssemblyReport
HostCompatibilityMatrix
```

Generated snapshots remain optional evidence mirrors unless a later accepted design promotes a specific generated file to a narrow contract.

## Phase G — Runtime plan bridge

### Goal

Connect frozen app assembly output to runtime startup without letting runtime own static domain truth.

Target shape:

```text
FrozenAppAssembly
  PluginGraphSnapshot
  FrozenCatalogs
  HostCompatibilityMatrix
  RuntimeSystemPlan
  HostAdapterPlan
  RenderBackendPlan
  ProofPlan
```

ECS consumes frozen catalogs and runtime artifacts. ECS does not mutate static contribution registries.

## Phase H — Controlled shared extraction

### Goal

Extract only domain-neutral primitives after UI and at least one non-UI domain prove the same structure.

Candidate extraction scope:

```text
PluginId
PluginVersion
PluginManifest
PluginDependency
ExtensionPointId
PluginDiagnostic
PluginGraphReport
PluginBuildReport
```

Do not extract:

```text
ControlContribution
MaterialNodeContribution
ProcgenNodeContribution
AbilityNodeContribution
domain-specific registry validation
domain-specific descriptor lowering
domain-specific catalog semantics
universal graph node meaning
universal evaluator/compiler semantics
```

## Stop conditions

Stop and redesign if any phase requires:

```text
domain semantics in foundation
ECS-owned package/catalog truth
renderer-owned product truth
hidden global mutable plugin registry
generic graph interpretation in hot paths by default
SurfaceSlot replacing current ui_composition vocabulary without accepted migration
plugin mutation after runtime freeze
shared extraction before UI plus one non-UI proof
root docs becoming full design manuals
```

## Continuation after this PR

After this PR is accepted or merged, the correct next agenda is:

```text
1. Review PR #37 against this accepted direction.
2. Decide whether to replace its explicit base_control inventory with contribution/preset/lowering authoring.
3. Keep Phase 11 implementation UI-local.
4. Do not start shared plugin framework implementation.
5. Do not open Phase 12 until Phase 11 is validated and closed.
```
