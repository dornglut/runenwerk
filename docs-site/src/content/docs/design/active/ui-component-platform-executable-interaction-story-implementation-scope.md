---
title: UI Component Platform Executable Interaction Story Implementation Scope
description: Exact active-implementation scope for the Tier 5 executable generic interaction story proof-host core slice.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-29
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

Lifecycle state: `active-implementation`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-012A`.

This is the exact implementation scope for the first Tier 5 executable generic interaction story **proof-host core** slice. It authorizes implementation inside the files and crates named below only.

Architecture acceptance remains separate from implementation evidence. This scope does not mark Phase 12A complete.

## Target outcome

Implement a minimal but real Tier 5 proof-host core for generic reusable interaction:

```text
one executable generic interaction story
  -> deterministic replay mode
  -> live proof-host mode fed by UiInputEvent / NormalizedInputSample
  -> shared runtime interaction formation after NormalizedInputSample
  -> semantic replay/live parity report
  -> InteractionVisualProof / InteractionProofRenderFrame
  -> UiStaticMountReport::from_frame
  -> zero host-command/product-mutation/overlay/text-edit boundary counters
```

The implementation must not create product behavior. It may create a proof host that can be driven live by input events and can return current proof/frame/report state.

This slice proves the executable interaction proof-host core. It does **not** require product-facing editor window, gallery page, UI Designer surface, or workspace panel exposure. Product-facing visual exposure is a later adoption slice unless it can be achieved inside this scope without touching editor shell surface registries.

## Exact owner files and crates

### `domain/ui/ui_story`

Authorized write files:

```text
domain/ui/ui_story/src/workflow/builtin.rs
domain/ui/ui_story/src/workflow/mod.rs
```

Read/verify-only files:

```text
domain/ui/ui_story/src/lib.rs
```

Allowed work:

- Add a built-in workflow profile for executable interaction proof.
- Add stable workflow node constants for interaction story definition, replay evidence, live proof-host evidence, replay/live parity evidence, and static mount evidence.
- Update built-in workflow profile iteration and tests.
- Keep `ui_story` as proof-envelope authority only.

Forbidden work:

- Do not make `ui_story` depend on `ui_runtime`, `ui_controls`, `ui_static_mount`, `runenwerk_editor`, or product/app crates.
- Do not execute filesystems, compilers, renderers, live hosts, static mounts, or editor behavior from `ui_story`.
- Do not reintroduce the old flat-stage API that the root `ui_story` test forbids.
- Do not edit `domain/ui/ui_story/src/lib.rs` unless implementation proves the workflow profile cannot be exported through the existing `workflow::*` path; if that happens, record the reason in the PR body.

Rationale:

`ui_story` currently exports V2 proof-envelope modules and tests that the old flat-stage API is not reintroduced. It already exports `workflow::*`, so the implementation should normally stay inside `workflow/builtin.rs` and `workflow/mod.rs`. It is the correct owner for workflow profile identity, not runtime execution.

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

Allowed work:

- Add `interaction_story_session.rs`.
- Add `InteractionStorySession` with replay and live modes.
- Add `InteractionStoryExecutionMode`.
- Add `InteractionStoryRunReport`.
- Add `InteractionStoryStepEvidence` if useful for incremental host updates.
- Add `InteractionReplayLiveParityReport`.
- Add expected-evidence validation helpers for the canonical Phase 12A story.
- Refactor `replay_interactions` so batch replay and live incremental `apply_sample` share the same internal apply path.
- Keep `MountedInteractionFixture`, `InteractionReplayScript`, `InteractionFormationReport`, `InteractionVisualProof`, and `InteractionProofRenderFrame` as the lower-tier assets that Tier 5 reuses.
- Add `phase12_executable_generic_interaction_story_session` or an equivalently named helper that builds the canonical story session from the compiled base-control package.
- Export the new session API from `domain/ui/ui_runtime/src/input/mod.rs`.

Required shared-path rule:

```text
replay_interactions(script)
  and
InteractionStorySession::apply_sample(sample)

