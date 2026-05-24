---
title: WR-086 UI Lab Project IO Diff Apply And Rollback Productization Contract
description: Promotion and implementation-readiness contract for PM-UI-LAB-005 app-owned Editor Lab project IO, diff/apply review, activation reports, failed activation preservation, and rollback.
status: active
owner: editor
layer: app/editor-definition
canonical: true
last_reviewed: 2026-05-24
related:
  - ../../../design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md
  - ../../../design/accepted/ui-lab-operation-driven-visual-authoring-design.md
  - ../../../design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md
  - ../../../design/active/ui-lab-productization-design.md
  - ../../../reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-086 UI Lab Project IO Diff Apply And Rollback Productization Contract

## Goal

Implement `PM-UI-LAB-005` by turning Editor Lab drafts into an app-owned,
recoverable project workflow: package save/load/import/export, migration
preflight, deterministic diff review, apply/reject, typed activation reports,
failed activation preservation, rollback, and reload-last-applied behavior.

This contract covers promotion readiness and the future bounded implementation
slice only. It must not implement product code until WR-086 is promoted by the
roadmap workflow and `task ai:goal -- --track PT-UI-LAB --scope non-deferred`
selects an implementation action.

## Source Of Truth

- Production milestone: `PM-UI-LAB-005` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-086` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted PM005 design:
  `docs-site/src/content/docs/design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md`.
- Active productization design:
  `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`.
- Generic UI persistence design:
  `docs-site/src/content/docs/design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md`.
- Completed PM004 closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md`.

Current implementation sources to inspect before product code changes:

- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/applied_editor_definition/activation.rs`
- `apps/runenwerk_editor/src/editor_app/state.rs`
- `apps/runenwerk_editor/src/runtime/resources.rs`
- `domain/editor/editor_definition/src/document.rs`
- `domain/editor/editor_definition/src/operation.rs`
- `domain/editor/editor_definition/src/validate.rs`
- `domain/ui/ui_definition/src/persistence_activation/mod.rs`

## Promotion Readiness

`task production:plan -- --milestone PM-UI-LAB-005 --roadmap WR-086` reports
WR-086 as `write_promotion_contract`, with promotion preflight status
`promotable`.

Promotion is honest because:

- `PM-UI-LAB-004` is completed with runtime-proven operation-driven authoring
  closeout evidence.
- `WR-085` is completed and archived as the operation-driven authoring
  prerequisite.
- `PM-UI-LAB-005` has an accepted persistence/project-IO/apply/rollback
  design.
- `WR-086` has bounded write scopes and explicit non-goals.
- No product code is required before promotion.

Use this evidence string for promotion:

```text
Accepted PM-UI-LAB-005 persistence/project-IO/apply/rollback design plus completed PM-004 runtime-proven operation authoring closeout clear WR-086 for current-candidate planning; WR-085 is completed prerequisite evidence and PM005 ownership remains app/editor-owned.
```

Suggested command after this contract validates:

```text
task roadmap:promote -- --id WR-086 --state current_candidate --evidence "Accepted PM-UI-LAB-005 persistence/project-IO/apply/rollback design plus completed PM-004 runtime-proven operation authoring closeout clear WR-086 for current-candidate planning; WR-085 is completed prerequisite evidence and PM005 ownership remains app/editor-owned."
```

Do not run product-code implementation before promotion and a subsequent
`task ai:goal` rerun select a legal implementation action.

## Architecture Decisions

Source-truth decisions:

- Editor Lab draft documents are the saved package source truth.
- App-owned applied snapshots are the rollback and reload-last-applied source
  truth for editor runtime recovery.
- `domain/ui/ui_definition` owns generic persistence, migration, diff, and
  activation-preflight descriptors only. It does not own project files, app
  activation, provider sessions, rollback, or runtime recovery.
- `domain/editor/editor_definition` may own runtime-neutral editor package,
  apply-review, diff, and diagnostic contracts when they contain no concrete
  file paths, provider handles, runtime resources, or app execution.
- `apps/runenwerk_editor` owns concrete `EditorLabDocumentStore` behavior,
  filesystem/project-session errors, activation attempts, failed activation
  preservation, rollback records, reload-last-applied behavior, command
  dispatch, provider rendering, and runtime evidence.
- Console lines, retained previews, status rows, and activation enum variants
  are derived evidence or projections. They are not source truth.

Forbidden shortcuts:

- moving project IO, activation execution, failed activation preservation, or
  rollback into `domain/ui/ui_definition`;
- treating `EditorDefinitionExportPackage` single-document export as the full
  project store;
- applying directly from a draft without a deterministic `DefinitionApplyReview`;
- reporting activation through unstructured console strings only;
- losing failed activation input or previous live state on activation failure;
- claiming PM006 screenshot/accessibility/performance breadth or PM007 public
  API/docs/examples closeout under WR-086;
- using a broad catch-all module such as `utils.rs` or `helpers.rs`.

## Implementation Scope

### Project Package And Document Store

Introduce an app-owned Editor Lab project package and document-store boundary.
The implementation may place reusable runtime-neutral DTOs in
`domain/editor/editor_definition/src` only when they do not depend on app IO.

Suggested app module shape, subject to nearby code inspection:

```text
apps/runenwerk_editor/src/shell/editor_lab_project/
|-- mod.rs
|-- package.rs
|-- store.rs
|-- migration.rs
|-- diagnostic.rs
`-- review.rs
```

