---
title: WR-124 Operation Diff Apply And Rollback Contract
description: Ready-next implementation contract for PM-UI-DESIGNER-WB-005 typed UI Designer operations, deterministic diffs, apply/reject, undo/redo, rollback, and reload preservation.
status: active
owner: editor
layer: domain/ui-definition / domain/editor / app
canonical: false
last_reviewed: 2026-05-26
related:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md
  - ../../../design/accepted/ui-lab-operation-driven-visual-authoring-design.md
  - ../../../design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../reports/closeouts/pm-ui-designer-wb-004-catalog-hierarchy-canvas-inspector-v1/closeout.md
---

# WR-124 Operation Diff Apply And Rollback Contract

## Goal

Implement `PM-UI-DESIGNER-WB-005` by routing normal UI Designer Workbench
authoring gestures through typed operations, deterministic diffs, apply/reject,
undo/redo, rollback, and reload-preservation paths.

This contract covers operation and recovery behavior only:

- insert, move, reorder, layout edit, token-reference edit, binding-reference
  edit, and accessibility edit produce typed operations or typed rejection
  diagnostics;
- UI template tree operations reuse the generic `domain/ui/ui_definition`
  visual layout contracts;
- editor/workbench-specific edits use editor-owned operation adapters and
  diagnostics;
- app state owns interaction translation, draft mutation, session history,
  apply/reject, rollback, and reload-preservation evidence;
- deterministic diffs and textual patch previews are derived review artifacts,
  not source truth.

It must not implement scenario evidence packets, performance baselines,
game-runtime HUD behavior, final usage docs/examples, or a perfectionist
no-gap audit. Those remain owned by `PM-UI-DESIGNER-WB-006` through
`PM-UI-DESIGNER-WB-008`.

## Source Of Truth

- Production milestone: `PM-UI-DESIGNER-WB-005` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-124` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Accepted visual layout and deterministic diff design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`.
- Accepted generic persistence, migration, diff, and activation design:
  `docs-site/src/content/docs/design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md`.
- Accepted Editor Lab operation-driven authoring design:
  `docs-site/src/content/docs/design/accepted/ui-lab-operation-driven-visual-authoring-design.md`.
- Accepted Editor Lab persistence, apply, and rollback design:
  `docs-site/src/content/docs/design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md`.
- Completed product-surface closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-004-catalog-hierarchy-canvas-inspector-v1/closeout.md`.
- Generic visual layout operations:
  `domain/ui/ui_definition/src/visual_layout/operation.rs` module
  `visual_layout::operation`.
- Generic visual layout diffs and apply diagnostics:
  `domain/ui/ui_definition/src/visual_layout/diff.rs` and
  `domain/ui/ui_definition/src/visual_layout/apply.rs` modules
  `visual_layout::diff` and `visual_layout::apply`.
- Editor operation facade:
  `domain/editor/editor_definition/src/operation.rs` module `operation`.
- App draft, history, apply, and rollback state:
  `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`.
- UI Designer provider projection:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`.

## Readiness

`PM-UI-DESIGNER-WB-005` starts from `designing` after
`PM-UI-DESIGNER-WB-004` completed. Existing accepted UI Designer and UI Lab
designs cover the operation, deterministic diff, apply/reject, undo/redo,
rollback, migration, activation, and recovery doctrine, but this production
milestone had no linked WR row or decision-complete contract.

Architecture governance kickoff was run for this scope on 2026-05-26. The
bounded owner split remains:

- `domain/ui/ui_definition` owns generic UI visual layout operation mechanics,
  deterministic visual layout diffs, generic persistence/diff/activation
  descriptors, and UI-definition diagnostics.
- `domain/editor/editor_definition` owns editor/workbench-specific operation
  envelopes, operation reports, diff families, validation, and adapters from
  editor definition vocabulary into generic UI operations where legal.
- `domain/editor/editor_shell` owns app-neutral surface contracts and retained
  review/composition view models.
- `apps/runenwerk_editor` owns concrete UI Designer interaction translation,
  draft mutation, operation history, apply/reject execution, rollback,
  activation bridging, reload preservation, and runtime evidence.

No ADR is required while WR-124 preserves these ownership and dependency
boundaries. Require an ADR or accepted design update before creating a global
operation engine, moving app draft mutation/history/project IO into
`domain/ui`, making retained previews source truth, or moving concrete
activation/rollback execution into a domain crate.

After this planning action, expected next workflow is:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-005 --roadmap WR-124
```

The command must report the next promotion or implementation action before any
product code changes start.

## Promotion Readiness

After the ready-next intake row and this contract were applied,
`task production:plan -- --milestone PM-UI-DESIGNER-WB-005 --roadmap WR-124`
reported:

- production milestone state: `ready_next`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-004:support_only`, `WR-046:support_only`,
  `WR-096:completed`, `WR-108:completed`, `WR-120:completed`,
  `WR-122:completed`, and `WR-123:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-124 --state current_candidate --evidence "<accepted evidence>"
```

Accepted promotion evidence:

- accepted UI Designer product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`;
- accepted visual layout and deterministic diff design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`;
- accepted generic persistence, migration, diff, and activation design:
  `docs-site/src/content/docs/design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md`;
- accepted UI Lab operation-driven authoring design:
  `docs-site/src/content/docs/design/accepted/ui-lab-operation-driven-visual-authoring-design.md`;
