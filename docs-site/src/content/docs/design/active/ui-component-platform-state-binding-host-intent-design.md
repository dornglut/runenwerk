---
title: UI Component Platform State Binding And Host Intent Design
description: Phase 6 design for reusable control state ownership, binding declarations, and host intent proposals without host-owned mutation.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-26
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./ui-component-platform-input-gesture-device-design.md
  - ./ui-component-platform-catalog-discovery-inspection-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# UI Component Platform State Binding And Host Intent Design

## Status

This is the Phase 6 planning and acceptance design for `PT-UI-COMPONENT-PLATFORM-006`.

It follows Phase 1 `ControlPackage` / `ControlKernel` contracts, Phase 2 authoring ergonomics, Phase 3 story-proof envelopes, Phase 4 catalog/discovery/inspection, and Phase 5 input/gesture/device declarations. It defines reusable control state ownership, binding declarations, edit lifecycle declarations, validation state declarations, host intent proposals, and route/capability decision facts. It does not authorize app/editor/game mutation, host command execution, persistence, domain-specific rules, runtime widget behavior, runtime mount eligibility, text editing implementation, canvas behavior, Gallery previews, Designer UX, Workbench behavior, renderer behavior, or ECS behavior.

## Existing Authority

`ui_controls` owns reusable control semantics and may declare state buckets, binding requirements, validation state, edit lifecycle, and host intent proposal shape.

`ui_program` and route/capability contracts own route identity and authorization vocabulary consumed by hosts.

Apps, editor, game hosts, and domain crates own actual state truth, command execution, persistence, authorization decisions, domain-specific validation rules, and mutation.

Gallery, Workbench, UI Designer, docs, and agents consume state/binding/intent declarations through catalog and inspection facts. They do not own reusable control semantics.

## Problem

Reusable controls can now declare packages, stories, catalog facts, input modes, gestures, and device facts. They still need a shared way to describe which state they own, which state is host-fed, which values are editable previews, and which host actions they propose.

Without a shared contract, later controls could duplicate state ownership language or silently mutate host truth from reusable UI code. That would blur boundaries between component semantics and product behavior.

## Decision

Add a reusable state binding and host intent declaration layer for the UI Component Platform.

The component layer owns declarations. It may describe state buckets, binding shape, edit lifecycle, validation state, and proposed host intent. It must not execute commands, authorize routes, persist state, mutate app/editor/game truth, or own domain-specific rules.

Correct ownership split:

```text
ui_controls
  owns state bucket names, binding declarations, validation state facts,
  edit lifecycle declarations, host intent proposal shape, and inspection summaries.

ui_program / routing contracts
  own route IDs, schema versions, and capability vocabulary.

apps / editor / game hosts
  own authorization decisions, command execution, persistence, route handling,
  domain validation, and mutation.

Gallery / Workbench / UI Designer / docs / agents
  consume declarations; they do not own reusable control semantics.
```

## Proposed Contract Shape

The first implementation should prefer one focused module:

```text
domain/ui/ui_controls/src/state.rs
```

Split later only when stable responsibilities require it.

Candidate public concepts:

```text
ControlStateBucket
ControlStateBindingKind
ControlStateBindingRequirement
ControlStateDescriptor
ControlEditLifecycle
ControlValidationState
ControlHostIntentProposal
ControlHostIntentKind
ControlRouteCapabilityDecision
ControlStateCapabilitySummary
ControlStateInspectionFact
```

The final names may differ after inspecting nearby conventions, but the responsibilities should remain:

- declare state buckets such as transient, preview, committed, focus, hover, drag, animation, host-fed, and package-owned;
- declare read, write, collection, option, and selection binding shapes;
- distinguish clean, dirty, read-only, invalid, warning, and pending-validation states;
- describe live edit, commit edit, cancel edit, and rollback edit lifecycle points;
- describe host intent proposals without executing them;
- reference route IDs, schema versions, and capability names without authorizing them;
- expose route/capability decision facts as host-supplied inspection facts;
- expose summaries that catalog/inspection can consume;
- avoid product-specific commands, persistence, domain rules, or host mutation.

## Minimum Phase 6 Scope

The first implementation pass should prove the contract with a small declaration model:

```text
ControlStateDescriptor
ControlStateBucket
ControlStateBindingRequirement
ControlEditLifecycle
ControlHostIntentProposal
ControlRouteCapabilityDecision
ControlStateCapabilitySummary
```

