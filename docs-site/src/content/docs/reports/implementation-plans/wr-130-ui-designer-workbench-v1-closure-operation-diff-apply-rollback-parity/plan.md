---
title: WR-130 UI Designer Workbench V1 Closure Operation Diff Apply Rollback Parity Contract
description: Design-first implementation contract for PM-UI-DESIGNER-WB-V1-CLOSURE-004 operation-driven edit parity, deterministic diffs, undo/redo, apply/reject/reload/rollback, and fail-closed diagnostics.
status: active
owner: editor
layer: domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-lab-operation-driven-visual-authoring-design.md
  - ../../../design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md
  - ../../../design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md
  - ../../../design/accepted/ui-designer-workbench-product-design.md
related_reports:
  - ../wr-129-ui-designer-workbench-v1-closure-recipe-catalog-insertion/plan.md
  - ../../closeouts/pm-ui-designer-wb-v1-closure-003-recipe-catalog-insertion-and-authoring-surface/closeout.md
  - ../../roadmap-intake/2026-05-26-ui-designer-workbench-v1-closure-operati/proposal.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-130 UI Designer Workbench V1 Closure Operation Diff Apply Rollback Parity Contract

## Goal

Define the decision-complete design-first contract for
`PM-UI-DESIGNER-WB-V1-CLOSURE-004` and `WR-130` before any operation parity
product code starts.

`WR-130` may close only the operation-driven author workflow parity slice:
insert, move, reorder, layout edit, token-reference edit, binding-reference
edit, accessibility edit, deterministic patch preview, apply, reject, undo,
redo, reload, rollback, and fail-closed diagnostics over the same
source-versioned package state completed by `WR-128` and recipe insertion
surface completed by `WR-129`.

It must not claim scenario matrix evidence, game-runtime compatibility
evidence, performance baselines, final product closeout, or concrete game HUD
runtime behavior.

After the design-first contract and WR metadata were accepted, the current
readiness report is:

```text
Production milestone state: ready_next
Roadmap planning_state: ready_next
Roadmap blocker: B2
Roadmap dependencies: WR-129:completed
Milestone links WR item: yes
Next action: write_promotion_contract
Promotion preflight: promotable
Suggested command: task roadmap:promote -- --id WR-130 --state current_candidate --evidence "<accepted evidence>"
```

Accepted promotion evidence:

```text
Accepted PM-UI-DESIGNER-WB-V1-CLOSURE-004 design-first operation parity contract at docs-site/src/content/docs/reports/implementation-plans/wr-130-ui-designer-workbench-v1-closure-operation-diff-apply-rollback-parity/plan.md; WR-129 prerequisite is completed, design gates are accepted, write scopes are bounded, and production:plan reports WR-130 promotable.
```

After promotion, the implementation-readiness report is:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-129:completed
Milestone links WR item: yes
Next action: write_implementation_contract
```

This document is the decision-complete implementation contract for the
current-candidate row. The next coding pass may implement only the PM-004
operation diff/apply/rollback parity slice described here, then must run
focused tests, close out PM-004, archive WR-130, update production evidence,
and rerun `task ai:goal` before any PM-005 scenario/evidence work.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml` milestone
  `PM-UI-DESIGNER-WB-V1-CLOSURE-004`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml` item `WR-130`
  after the accepted intake is applied.
- Completed prerequisites:
  `WR-128` package/session source-truth closeout and `WR-129` recipe catalog
  insertion closeout.
- Generic UI operation primitives:
  `domain/ui/ui_definition/src/visual_layout/operation.rs` and
  `domain/ui/ui_definition/src/visual_layout/apply.rs`.
- Editor operation vocabulary and reducers:
  `domain/editor/editor_definition/src/operation.rs`.
- App-neutral authoring surface contracts:
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` and
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`.
- App-owned source-versioned session and workflow orchestration:
  `apps/runenwerk_editor/src/shell/self_authoring.rs`,
  `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`, and
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`.
- App-owned package, apply, reload, and rollback evidence:
  `apps/runenwerk_editor/src/shell/editor_lab_project/mod.rs`.

## Readiness

`WR-130` is ready for implementation only after all of these conditions stay
true:

- `WR-129` is completed and archived with PM-003 closeout evidence.
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-004 --roadmap WR-130`
  reports `planning_state: current_candidate`, `blocker: B2`, and next action
  `write_implementation_contract`.
- The accepted operation, persistence, and binding design gates remain
  accepted.
- The roadmap row write scopes include every Rust module, test module,
  closeout report, roadmap artifact, and production artifact needed by this
  bounded operation parity slice before product-code edits start.
- The architecture-governance decision below remains valid; if implementation
  needs a different ownership direction, stop for ADR/design work first.

## Ownership And Invariants

Owners:

- `domain/ui/ui_definition` owns generic UI edit primitives, Canonical UI IR,
  visual layout operation application, binding descriptors, recipes, and
  runtime-neutral diagnostics.
- `domain/editor/editor_definition` owns editor-facing operation taxonomy,
  operation reports, deterministic diffs, and reducers that translate generic
  UI primitives into editor definition document edits.
- `domain/editor/editor_shell` owns app-neutral action/view-model projection
  for workbench gestures and review surfaces.
- `apps/runenwerk_editor` owns selected document/node session state,
  operation dispatch, source-versioned history, apply/reject/reload/rollback
  orchestration, and evidence capture.

Invariants:

- Accepted user gestures route through typed operation reports before mutating
  draft documents, except explicitly documented read-only preview/review
  commands.
- Rejected operations preserve draft state, do not enter undo/redo history, and
  record typed diagnostics.
- Accepted operations update one selected source-versioned document path and
  refresh hierarchy, canvas, inspector, diagnostics, and diff projections from
  that source version.
- Undo/redo, apply/reject, reload, and rollback restore explicit snapshots and
  never treat provider/runtime caches as source truth.
- Binding and intent edits remain validated descriptors/proposals, not direct
  runtime command execution.

## Critical Product Chain Decisions

Source truth:

- Generic authored UI structure, authored ids, node paths, visual-layout edit
  kinds, target-profile compatibility, and visual-layout diagnostics stay in
  `domain/ui/ui_definition`.
- Editor-facing operation taxonomy, operation reports, deterministic operation
  diffs, and reducers stay in `domain/editor/editor_definition`.
- Shell actions and view models in `domain/editor/editor_shell` are projection
  and routing contracts. They are never the package source of truth.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` owns concrete mutable
  session state, selected document/node state, operation history, apply review,
  reload, and rollback orchestration for this app.
- `apps/runenwerk_editor/src/shell/editor_lab_project/mod.rs` owns app project
  IO and apply/rollback report vocabulary. It must not become generic UI IR.

Complete source-to-runtime chain:

1. A catalog, hierarchy, canvas, inspector, token, binding, or accessibility
   gesture emits a typed `EditorDefinitionSurfaceAction`.
2. Provider routing in
   `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` maps the
   action to a typed `ShellCommand`.
3. `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` builds an
   `EditorLabOperation` with selected document identity and target profile.
4. `domain/editor/editor_definition/src/operation.rs`
   `apply_editor_lab_operation` applies the operation against an
   `EditorDefinitionDocument` and returns an `EditorLabOperationReport`.
5. Accepted reports update the source-versioned draft and operation history in
   `SelfAuthoringWorkspaceState::apply_editor_lab_operation`; rejected reports
   preserve draft state and history.
6. Apply review, reject, reload, rollback, hierarchy, canvas, inspector,
   diagnostics, and diff surfaces reproject from that source-versioned draft or
   explicit applied snapshot.

Typed contracts that must replace ad hoc behavior:

- Recipe insertion must become an `EditorLabOperationKind` reducer path rather
  than a separate app-local mutation helper when it mutates source truth.
- Move, reorder, insert, and layout edits must use
  `UiVisualLayoutOperation` / `UiVisualLayoutEditKind` where the generic UI
  primitive already exists.
- Token-reference, binding-reference, and accessibility edits must be typed
  editor operation variants with deterministic diff families and fail-closed
  diagnostics. They may remain descriptor/proposal edits, but they cannot
  silently execute runtime commands.
- Apply/reject/reload/rollback evidence must reference operation report ids,
  document ids, source versions, diff rows, and explicit snapshot provenance.

Forbidden fallbacks:

- no provider-state, retained-output, or runtime-cache source truth;
- no stringly "success" reports without `EditorLabOperationReport`;
- no accepted mutation without deterministic diff changes unless the operation
  is explicitly preview-only or no-op and tested as such;
- no rejected mutation entering undo/redo history;
- no binding or intent edit that invokes concrete game runtime behavior under
  this row.

## Architecture Governance Decision

The architecture-governance kickoff for this task was run with scope:

```text
domain/ui/ui_definition; domain/editor/editor_definition; domain/editor/editor_shell; apps/runenwerk_editor/src/shell; docs-site/src/content/docs/workspace/production-tracks.yaml
```

No new ADR is required while the implementation preserves the ownership and
dependency direction above. An ADR or accepted design update is required before:

- moving generic UI operation truth into `apps/runenwerk_editor`;
- making editor shell projections authoritative source truth;
- introducing runtime/game HUD behavior under this editor closure row;
- changing dependency direction between `domain/ui`, `domain/editor`, and app
  shells.

## Implementation Scope

Allowed for the `WR-130` implementation:

- `domain/editor/editor_definition/src/operation.rs` module `operation`:
  extend `EditorLabOperationKind`, `apply_editor_lab_operation`, reducer
  helpers, diff-family mapping, validation diagnostics, and unit tests for the
  missing operation taxonomy.
- `domain/ui/ui_definition/src/visual_layout/operation.rs` module
  `visual_layout::operation` and
  `domain/ui/ui_definition/src/visual_layout/apply.rs` module
  `visual_layout::apply`: reuse or refine generic visual layout operation
  primitives only when PM-004 needs move/reorder/insert/layout parity that is
  not already covered.
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`: add app-neutral typed actions or view-model fields for
  move/reorder/layout/token/binding/accessibility gestures only when current
  actions cannot express them.
- `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`: render only app-neutral actions and
  review controls that route through the operation contract.
- `domain/editor/editor_shell/src/commands/shell_command.rs` module
  `shell_command`: add typed shell commands only when existing commands would
  force stringly dispatch.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`: map workbench actions to typed shell commands and project
  post-operation history, diagnostics, and diff readiness from source truth.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` module
  `dispatch_shell_command`: build `EditorLabOperation` values from selected
  source-versioned state and dispatch all mutating gestures through
  `dispatch_editor_lab_operation`.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`: consolidate source mutations through
  `SelfAuthoringWorkspaceState::apply_editor_lab_operation`, preserve
  accepted/rejected history semantics, and connect operation report provenance
  to apply/reject/reload/rollback evidence.
- `apps/runenwerk_editor/src/shell/editor_lab_project/mod.rs` module
  `editor_lab_project`: extend app-level evidence/report structures only if
  PM-004 apply/reload/rollback evidence cannot identify operation provenance
  from existing fields.
- Focused tests in the owning modules and shell/provider tests for accepted
  operations, rejected diagnostics, deterministic diffs, undo/redo, source
  version freshness, and apply/reject/reload/rollback parity.

Forbidden under `WR-130`:

- scenario matrix, game-runtime compatibility workflow, evidence packets, or
  performance baseline expansion; that remains
  `PM-UI-DESIGNER-WB-V1-CLOSURE-005`;
- final runtime-proven product closeout and handoff; that remains
  `PM-UI-DESIGNER-WB-V1-CLOSURE-006`;
- concrete game HUD runtime behavior;
- making provider state or retained UI output the definition source of truth.

## Implementation Steps

1. Inspect the current operation paths:
   `domain/editor/editor_definition/src/operation.rs`
   `apply_editor_lab_operation`,
   `apps/runenwerk_editor/src/shell/self_authoring.rs`
   `SelfAuthoringWorkspaceState::apply_editor_lab_operation`,
   `insert_selected_ui_recipe`, `undo_editor_lab_operation`,
   `redo_editor_lab_operation`, `build_selected_apply_review`,
   `reject_last_apply_review`, `reload_selected_from_last_applied`, and
   `rollback_selected`.
2. Move recipe insertion from the separate app-local mutation path into a typed
   `EditorLabOperationKind` path, preserving `expand_ui_recipe` compatibility
   diagnostics, namespace-stable authored ids, selected-node behavior, and
   deterministic diff output.
3. Add missing operation variants only for PM-004 gestures that are actually
   projected by the workbench: hierarchy/canvas move and reorder, layout edit,
   token-reference edit, binding-reference edit, and accessibility edit.
4. Reuse `UiVisualLayoutOperation` for generic insert/move/reorder/layout
   changes. Add editor operation wrappers only for editor-specific context,
   validation, selected document identity, target profile, or diff family.
5. Ensure every accepted operation produces an `EditorLabOperationReport` with
   deterministic `EditorLabOperationDiffChange` rows. Ensure every rejected
   operation stores diagnostics, preserves the draft document, and does not
   push undo/redo history.