The store must support:

- package creation from `SelfAuthoringWorkspaceState`;
- deterministic package serialization;
- package reload into drafts without live activation;
- selected-definition and full-package import/export;
- migration preflight over package contents;
- invalid package preservation with typed diagnostics;
- last saved package and last applied snapshot access.

If a module location under `shell` proves wrong during implementation, choose
the owning app/editor subsystem explicitly and update this contract or closeout
with the final location.

### DefinitionApplyReview

Create a typed `DefinitionApplyReview` path before live activation. The review
must contain:

- selected document id and kind;
- draft snapshot, last applied snapshot, and proposed applied snapshot;
- deterministic diff rows and optional textual diff;
- migration and validation diagnostics;
- operation reports that contributed to the proposal when available;
- user decision state: pending, rejected, accepted, blocked;
- activation preflight status;
- rollback metadata for the prior applied snapshot.

Review construction must be deterministic. If a diff cannot be produced, apply
is blocked and the review remains preview-only.

### Activation Reports And Failed Activation Preservation

Replace queue-only activation evidence with typed app-owned activation reports.
The implementation should preserve the existing activation enum as an adapter
if it remains useful, but the PM005 product evidence must include structured
reports for:

- queued;
- applied;
- rejected or validation-blocked;
- failed with previous state preserved;
- no-live-activation;
- degraded-provider or installer-blocked state where the current runtime can
  express it.

`apps/runenwerk_editor/src/runtime/resources.rs` may continue to append console
lines, but it must also return or record typed activation report entries that
the Editor Lab can show and closeout evidence can assert.

### Rollback And Reload Last Applied

Rollback must be app-owned and snapshot-backed:

- applying a review records the prior applied snapshot before mutating applied
  state;
- rollback fails closed when no prior applied snapshot exists;
- rollback restores the previous applied snapshot and records diagnostics;
- reload-last-applied reloads the last applied snapshot from the package/session
  where the current runtime supports it;
- failed activation preserves the failed input and leaves prior active state
  available for rollback or inspection.

### Editor Lab UI And Commands

Wire normal Editor Lab affordances through typed commands and provider
controls:

- save package;
- load or reload package;
- import selected definition;
- export selected definition;
- build apply review;
- reject apply review;
- apply review;
- inspect activation report;
- rollback selected applied definition;
- reload last applied.

The provider UI must expose typed diagnostics and review state. Debug action
lists, status-only panels, and raw console strings are not sufficient product
UI for PM005 acceptance.

## Implementation Steps

1. Re-inspect current PM004 state and existing export/import/apply/rollback
   methods before editing.
2. Add runtime-neutral package/review DTOs in `editor_definition` only where
   ownership is clean; keep concrete project IO in `apps/runenwerk_editor`.
3. Introduce app-owned document-store/session state for Editor Lab packages.
4. Convert selected-document export/import into package-aware store operations.
5. Add migration preflight and deterministic diff construction for draft versus
   applied documents.
6. Replace direct apply with `DefinitionApplyReview` construction, reject, and
   accept paths.
7. Record typed activation reports for success, validation-blocked, installer
   blocked, no-live-activation, failed, and prior-state-preserved outcomes.
8. Add rollback records and reload-last-applied behavior.
9. Wire provider controls and command dispatch to the store, review, apply,
   report, and rollback flows.
10. Add focused tests and runtime-proof artifact generation.
11. Update roadmap, production, and closeout evidence only after runtime proof
   exists.

If this scope cannot stay bounded without mixing PM006 evidence breadth or
PM007 public API closeout into the implementation, stop and split WR-086 into
project-store and apply-review/rollback WR rows before product code continues.

## Runtime Evidence Contract

WR-086 closeout must include app-hosted runtime evidence proving:

- an Editor Lab package is saved from current drafts;
- the saved package reloads into drafts without changing live state;
- selected definition import/export and full package import/export produce
  typed results;
- invalid package or migration input is preserved with diagnostics;
- draft versus applied state produces deterministic diff rows;
- rejected apply preserves draft and applied state;
- accepted apply records a `DefinitionApplyReview`, queues activation, executes
  activation, and records a typed activation report;
- failed activation preserves failed input and previous live/apply state;
- rollback restores the previous applied snapshot and records diagnostics;
- reload-last-applied restores the last applied state where the current runtime
  supports it;
- Editor Lab surfaces expose review, diagnostics, activation report, and
  rollback state without relying on raw debug action lists.

