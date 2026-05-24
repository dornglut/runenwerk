---
title: WR-085 UI Lab Operation-Driven Visual Authoring Core Contract
description: Promotion and implementation-readiness contract for PM-UI-LAB-004 operation-driven Editor Lab authoring.
status: active
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-24
related:
  - ../../../design/accepted/ui-lab-operation-driven-visual-authoring-design.md
  - ../../../design/accepted/ui-lab-app-hosted-editor-lab-surface-shell-design.md
  - ../../../design/active/ui-lab-productization-design.md
  - ../../../reports/closeouts/pm-ui-lab-003-app-hosted-editor-lab-surface-shell/closeout.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-085 UI Lab Operation-Driven Visual Authoring Core Contract

## Goal

Implement `PM-UI-LAB-004` by routing supported Editor Lab visual authoring
edits through typed `EditorLabOperation` envelopes with deterministic diffs,
validation diagnostics, app-owned edit history, undo/redo, and retained preview
or inspector refresh.

This contract covers promotion readiness and the future bounded
implementation slice only. It must not implement product code until WR-085 is
promoted by roadmap workflow and `task ai:goal -- --track PT-UI-LAB --scope
non-deferred` selects an implementation action.

## Source Of Truth

- Production milestone: `PM-UI-LAB-004` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-085` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted operation design:
  `docs-site/src/content/docs/design/accepted/ui-lab-operation-driven-visual-authoring-design.md`.
- Active productization design:
  `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`.
- Completed prerequisite shell closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-003-app-hosted-editor-lab-surface-shell/closeout.md`.

Current implementation sources to inspect before code changes:

- `domain/ui/ui_definition/src/visual_layout/operation.rs`
- `domain/ui/ui_definition/src/visual_layout/apply.rs`
- `domain/ui/ui_definition/src/visual_layout/diff.rs`
- `domain/ui/ui_definition/src/visual_layout/diagnostic.rs`
- `domain/editor/editor_definition/src/document.rs`
- `domain/editor/editor_definition/src/validate.rs`
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs`
- `domain/editor/editor_shell/src/composition`
- `domain/editor/editor_shell/src/commands/map_interactions.rs`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/tests.rs`
- `domain/editor/editor_core/src/history.rs`
- `apps/runenwerk_editor/src/editor_runtime/history`

## Promotion Readiness

`task production:plan -- --milestone PM-UI-LAB-004 --roadmap WR-085` reports
WR-085 as `write_promotion_contract`, with promotion preflight status
`promotable`.

Promotion is honest because:

- `PM-UI-LAB-003` is completed with runtime-proven closeout evidence.
- `WR-084` is completed and archived as the app-hosted Editor Lab shell
  prerequisite.
- `PM-UI-LAB-004` now has an accepted operation-driven visual authoring design.
- `WR-085` has disjoint write scopes and explicit non-goals.
- No implementation code is required before promotion.

Use this evidence string for promotion:

```text
Accepted PM-UI-LAB-004 operation-driven visual authoring design plus PM-003 runtime-proven shell closeout clear WR-085 for current-candidate planning; dependencies WR-004 and WR-046 are support-only, and WR-083/WR-084 are completed prerequisite evidence.
```

Suggested command after this contract validates:

```text
task roadmap:promote -- --id WR-085 --state current_candidate --evidence "Accepted PM-UI-LAB-004 operation-driven visual authoring design plus PM-003 runtime-proven shell closeout clear WR-085 for current-candidate planning; dependencies WR-004 and WR-046 are support-only, and WR-083/WR-084 are completed prerequisite evidence."
```

Do not run product-code implementation before the promotion and subsequent
`task ai:goal` rerun select a legal implementation action.

## Architecture Decisions

Source-truth decisions:

- `domain/ui/ui_definition` owns generic visual layout operation mechanics,
  deterministic layout diffs, source-mapped UI layout diagnostics, and
  target-profile compatibility checks.
- `domain/editor/editor_definition` owns editor-definition-specific operation
  vocabulary and validation for theme, workspace layout, menu, shortcut,
  command binding, panel registry, and tool surface registry documents.
- `domain/editor/editor_shell` owns app-neutral Editor Lab operation review,
  history, selection, and diagnostics view-model contracts plus retained UI
  composition support.
- `apps/runenwerk_editor` owns operation dispatch over app draft state,
  interaction-to-operation translation, operation history stacks, undo/redo,
  retained preview refresh, console feedback, and runtime evidence capture.

Forbidden shortcuts:

- moving app-owned draft mutation, history, project IO, runtime activation, or
  preview refresh into `domain/ui/ui_definition`;