must call the same internal fact/step application path after NormalizedInputSample.
```

Implementation shape:

- Keep `ReplayState` or its renamed equivalent as internal runtime state.
- Extract the current private replay loop into an internal helper such as `apply_interaction_step` or `apply_interaction_sample`.
- Use that helper from both batch replay and live session apply.
- Do not duplicate pointer, focus, keyboard, semantic, text-intent, suppression, no-target, or outcome rules.

Forbidden work:

- Do not execute host commands.
- Do not mutate product/app/editor/game state.
- Do not create overlays, popups, dropdowns, tooltips, modals, or layering behavior.
- Do not implement full text editing, caret, selection, text buffer mutation, clipboard, undo/redo, or text layout.
- Do not move story registry/discovery authority into `ui_runtime`.
- Do not introduce a generic plugin framework, generic plugin primitives, `foundation/meta`, or shared plugin extraction.

### `domain/ui/ui_input`

Authorized files:

```text
domain/ui/ui_input/src/facts.rs
domain/ui/ui_input/src/event.rs
domain/ui/ui_input/src/lib.rs
domain/ui/ui_input/tests/input_normalized_facts.rs
```

Allowed work:

- Add only minimal conversion helpers needed by the proof host, if existing constructors are insufficient.
- Prefer helper functions that convert existing `UiInputEvent` variants into `NormalizedInputSample` without adding control semantics.
- Preserve the existing rule that input facts are data only and do not decide reusable control behavior.

Forbidden work:

- Do not add Button/List/Tree/Table/Inspector semantics to `ui_input`.
- Do not add product command routing or mutation.
- Do not add host-specific windowing assumptions.

### `domain/ui/ui_static_mount`

Authorized files:

```text
domain/ui/ui_static_mount/src/lib.rs
domain/ui/ui_static_mount/tests/phase12_generic_interaction_static_mount.rs
domain/ui/ui_static_mount/tests/phase12_executable_interaction_story_static_mount.rs
```

Allowed work:

- Reuse `UiStaticMountReport::from_frame`.
- Add focused tests proving the executable story's current frame still passes static mount validation.
- Do not broaden static mount into live interaction ownership.

Forbidden work:

- Do not move story execution or interaction semantics into `ui_static_mount`.

### `apps/runenwerk_editor`

Authorized files:

```text
apps/runenwerk_editor/Cargo.toml
apps/runenwerk_editor/src/editor_features/mod.rs
apps/runenwerk_editor/src/editor_features/phase12a_interaction_proof_host.rs
apps/runenwerk_editor/tests/phase12a_interaction_proof_host.rs
```

Allowed work:

- Add `ui_runtime.workspace = true` to `apps/runenwerk_editor/Cargo.toml` only if the proof-host module requires it.
- Add a proof-host module under `editor_features` that adapts existing `UiInputEvent` values to normalized samples and feeds an `InteractionStorySession`.
- Keep the module proof-only. The name must make the Phase 12A/proof-host purpose explicit.
- The proof host may expose current `InteractionVisualProof`, `InteractionProofRenderFrame`, and static mount status for tests and future UI display.
- The proof host may be driven by real pointer/keyboard/text input events in tests without mutating editor state.

Forbidden work:

- Do not wire activation outcomes to editor commands.
- Do not mutate editor scene/product/runtime state.
- Do not add overlay, popup, or text editing behavior.
- Do not add a new editor workspace surface in this slice unless implementation proves it is required and remains diagnostic/proof-only.
- Do not modify `editor_shell` surface registries in this slice unless the proof-host module cannot be validated without it. If shell surface changes become necessary, stop and record a scope revision first.

Rationale:

`runenwerk_editor` already depends on `ui_input`, `ui_story`, `ui_static_mount`, and other UI crates, and exposes shell input entrypoints. There is no dedicated `ui_gallery` crate/module on this branch, so the first live proof host should be a narrow editor proof-host feature, not a broad gallery framework. The module is proof-host infrastructure, not product/editor behavior.

## Required public API shape

Names may be adjusted to nearby conventions, but the responsibilities must remain:

```text
InteractionStoryExecutionMode
InteractionStorySession
InteractionStoryRunReport
InteractionStoryStepEvidence
InteractionReplayLiveParityReport
phase12_executable_generic_interaction_story_session
phase12_executable_generic_interaction_expected_evidence
```

The runtime session must support:

```text
start replay session
start live session
apply one NormalizedInputSample
append sample to input log
return current report/proof/frame
finish run report
replay recorded live input log
compare semantic parity
```

The editor proof host must support:

```text
build canonical Phase 12A proof host
apply UiInputEvent pointer/key/text/semantic input
apply explicit focus sample when needed by the proof
return current proof/frame/report
return boundary counters
return static mount report from current frame
```

## Implementation order

Implement in this order:

```text
1. ui_runtime shared apply path
2. InteractionStorySession replay/live API
3. replay/live semantic parity report
4. canonical Phase 12A expected-evidence helper
5. ui_runtime executable story tests
6. ui_story executable interaction workflow profile
7. ui_static_mount executable story static mount test
8. runenwerk_editor proof-host adapter
9. docs/closeout update recording whether display is proof-host-core only or product-facing
```

Do not start with the editor proof host. The runtime shared path must be the behavior owner before any host adapter exists.

## Story workflow profile

Add a built-in `ui_story` workflow profile equivalent to:

```text
WORKFLOW_EXECUTABLE_INTERACTION_PROOF
```

Candidate workflow nodes:

```text
NODE_INTERACTION_STORY
NODE_INTERACTION_REPLAY
NODE_LIVE_INTERACTION_PROOF
NODE_REPLAY_LIVE_PARITY
NODE_INTERACTION_STATIC_MOUNT
```

The profile should remain a proof-envelope graph. App/runtime producers attach evidence to these nodes; `ui_story` does not execute those producers.

## Canonical Phase 12A story coverage

The canonical story must reuse the existing Phase 12 fixture controls:

```text
Button
Disabled Button
Inert Button
ActionPrompt
InspectorField
Read-only InspectorField
ListView
TreeView
TableView
Label
```

Required interactions:

```text
hover Button
press Button
release Button inside
press Button then release outside
focus Button
keyboard activate Button
focus ActionPrompt and activate
focus List and navigate
focus Tree and navigate
focus Table and navigate
send text-intent to InspectorField
send text-intent to Read-only InspectorField
attempt text-intent on non-text control
press Disabled Button
click outside all controls
```

Required visible evidence:

```text
hovered
pressed
captured
focused
focus-visible
activation-requested
action-intent
list-active-item-intent
tree-node-intent
table-cell-or-row-intent
text-intent-probe
read-only-text-intent-probe
disabled
suppressed
no-target
```

Required final-state distinction:

```text
pressed is observed during press
pressed is not current after release
captured is observed during press
captured is not current after release
focused may remain current
disabled remains current
read-only remains current
```

## Semantic replay/live parity

Compare these fields semantically:

- input step ids or normalized sample order;
- target resolution;
- focus resolution;
- state transitions;
- runtime facts;
- runtime events;
- semantic outcomes;
- suppressed events;
- no-target events;
- observed markers;
- final current states;
- boundary assertions.

Do not compare:

- wall-clock timing;
- raw OS event ids;
- pixel snapshots;
- animation interpolation;
- exact glyph layout or text wrapping;
- full primitive equality after every step.

## Tests required

Add or update focused tests equivalent to:

```text
cargo test -p ui_story executable_interaction_workflow
cargo test -p ui_runtime executable_interaction_story
cargo test -p ui_runtime --test interaction_replay_report
cargo test -p ui_static_mount phase12_executable_interaction_story
cargo test -p runenwerk_editor phase12a_interaction_proof_host
```

The tests must prove:

- `ui_story` exposes the executable interaction workflow profile without executing runtime behavior;
- replay mode produces expected evidence;
- live mode fed by samples/events produces expected evidence;
- replay and live use the same internal apply path;
- recorded live input log replays deterministically;
- semantic replay/live parity passes;
- current proof converts to `InteractionProofRenderFrame` and passes `UiStaticMountReport::from_frame`;
- boundary counters remain zero;
- editor proof host does not execute commands or mutate product/editor state.

## Full validation command set

Run and record:

```text
cargo fmt --all --check
cargo check -p ui_story
cargo check -p ui_controls
cargo check -p ui_input
cargo check -p ui_runtime
cargo check -p ui_static_mount
cargo check -p runenwerk_editor
cargo test -p ui_story executable_interaction_workflow
cargo test -p ui_controls control_interaction
cargo test -p ui_input input
cargo test -p ui_runtime interaction
cargo test -p ui_runtime executable_interaction_story
cargo test -p ui_runtime --test interaction_replay_report
cargo test -p ui_static_mount phase12_executable_interaction_story
cargo test -p runenwerk_editor phase12a_interaction_proof_host
python3 tools/docs/validate_docs.py
git diff --check
```

If exact test names differ after implementation, the PR body and closeout must record the actual focused command mapping.

## Manual validation expectation

Minimum proof-host-core validation:

```text
create/open the Phase 12A proof host core
apply pointer move over Button
verify current proof/frame shows hovered evidence
apply pointer down on Button
verify pressed and captured evidence
apply pointer up inside Button
verify activation-requested evidence and pressed/captured no longer current
apply press then release outside
verify cancellation/suppression/no activation evidence
apply disabled Button press
verify disabled/suppressed evidence
apply outside click
verify no-target evidence
apply focus and keyboard activation
verify focus-visible and activation evidence
apply List/Tree/Table navigation
verify intent markers
apply InspectorField text intent and read-only InspectorField text intent
verify probe markers without text-edit transactions
verify boundary counters remain zero
verify static mount report passes from current frame
```

If the proof host is not visually exposed in an editor window in this slice, record that explicitly. The slice is still acceptable only if the proof host processes live-shaped `UiInputEvent`/`NormalizedInputSample` input through the same runtime path and produces current proof/frame/report evidence. Product-facing gallery/window exposure remains a later adoption task unless implemented within this scope without touching shell registries.

## Product-facing display policy

This slice has two acceptable display outcomes:

```text
Accepted outcome A:
  proof-host core only
  live-shaped UiInputEvent / NormalizedInputSample input
  current proof/frame/report evidence
  no editor shell surface changes