Minimum state buckets:

```text
transient
preview
committed
focus
hover
drag
animation
host-fed
package-owned
```

Minimum binding kinds:

```text
read
write
collection
option
selection
```

Minimum edit lifecycle facts:

```text
live-edit
commit-edit
cancel-edit
rollback-edit
```

Minimum validation facts:

```text
clean
dirty
read-only
invalid
warning
pending-validation
```

Minimum host intent facts:

```text
proposal
route-id
route-schema-version
required-capability
host-decision
blocked-reason
```

## Non-Goals

Do not implement:

- app/editor/game mutation;
- host command execution;
- route authorization logic;
- persistence;
- domain-specific validation rules;
- game/editor rule ownership;
- runtime widget behavior;
- runtime mount eligibility;
- text editing implementation;
- input event execution;
- drawing semantics;
- canvas document truth;
- Gallery previews;
- Designer UX;
- Workbench behavior;
- Surface2D;
- SpatialCanvas;
- NodeCanvas;
- PortGraphCanvas;
- ProgressionTreeView;
- TrackSurface;
- Timeline;
- transitions;
- renderer behavior;
- ECS behavior.

## Boundary Rules

- State buckets are ownership declarations, not storage engines.
- Bindings describe shape and requirements, not live data pipes.
- Host intent proposals are proposed actions, not executed commands.
- Route/capability decisions are host-supplied facts, not component-owned authorization.
- Dirty/read-only/invalid/warning states are reusable UI facts, not domain validation truth.
- Catalog/inspection may expose state and intent declarations as read-only data.
- Story proof may reference state and intent requirements, but Phase 6 does not run stories.
- Runtime mount eligibility remains future-gated.

## Acceptance Criteria

Phase 6 is implementation-complete only when:

- reusable state/binding/host-intent declarations exist in `ui_controls`;
- declarations distinguish state buckets, binding kinds, validation states, edit lifecycle points, host intent proposals, and route/capability decisions;
- declarations can be summarized for catalog/inspection without mutating host truth;
- focused tests prove state buckets, binding requirements, host intent proposals, host decision facts, inspection summaries, and no host mutation;
- no app/editor/game mutation, route authorization logic, persistence, domain rules, runtime widget behavior, runtime mount, text editing implementation, canvas behavior, Gallery, Designer, Workbench, renderer, or ECS behavior is implemented.

## Candidate Implementation Scope

The first implementation pass may touch:

```text
domain/ui/ui_controls/src/state.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/tests/control_state_contract.rs
domain/ui/ui_controls/tests/control_state_catalog_contract.rs
```

Use catalog inspection only to expose read-only state/intent summaries. Do not add app/editor/gallery code in Phase 6.

## Test Plan

Required focused tests for the future implementation pass:

```text
cargo test -p ui_controls control_state
cargo test -p ui_controls control_input
cargo test -p ui_controls control_catalog
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_controls control_story_proof
cargo test -p ui_controls control_authoring
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
```

Required static checks:

```text
cargo fmt --all --check
cargo check -p ui_controls
git diff --check
```

Recommended test cases:

- state descriptor records transient, preview, committed, host-fed, and package-owned buckets;
- binding declarations distinguish read, write, collection, option, and selection bindings;
- validation state facts distinguish clean, dirty, read-only, invalid, warning, and pending-validation;
- edit lifecycle distinguishes live edit, commit edit, cancel edit, and rollback edit;
- host intent proposals reference route IDs and required capabilities without executing commands;
- route/capability decision facts remain host-supplied inspection facts;
- catalog/inspection summaries expose state and intent declarations read-only;
- no declaration makes a control runtime-mount eligible.

## Phase 6 Implementation Gate

Before writing Rust code, confirm:

- this design is accepted;
- Phase 5 remains green on the branch base;
- existing `ui_controls` package, catalog, input, story-proof, authoring, and validation contracts are still current;
- planning records name Phase 6 implementation as active;
- no new stop condition has been triggered.

## Handoff

Start implementation only after this design and planning state are accepted. The first implementation pass should add the smallest state binding and host intent declaration contract and focused tests. Do not implement host command execution, route authorization, app/editor/game mutation, persistence, domain validation rules, runtime widget behavior, text editing, canvas behavior, Gallery previews, Designer UX, Workbench behavior, renderer behavior, ECS behavior, or runtime mount eligibility in Phase 6.