- treating retained previews, provider frames, or screenshots as source truth;
- reusing scene-runtime undo/redo as authoritative Editor Lab definition
  history without a new accepted design or ADR;
- accepting operations that cannot produce deterministic diffs into history;
- implementing PM-005 project IO, PM-006 evidence matrix, PM-007 public API
  closeout, game-runtime UI projection, or no-gap audit work under WR-085.

## Implementation Scope

### Operation Contracts

Add typed operation contracts without creating a global command bus:

- `EditorLabOperation` envelope with operation id, target document id,
  document kind, expected schema or revision token, target profile, source
  interaction provenance, operation kind, preview-only flag, and optional
  source location.
- `EditorLabOperationKind` variants for generic UI visual layout operations,
  UI authored value edits, editor theme edits, workspace layout edits, menu
  edits, shortcut edits, command binding edits, and registry edits.
- `EditorLabOperationReport` with status, updated draft when accepted, diff,
  diagnostics, selection update, retained preview summary, history entry id,
  and undo/redo availability.
- `EditorLabOperationDiff` wrapping `UiVisualLayoutDiff` for generic UI layout
  and editor-definition-specific diff entries for editor-owned document edits.

The names may evolve during implementation if nearby module naming makes a
more precise ownership boundary obvious, but the contracts must preserve the
same responsibilities.

### Domain Reducers

Use existing `ui_definition` visual layout application for generic UI layout
operations. Add editor-definition-specific reducers or validation helpers under
responsibility-oriented modules in `domain/editor/editor_definition/src`, for
example an `operation` module if the subsystem grows beyond one cohesive file.

The editor-definition reducers must cover at least the PM-003-supported edit
families:

- UI authored text/value edit for selected editable UI nodes;
- theme token edit;
- workspace layout tab insertion, tab close, split, and active tab changes;
- command/menu/shortcut/binding edit validation where the shell exposes the
  operation.

If a reducer cannot produce a deterministic diff, it must return preview-only
or rejected status instead of entering history.

### App Dispatch And History

Refactor `apps/runenwerk_editor/src/shell/self_authoring.rs` through a
Strangler migration:

1. keep existing direct mutation methods private or narrow while operation
   parity is introduced;
2. add app-owned operation dispatcher over `SelfAuthoringWorkspaceState`;
3. store session-local operation history with before/after document state or
   deterministic inverse operations;
4. expose undo/redo commands and history summaries to the Editor Lab view
   models;
5. preserve draft state on rejected operations;
6. refresh retained preview or inspector state after accepted, undo, and redo
   paths.

Persisted operation logs, project package reload, and cross-session recovery
remain PM-005.

### Shell And Provider Integration

PM-003 direct controls stay as the user-facing shell. WR-085 changes their
normal dispatch path:

- provider-local direct controls build operation intents;
- text-input mapping updates payloads but does not mutate app state;
- `dispatch_shell_command.rs` routes operation commands into app-owned
  dispatch;
- review, diagnostics, inspector, and console surfaces show operation reports,
  deterministic diffs, history entries, and undo/redo availability.

`domain/editor/editor_shell/src/surfaces/editor_definition.rs` may gain
operation review and history view models, but it must not own app mutation or
history storage.

## Implementation Steps

1. Add operation, report, diff, diagnostic, and history view-model contracts at
   the owning boundaries.
2. Reuse `ui_definition` visual layout operations for generic layout edits and
   add editor-definition-specific operation reducers for editor-owned document
   edits.
3. Add an app-owned operation dispatcher and session-local history stack in the
   self-authoring workspace state.
4. Route PM-003 direct controls through operations while retaining explicit
   degraded and disabled states.
5. Add undo/redo shell commands and show their availability in the Editor Lab
   review or inspector surfaces.
6. Add runtime-proof tests and artifact writing for accepted, rejected, undo,
   redo, and retained preview refresh paths.
7. Close PM-004 only after runtime evidence, validation, roadmap/production
   completion metadata, and closeout docs prove the whole PM-004 acceptance
   surface.

## Runtime Evidence Contract

WR-085 closeout must include app-hosted runtime evidence proving:

- the Editor Design profile opens through the PM-003 app-hosted shell path;
- a hierarchy, canvas, or inspector edit creates an `EditorLabOperation`;
- a generic UI visual layout edit uses the `ui_definition` visual layout
  operation path;
- an editor-specific edit uses editor-owned operation contracts or app-owned
  reducer adapters, not `ui_definition`;
- accepted operations produce deterministic diffs;
- rejected operations preserve draft state and surface typed diagnostics;
- accepted operations enter operation history and rejected operations do not;
- undo and redo update draft state and retained preview or structured
  inspector state;