Accepted outcome B:
  proof-host core plus diagnostic product-facing surface
  no product commands
  no product mutation
  no overlay/text editing
  no broad shell registry expansion
```

Outcome A is sufficient for this implementation scope. Outcome B is allowed only if it remains diagnostic/proof-only and stays inside the authorized files. If product-facing surface registration or shell/workspace registry edits are required, stop and revise scope.

## Stop conditions

Stop and record a scope revision if implementation requires:

- adding a parallel story runner outside `ui_story`;
- moving story execution into `ui_controls`;
- making `ui_story` depend on runtime/product crates;
- editing `domain/ui/ui_story/src/lib.rs` for anything other than a justified export necessity;
- duplicating interaction semantics in the editor proof host;
- live proof-host state changes that do not pass through `NormalizedInputSample` and `InteractionStorySession`;
- product/editor/game command execution;
- product/editor/game state mutation;
- overlay/popup/layering behavior;
- full text editing behavior;
- editor shell surface registry changes for a product-facing panel;
- shared plugin framework extraction;
- `foundation/meta`;
- generic plugin primitives;
- pixel-perfect replay/live parity.

## Completion criteria

Implementation is complete only when:

- exact tests above pass or the PR records the renamed equivalent commands;
- the executable workflow profile exists in `ui_story`;
- the runtime session supports replay and live apply through the same internal path;
- the canonical Phase 12A story produces expected evidence;
- live input logs replay deterministically;
- replay/live semantic parity passes;
- static mount validation passes from the current story frame;
- the editor proof host can be driven by `UiInputEvent`/normalized samples;
- boundary counters remain zero;
- docs and closeout record whether the delivered result is proof-host core only or also product-facing gallery/window display.