6. Keep undo and redo snapshot-backed through
   `SelfAuthoringWorkspaceState::undo_editor_lab_operation` and
   `redo_editor_lab_operation`, and add coverage that each newly supported
   accepted operation can be undone and redone.
7. Connect apply review, reject, reload, and rollback evidence to the same
   operation report/diff vocabulary and source-version labels. Do not add
   scenario packets or performance counters under PM-004.
8. Update shell provider and dispatch tests so catalog, hierarchy, canvas, and
   inspector actions all route through typed commands and operation reports.
9. Add the PM-004 closeout report, update `production-tracks.yaml` evidence,
   archive WR-130 only after focused validation passes, then rerun generated
   roadmap/production/docs/planning/PUML checks.

## Validation

Focused validation for a later implementation:

```text
cargo test -p ui_definition visual_layout
cargo test -p editor_definition editor_lab_operation
cargo test -p editor_shell editor_lab
cargo test -p runenwerk_editor self_authoring
cargo test -p runenwerk_editor ui_designer
```

Planning and closeout validation:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-004 --roadmap WR-130
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task puml:validate
git diff --check
task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE
```

Run `./quiet_full_gate.sh` only if implementation expands beyond the bounded
operation parity and source-versioned recovery slice.

## Acceptance Criteria

This implementation-contract action is complete when:

- this file exists with `status: active`;
- `WR-130` is `current_candidate`, links
  `PM-UI-DESIGNER-WB-V1-CLOSURE-004`, and records the implementation write
  scopes needed for the bounded PM-004 slice;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-004 --roadmap WR-130`
  reports `write_implementation_contract`;
- roadmap, production, docs, planning, PUML, and whitespace checks pass;
- `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` is rerun before any
  implementation starts.

The PM-004 implementation closeout is complete only when:

- supported catalog, hierarchy, canvas, inspector, token, binding, and
  accessibility mutations route through typed operation reports;
- rejected operations preserve draft state, preserve undo/redo history, and
  surface typed diagnostics;
- accepted operations update the selected source-versioned document and
  produce deterministic diff rows with family, kind, path, before, and after
  values;
- undo, redo, apply review, reject review, reload last applied, and rollback
  are covered for the supported operation families;
- PM-004 closeout evidence records focused test results, roadmap archive
  evidence, production evidence, known gaps, and completion quality.

## Stop Conditions

Stop before code changes if:

- `task production:plan` no longer reports `WR-130` as `current_candidate`,
  `B2`, dependency `WR-129:completed`, and next action
  `write_implementation_contract`;
- any design gate or PM-003 closeout evidence gate is missing or no longer
  accepted/completed;
- implementation requires concrete game HUD runtime behavior, scenario matrix
  evidence, performance baselines, or final V1 closeout;
- a needed write target is outside the WR-130 write scopes;
- operation parity would require changing dependency direction between
  `domain/ui`, `domain/editor`, and `apps/runenwerk_editor`;
- focused validation fails and the failing behavior is outside the bounded
  PM-004 contract;
- source files change enough during the pass that the coordinator must rerun
  `task ai:goal` before continuing.

## Closeout Requirements

Closeout for PM-004 must add:

- a closeout report under
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-v1-closure-004-operation-diff-apply-rollback-parity/closeout.md`;
- PM-004 `production-tracks.yaml` evidence gate pointing to that closeout;
- WR-130 completion evidence in `roadmap-archive.yaml` after the roadmap row
  is completed/archived;
- focused validation transcript covering the Rust tests and generated
  docs/roadmap/production/planning/PUML checks listed above;
- explicit known gaps for PM-005 scenario/evidence/performance and PM-006
  final runtime-proven closeout, without claiming those outcomes from PM-004.

## Perfectionist Closeout Audit

Expected PM-004 completion quality is `runtime_proven` if the normal
source-versioned author workflow is proven through typed operation reports,
deterministic diffs, undo/redo, apply/reject, reload, and rollback behavior.
It is not `perfectionist_verified` because scenario matrix evidence,
game.runtime compatibility evidence, performance baselines, and final no-gap
product handoff remain intentionally assigned to PM-005 and PM-006.

The closeout audit must explicitly check for and reject these false completion
patterns:

- descriptor-only operation variants that are never consumed by app dispatch;
- status-panel-only readiness claims without source-versioned draft mutation;
- recipe insertion or inspector edits that bypass `EditorLabOperationReport`;
- apply/reload/rollback reports that cannot identify source version or
  operation provenance;
- fallback success paths that accept invalid token, binding, accessibility, or
  layout edits without typed diagnostics.