- operation review, diagnostics, history, undo, and redo states have
  screenshots or equivalent retained visual artifacts.

The broader scenario matrix, accessibility checks, performance evidence, and
visual-diff infrastructure remain PM-006.

## Acceptance Criteria

- PM-003 direct controls route supported edits through typed operations rather
  than direct draft mutation.
- Generic UI layout edits reuse `domain/ui/ui_definition/src/visual_layout`
  operation contracts.
- Editor-specific workspace, theme, menu, shortcut, binding, panel, and surface
  edits stay in editor-owned or app-owned boundaries.
- Deterministic operation diffs are visible in Editor Lab review surfaces.
- Rejected operations fail closed with typed diagnostics and preserve draft
  state.
- Operation history records accepted edits and supports undo/redo with preview
  or inspector refresh.
- No PM-005 persistence, full apply/rollback, or project IO behavior is
  claimed by WR-085.

## Validation

Minimum validation before WR-085 closeout:

```text
cargo fmt
cargo test -p ui_definition visual_layout
cargo test -p editor_definition operation
cargo test -p editor_shell editor_lab
cargo test -p runenwerk_editor editor_lab_operation
cargo test -p runenwerk_editor pm_ui_lab_004
task docs:validate
task puml:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

Focused tests should prove:

- deterministic visual layout operation diffs remain stable across identical
  edits;
- editor-specific operations reject invalid document kinds, missing targets,
  duplicate ids, unsupported target profiles, and invalid command references;
- app operation dispatch mutates drafts only through accepted operation
  reports;
- undo/redo restores prior and later draft state deterministically;
- retained preview or inspector state changes after accepted, undo, and redo
  paths;
- project IO and persisted operation logs are not introduced under WR-085.

## Closeout Requirements

- Create
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md`
  only after runtime-proof artifacts and focused tests pass.
- Store PM-004 runtime artifacts under the same closeout directory, including
  screenshots or equivalent retained visual artifacts for operation review,
  rejected diagnostics, undo, and redo states.
- Add the closeout path to WR-085 write scopes only when the closeout file
  exists.
- Move WR-085 to `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
  and mark PM-UI-LAB-004 completed only after the evidence supports
  `runtime_proven`.
- Run roadmap, production, docs, planning, and PUML gates after evidence edits
  and after moving WR-085 out of active roadmap items.
- Rerun `task ai:goal -- --track PT-UI-LAB --scope non-deferred` after
  closeout and follow the next milestone action instead of starting PM-005 from
  memory.

## Perfectionist Closeout Audit

WR-085 may close PM-004 as `runtime_proven` if operation routing, diffs,
history, undo/redo, diagnostics, retained preview refresh, and runtime evidence
are proven end to end and all known gaps are explicit. It must not claim
`perfectionist_verified`; the final no-gap audit belongs to
`PT-UI-LAB-PERFECTION` or an equivalent later track after PM-UI-LAB-007.

Known quality gaps that must remain visible if WR-085 closes successfully:

- PM-005 still owns project IO, package save/load/import/export, migration,
  reload, full diff/apply/reject, failed activation preservation, and rollback.
- PM-006 still owns screenshot matrices, visual diffing, accessibility,
  performance, and degraded-provider scenario breadth.
- PM-007 still owns public API ergonomics, examples, usage docs, and final
  runtime-proven track closeout.
- A later perfectionist audit must re-check module structure, UI ergonomics,
  operation breadth, runtime evidence depth, and no-gap completion claims.

## Non-Goals

- No project package save/load/import/export/reload or persisted operation log.
- No complete DefinitionApplyReview, failed activation preservation, or
  rollback productization.
- No preview scenario matrix, accessibility checks, performance evidence, or
  visual-diff infrastructure.
- No public API usage guides, examples, public API review, or final track
  closeout.
- No game-runtime UI projection execution.
- No global operation bus, global mutable registry, or universal command
  executor.
- No ownership move from app providers into `domain/ui/ui_definition`.

## Stop Conditions

Stop implementation if:

- a supported edit can only be implemented by moving editor/app behavior into
  `domain/ui/ui_definition`;
- retained previews or provider frames are treated as source truth;
- accepted operations cannot produce deterministic diffs;
- undo/redo cannot restore draft state deterministically;
- a PM-005 through PM-007 feature becomes necessary to claim PM-004 complete;
- scene-runtime history must become the generic Editor Lab history authority
  without an accepted design or ADR;
- an ownership or dependency-direction change needs an ADR or accepted design
  update before code can continue.