Screenshots, visual-diff matrices, accessibility checks, performance evidence,
and broad degraded-provider scenario coverage remain PM006 unless the PM005
implementation naturally produces narrow equivalent artifacts.

## Acceptance Criteria

- `EditorLabDocumentStore` is app-owned and handles package save/load/import,
  export, migration preflight, invalid-input preservation, and reload.
- `DefinitionApplyReview` is the only normal path from changed draft to apply.
- Apply/reject paths preserve source truth and expose typed diagnostics.
- Activation reports are structured and app-owned, not console-only.
- Failed activation preserves the failed input and previous active/applied
  state.
- Rollback is snapshot-backed, typed, and fails closed when unavailable.
- Runtime evidence proves save, reload, import, export, migration, diff, apply,
  reject, failed activation preservation, rollback, and reload-last-applied
  behavior.
- `ui_definition` remains behavior-free and does not gain project IO or editor
  activation behavior.

## Validation

Minimum validation before WR-086 closeout:

```text
cargo fmt
cargo test -p ui_definition persistence_activation
cargo test -p editor_definition
cargo test -p runenwerk_editor editor_lab
cargo test -p runenwerk_editor editor_definition_activation
cargo test -p runenwerk_editor pm_ui_lab_005
$env:RUNENWERK_WRITE_PM_UI_LAB_005_EVIDENCE='1'; cargo test -p runenwerk_editor pm_ui_lab_005_runtime_evidence_reports_project_io_apply_rollback -- --nocapture
task docs:validate
task puml:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
git diff --check
```

Focused tests should prove:

- deterministic package serialization and reload round-trip;
- selected and full-package import/export;
- unsupported package version and invalid schema diagnostics;
- migration dry-run diagnostics before activation;
- deterministic draft-versus-applied diff rows;
- rejected apply preserving draft and applied state;
- successful apply producing activation report and applied snapshot;
- failed activation preserving failed input and previous active state;
- rollback requiring a prior applied snapshot;
- rollback restoring prior applied state;
- reload-last-applied behavior where supported;
- command/provider UI exposes typed diagnostics.

## Closeout Requirements

- Create
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-005-persistence-project-io-diff-apply-and-rollback/closeout.md`
  only after runtime-proof artifacts and focused tests pass.
- Store PM005 runtime artifacts under that closeout directory, including
  package round-trip, diff/apply review, activation report, failure
  preservation, rollback, and reload-last-applied evidence.
- Add the closeout path and artifact paths to WR-086 write scopes only when
  the files exist.
- Move WR-086 to `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
  and mark PM-UI-LAB-005 completed only after evidence supports
  `runtime_proven`.
- Record PM006 and PM007 as remaining known gaps, not PM005 failures, if PM005
  closes its runtime contract.
- Run roadmap, production, docs, planning, and PUML gates after evidence edits
  and after moving WR-086 out of active roadmap items.
- Rerun `task ai:goal -- --track PT-UI-LAB --scope non-deferred` after
  closeout and follow the next milestone action instead of starting PM006 from
  memory.

## Perfectionist Closeout Audit

WR-086 may close PM005 as `runtime_proven` if project IO, diff/apply, activation
reports, failed activation preservation, rollback, reload-last-applied, and
runtime evidence are proven end to end. It must not claim
`perfectionist_verified`; the final no-gap audit belongs to
`PT-UI-LAB-PERFECTION` or an equivalent later track after PM-UI-LAB-007.

Known quality gaps that must remain visible if WR-086 closes successfully:

- PM006 still owns screenshot matrices, visual diffing, accessibility,
  performance, and broad degraded-provider scenario evidence.
- PM007 still owns public API ergonomics, examples, usage docs, and final
  runtime-proven track closeout.
- Game-runtime UI projection execution remains out of Editor Lab V1 scope.
- A later perfectionist audit must re-check module structure, UI ergonomics,
  persistence recovery depth, runtime evidence breadth, and no-gap completion
  claims.

## Non-Goals

- No PM006 screenshot matrix, visual-diff infrastructure, accessibility check,
  performance evidence, or broad scenario catalog.
- No PM007 public API review, usage guide, examples, or final track closeout.
- No game-runtime UI projection execution.
- No moving app project IO, activation execution, rollback, or provider
  sessions into `domain/ui/ui_definition`.
- No treating retained previews, descriptors, status rows, or console lines as
  source truth.
- No no-gap or `perfectionist_verified` claim.

## Stop Conditions

Stop implementation if:

- PM005 requires `ui_definition` to execute app, editor, project IO, or runtime
  behavior;
- deterministic package serialization or diff output cannot be produced;
- activation report evidence would be console-only;
- failed activation would drop failed input or previous active state;
- rollback cannot be snapshot-backed;
- the work cannot remain bounded without pulling in PM006 or PM007 scope;
- the implementation needs a durable ownership or dependency-direction change
  not covered by the accepted design or existing ADRs.
