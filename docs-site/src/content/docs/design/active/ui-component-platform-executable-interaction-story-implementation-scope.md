---
title: UI Component Platform Executable Interaction Story Implementation Scope
description: Exact review scope for the Tier 5 executable base-controls interaction story proof-host core slice.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-30
related_docs:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ./ui-component-platform-executable-interaction-story-design.md
  - ./ui-component-platform-generic-interaction-design.md
  - ./ui-component-platform-story-proof-envelope-design.md
---

# UI Component Platform Executable Interaction Story Implementation Scope

## Status

Lifecycle state: `review`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-012A`.

This scope records the current PR #43 evidence and cleanup boundary for the first Tier 5 executable interaction story proof-host core. The implementation must use durable base-controls names for public APIs, stable ids, tests, and current implementation files. Phase or PR labels belong only in planning history, reports, and roadmap state.

Architecture acceptance remains separate from implementation evidence. This scope does not mark Phase 12A complete.

## Target outcome

```text
one executable base-controls interaction story
  -> deterministic replay mode
  -> live proof-host mode fed by UiInputEvent / NormalizedInputSample
  -> shared runtime interaction formation after NormalizedInputSample
  -> semantic replay/live parity report
  -> InteractionVisualProof / InteractionProofRenderFrame
  -> UiStaticMountReport::from_frame
  -> zero host-command/product-mutation/overlay/text-edit boundary counters
```

The implementation must not create product behavior. Product-facing Gallery or editor-window exposure remains separate future work under `PT-UI-GALLERY-001`.

## Authorized files and crates

### `domain/ui/ui_story`

Authorized files:

```text
domain/ui/ui_story/src/workflow/builtin.rs
domain/ui/ui_story/src/workflow/mod.rs
domain/ui/ui_story/tests/executable_interaction_workflow.rs
```

Allowed work:

- Add a built-in executable interaction workflow profile.
- Keep `ui_story` as workflow/evidence-envelope authority only.
- Do not make `ui_story` depend on runtime, controls, static mount, editor, or product crates.

### `domain/ui/ui_runtime`

Authorized files:

```text
domain/ui/ui_runtime/Cargo.toml
domain/ui/ui_runtime/src/input/generic_interaction.rs
domain/ui/ui_runtime/src/input/generic_interaction_fixture.rs
domain/ui/ui_runtime/src/input/generic_interaction_visual_frame.rs
domain/ui/ui_runtime/src/input/interaction_story_session.rs
domain/ui/ui_runtime/src/input/mod.rs
domain/ui/ui_runtime/tests/interaction_replay_report.rs
domain/ui/ui_runtime/tests/executable_interaction_story.rs
```

Durable public/runtime names:

```text
BASE_CONTROLS_GENERIC_INTERACTION_PROOF_ID
BASE_CONTROLS_EXECUTABLE_INTERACTION_STORY_ID
base_controls_generic_interaction_fixture
base_controls_generic_interaction_positive_script
base_controls_generic_interaction_negative_scripts
base_controls_generic_interaction_proof_frame
base_controls_executable_interaction_story_session
base_controls_executable_interaction_expected_evidence
InteractionStoryExecutionMode
InteractionStorySession
InteractionStoryRunReport
InteractionStoryStepEvidence
InteractionReplayLiveParityReport
```

Forbidden work:

- No phase-shaped public aliases or compatibility shims.
- No host command execution.
- No product/editor/game state mutation.
- No overlays, popups, dropdowns, tooltips, modals, or layering behavior.
- No full text editing, caret, selection, text buffer mutation, clipboard, undo/redo, or IME/composition behavior.
- No story registry/discovery authority moved into `ui_runtime`.
- No generic plugin framework, generic plugin primitives, `foundation/meta`, or shared plugin extraction.

### `domain/ui/ui_input`

Authorized files:

```text
domain/ui/ui_input/src/facts.rs
domain/ui/ui_input/src/event.rs
domain/ui/ui_input/src/lib.rs
domain/ui/ui_input/tests/input_normalized_facts.rs
```

Allowed work:

- Add normalized input facts and minimal conversion helpers.
- Keep facts as data only.
- Do not add Button/List/Tree/Table/Inspector semantics to `ui_input`.

### `domain/ui/ui_static_mount`

Authorized files:

```text
domain/ui/ui_static_mount/src/lib.rs
domain/ui/ui_static_mount/Cargo.toml
domain/ui/ui_static_mount/tests/base_controls_generic_interaction_static_mount.rs
domain/ui/ui_static_mount/tests/base_controls_executable_interaction_story_static_mount.rs
```

Allowed work:

- Reuse `UiStaticMountReport::from_frame`.
- Validate the base-controls generic/executable interaction proof frames.
- Do not move story execution or interaction semantics into `ui_static_mount`.

### `apps/runenwerk_editor`

Authorized files:

```text
apps/runenwerk_editor/Cargo.toml
apps/runenwerk_editor/src/editor_features/mod.rs
apps/runenwerk_editor/src/editor_features/base_controls_interaction_proof_host.rs
apps/runenwerk_editor/tests/base_controls_interaction_proof_host.rs
```

Durable editor-side proof-host name:

```text
BaseControlsInteractionProofHost
```

Allowed work:

- Adapt existing `UiInputEvent` values to normalized samples.
- Feed samples into `InteractionStorySession`.
- Expose current proof/frame/report/static-mount evidence for tests and future display.

Forbidden work:

- No editor command execution.
- No product/editor scene mutation.
- No overlay, popup, or text editing behavior.
- No editor shell surface registry changes for product-facing display in PR #43.

## Required validation

```text
cargo fmt --all --check
cargo check -p runenwerk_editor
cargo test -p runenwerk_editor base_controls_interaction_proof_host
cargo check -p ui_controls
cargo check -p ui_input
cargo check -p ui_runtime
cargo check -p ui_static_mount
cargo test -p ui_story executable_interaction_workflow
cargo test -p ui_controls control_interaction
cargo test -p ui_input input
cargo test -p ui_runtime executable_interaction_story
cargo test -p ui_runtime --test interaction_replay_report
cargo test -p ui_static_mount base_controls
python tools/docs/validate_docs.py
git diff --check
```

## Completion criteria

Implementation is complete only when:

- no current public API, stable id, reusable fixture helper, current test file, or active implementation-scope file uses phase-shaped names;
- no compatibility alias or hidden shim remains;
- replay and live apply share the same runtime path after normalized input;
- semantic replay/live parity passes;
- static mount validation passes from the current story frame;
- boundary counters remain zero;
- docs and PR body accurately say product-facing Gallery exposure is separate future work.

## Stop conditions

Stop and redesign if implementation requires product-facing gallery/shell surface registration, product commands, product mutation, overlays, full text editing, a parallel story runner, shared plugin framework extraction, generic plugin primitives, `foundation/meta`, or pixel-perfect replay/live parity.