- accepted UI Lab persistence, apply, and rollback design:
  `docs-site/src/content/docs/design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md`;
- completed PM004 product-surface closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-004-catalog-hierarchy-canvas-inspector-v1/closeout.md`;
- this active WR-124 operation/apply/rollback contract.

Promotion may proceed only while this evidence remains true and the production
goal still selects `PM-UI-DESIGNER-WB-005`.

## Implementation Scope

Allowed future source scopes:

- `domain/ui/ui_definition/src/visual_layout/` module subtree: extend generic
  visual layout operation, apply, diff, and diagnostic contracts only for
  source-neutral UI template edits.
- `domain/ui/ui_definition/src/persistence_activation/` module subtree: reuse
  or extend generic migration, diff, activation-preflight, and fail-closed
  diagnostics without adding app project IO.
- `domain/editor/editor_definition/src/operation.rs` module `operation` and
  focused sibling modules if needed: add editor/workbench-specific operation
  families, diff families, diagnostics, and adapters.
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`: expose app-neutral review, apply, rollback, undo/redo,
  and typed rejection view models.
- `domain/editor/editor_shell/src/composition/` module subtree: render
  operation review and recovery state from editor-shell view models.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`: dispatch operations over app-owned draft documents,
  maintain session history, build deterministic review data, apply/reject, and
  rollback/reload state.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`: project operation, diff, apply, rollback, undo/redo, and
  typed diagnostic state into the UI Designer workbench.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` module
  `dispatch_shell_command`: bridge retained UI actions into operation, apply,
  reject, undo, redo, rollback, and reload commands.
- `apps/runenwerk_editor/src/shell/applied_editor_definition/` module subtree:
  record activation attempts and previous-state preservation when apply reaches
  live app state.
- `apps/runenwerk_editor/src/shell/providers/tests.rs` and
  `apps/runenwerk_editor/src/shell/tests.rs` modules: add focused runtime
  tests for operation/diff/apply/rollback UI Designer workflows.

Explicit non-goals:

- no scenario matrix or source-versioned evidence packet breadth;
- no performance baseline or resize instrumentation;
- no game-runtime HUD behavior or game UI runtime projection implementation;
- no broad project-file publishing workflow;
- no final usage docs, examples, or track handoff closeout;
- no perfectionist no-gap claim.

## Acceptance Criteria

- Insert, move, reorder, layout edit, token-reference edit,
  binding-reference edit, and accessibility edit produce typed operations or
  typed rejection diagnostics.
- Accepted operations produce deterministic operation diffs and reviewable
  textual patch previews.
- Rejected and preview-only operations fail closed and do not enter history.
- Undo and redo restore draft UI Designer documents and refresh hierarchy,
  canvas, inspector, diagnostics, and review surfaces.
- Apply, reject, rollback, and reload preserve authored package and document
  state, including failed-activation inputs where the current runtime reaches
  activation.
- Generic UI truth remains outside `apps/runenwerk_editor`; app code owns
  execution and evidence only.

## Validation

Future implementation validation must include at minimum:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-005 --roadmap WR-124
cargo fmt --package ui_definition --package editor_definition --package editor_shell --package runenwerk_editor
cargo test -p ui_definition visual_layout
cargo test -p ui_definition persistence_activation
cargo test -p editor_definition operation
cargo test -p editor_shell editor_lab
cargo test -p runenwerk_editor editor_lab_operation
cargo test -p runenwerk_editor ui_designer
cargo test -p runenwerk_editor apply_selected
cargo test -p runenwerk_editor rollback_selected
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
```

Run `./quiet_full_gate.sh` only if implementation expands beyond this focused
operation/apply/rollback slice or before final runtime-proven track closeout
requires full validation.

## Stop Conditions

Stop before product code changes if:

- `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` no longer selects
  `PM-UI-DESIGNER-WB-005` or linked `WR-124` work;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-005 --roadmap WR-124`
  reports a design, metadata, dependency, promotion, or current-candidate
  blocker;
- implementation would put generic UI document, Canonical UI IR, operation,
  token, recipe, migration, diff, or activation truth into
  `apps/runenwerk_editor`;
- implementation would make retained previews, provider output, runtime widget
  ids, project files, or activation reports the source of truth;
- operation support requires direct runtime-widget mutation instead of typed
  operation contracts;
- a required behavior belongs to `PM-UI-DESIGNER-WB-006` or later;
- validation or generated roadmap/production checks fail.

## Closeout Requirements

The later implementation closeout must include:

- this contract path;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-005 --roadmap WR-124`
  output classification used before coding;
- focused test evidence for typed operation dispatch, deterministic diffs,
  typed rejection diagnostics, undo/redo, apply/reject, rollback, reload
  preservation, and UI Designer surface refresh;
- confirmation that `WR-123` catalog, hierarchy, canvas, inspector,
  diagnostics, and review surfaces did not regress;
- roadmap evidence update for `WR-124`;
- production milestone evidence update only when PM005 acceptance criteria are
  actually met;
- rerun `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH`.

Expected completion quality for `WR-124` is `runtime_proven` only if focused
runtime or headless provider evidence proves the actual operation,
diff/review, apply/reject, undo/redo, rollback, and reload paths. Otherwise
close as `bounded_contract` with explicit known gaps.
