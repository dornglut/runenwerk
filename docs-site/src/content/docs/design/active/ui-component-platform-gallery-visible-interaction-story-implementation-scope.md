---
title: UI Component Platform Gallery Visible Interaction Story Implementation Scope
description: Active implementation scope for PT-UI-COMPONENT-PLATFORM-012B, exposing the executable interaction story through a stable-key UI Lab workbench surface.
status: active
owner: ui
layer: domain/app
canonical: true
last_reviewed: 2026-06-30
related_docs:
  - ../../workspace/planning/active-work.md
  - ./ui-component-platform-executable-interaction-story-design.md
  - ./ui-component-platform-executable-interaction-story-implementation-scope.md
  - ./editor-tool-suite-registry-and-workbench-host-design.md
  - ./runenwerk-capability-workbench-target-architecture.md
---

# UI Component Platform Gallery Visible Interaction Story Implementation Scope

## Status

Lifecycle state: `active-implementation`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-012B`.

This scope closes the remaining visible Phase 12 acceptance gap by exposing the
Phase 12A executable interaction proof host through the Workbench tool-surface
system.

## Target outcome

```text
runenwerk.ui_lab tool suite
  -> runenwerk.ui_lab.interaction_story stable surface
  -> provider-owned local Workbench surface
  -> per-mounted-surface Phase12aInteractionProofHost session
  -> visible retained UI proof/report projection
  -> no legacy ToolSurfaceKind identity
  -> no product/editor/game command execution
  -> no product mutation
  -> no overlay or text-edit ownership
```

## Ownership split

`ui_runtime` remains the owner of `InteractionStorySession`, replay/live parity,
proof formation, and proof-frame projection.

`runenwerk_editor` owns only the concrete Workbench exposure:

- first-party internal UI Lab suite metadata;
- provider-family assignment to current concrete provider plumbing;
- provider-owned local retained UI projection;
- per-mounted-surface proof-host session storage;
- focused validation that the stable-key surface exists and has no legacy
  `ToolSurfaceKind` identity.

`editor_shell` remains the app-neutral Workbench contract owner. This slice must
not move reusable interaction semantics into shell composition, panel chrome, or
legacy enum identity.

## Authorized write scope

```text
apps/runenwerk_editor/src/shell/tool_suites/mod.rs
apps/runenwerk_editor/src/shell/tool_suites/ui_lab_tool_suite.rs
apps/runenwerk_editor/src/shell/workbench_host.rs
apps/runenwerk_editor/src/shell/surface_session.rs
apps/runenwerk_editor/src/shell/providers/m6_workspace.rs
apps/runenwerk_editor/tests/phase12b_ui_lab_interaction_story_surface.rs
docs-site/src/content/docs/design/active/ui-component-platform-gallery-visible-interaction-story-implementation-scope.md
docs-site/src/content/docs/design/active/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
```

Do not edit `domain/ui/ui_runtime` or `domain/ui/ui_story` unless a compile-time
export issue is found and the reason is recorded.

## Implementation notes

The durable visible surface identity is:

```text
runenwerk.ui_lab.interaction_story
```

The first implementation is a compiled-in first-party ToolSuite contribution,
matching the current Capability Workbench maturity layer for internal ToolSuites.
Numeric `SurfaceProviderId` values remain current concrete registry plumbing and
must not be treated as durable user-facing identity.

The current provider bridge may use an existing concrete provider id while the
provider registry still lacks stable provider ids. A future Workbench cleanup
should replace numeric provider handles with stable provider identity or derived
runtime indices.

## Acceptance criteria

- `runenwerk.ui_lab` is installed in the full editor Workbench composition.
- `runenwerk.ui_lab.interaction_story` is registered as a stable Workbench
  surface.
- The surface has no legacy `ToolSurfaceKind` compatibility identity.
- The surface uses `ToolSurfaceRoute::ProviderOwnedLocal`.
- Full editor Workbench validation proves provider support for the UI Lab
  provider family.
- Per-mounted surface session state carries `Phase12aInteractionProofHost`.
- The visible provider surface displays proof id, input-log count, static mount
  status, replay/live parity status, boundary counters, mounted controls,
  observed markers, current states, and report counts.
- Host command, product mutation, overlay, and text-edit counters remain zero.

## Non-goals

This scope does not implement:

- dynamic external plugin loading;
- provider-id ergonomics cleanup;
- a general component gallery framework;
- UI Designer authoring of the proof story;
- product/editor/game command routing from proof outcomes;
- overlays, popups, text editing, drag/drop, docking runtime, or accessibility
  tree implementation.

## Validation expectation

```text
cargo fmt --all --check
cargo check -p runenwerk_editor
cargo test -p runenwerk_editor phase12b_ui_lab_interaction_story_surface
cargo test -p runenwerk_editor phase12a_interaction_proof_host
cargo test -p ui_runtime executable_interaction_story
cargo test -p ui_static_mount phase12_executable_interaction_story
python tools/docs/validate_docs.py
git diff --check
```

If plain Windows MSVC workspace tests hit PDB limits, validate the full workspace
with:

```text
CARGO_PROFILE_DEV_DEBUG=0 cargo test --workspace
```

## Stop conditions

Stop and redesign if the visible surface requires:

- adding a new legacy `ToolSurfaceKind`;
- moving interaction semantics into shell composition or panel chrome;
- fake hover/press/focus state outside `Phase12aInteractionProofHost`;
- product/editor/game mutation;
- overlay or text-edit behavior;
- dynamic plugin loading;
- treating numeric provider ids as durable user-facing identity.
