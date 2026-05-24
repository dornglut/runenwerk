---
title: WR-084 UI Lab App-Hosted Editor Lab Surface Shell Contract
description: Design-first implementation contract for PM-UI-LAB-003 app-hosted Editor Lab direct-control shell.
status: active
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-24
related:
  - ../../../design/accepted/ui-lab-app-hosted-editor-lab-surface-shell-design.md
  - ../../../design/accepted/ui-lab-command-catalog-and-surface-registry-design.md
  - ../../../design/active/ui-lab-productization-design.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-084 UI Lab App-Hosted Editor Lab Surface Shell Contract

## Goal

Implement `PM-UI-LAB-003` by replacing the normal Editor Design
self-authoring line/action panels with a real app-hosted Editor Lab shell:
definition hierarchy, palette, canvas or retained preview, inspector,
command/apply review, diagnostics, and preview-console feedback.

This contract covers only the shell product surface and direct-control routing.
It must not implement PM-004 operation history, PM-005 project IO and complete
diff/apply/rollback, PM-006 evidence matrix, PM-007 public API closeout, or any
game-runtime UI projection execution.

## Source Of Truth

- Production milestone: `PM-UI-LAB-003` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-084` in the roadmap source set.
- Accepted shell design:
  `docs-site/src/content/docs/design/accepted/ui-lab-app-hosted-editor-lab-surface-shell-design.md`.
- Productization design:
  `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`.
- Completed prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-002-registry-and-command-source-of-truth/closeout.md`.

Current implementation sources to inspect before code changes:

- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/tool_suites/editor_design_tool_suite.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs`
- `domain/editor/editor_shell/src/composition/build_self_authoring_control_panel.rs`
- `domain/editor/editor_shell/src/commands/map_interactions.rs`
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs`
- `domain/editor/editor_shell/src/workspace/profile.rs`
- `domain/editor/editor_definition`
- `domain/ui/ui_definition`

## Readiness

Initial `task production:plan -- --milestone PM-UI-LAB-003 --roadmap WR-084`
classified WR-084 as `repair_wr_promotion_metadata` because the roadmap row
referenced two missing write-scope paths.

The exact metadata repair is:

- create this implementation contract at
  `docs-site/src/content/docs/reports/implementation-plans/wr-084-ui-lab-app-hosted-editor-lab-surface-shell/plan.md`;
- remove the future PM-003 closeout path from WR-084 write scopes until closeout
  exists.

No product code is authorized by this repair. After this contract and metadata
repair validate, rerun `task ai:goal -- --track PT-UI-LAB --scope non-deferred`
and follow the next legal action. If the next action is promotion planning, use
this contract as the written evidence and do not start implementation until the
roadmap row is current-candidate eligible.

## Architecture Decisions

Source truth decisions:

- App-owned Editor Lab draft state in
  `apps/runenwerk_editor/src/shell/self_authoring.rs` remains the concrete
  source for loaded documents, selection, basic apply previews, snapshots, and
  app console feedback in PM-003.
- Authored UI templates and editor definition documents remain domain-owned
  descriptions. They do not execute behavior.
- `domain/ui/ui_definition` forms retained UI previews and remains
  behavior-free.
- `domain/editor/editor_definition` validates editor definition documents and
  owns reusable document vocabulary.
- `domain/editor/editor_shell` owns app-neutral Editor Lab view-model contracts,
  surface actions, route tables, and composition builders.
- `apps/runenwerk_editor` owns provider behavior, app command dispatch, runtime
  activation queues, console output, and evidence capture.

Complete shell chain:

1. Registry-backed Editor Design surfaces select the target provider through the
   PM-002 surface metadata path.
2. The app provider reads self-authoring state, command catalog data, registry
   metadata, and editor-definition diagnostics.
3. The provider builds typed Editor Lab view models for hierarchy, palette,
   preview, inspector, review, diagnostics, and console surfaces.
4. Editor-shell composition builders turn those view models into retained UI
   controls with stable widget ids and route tables.
5. Direct controls emit `SurfaceLocalAction::EditorDefinition` actions.
6. The provider maps local actions to app-owned `ShellCommand` values.
7. `dispatch_shell_command.rs` mutates app state and records typed console or
   diagnostic feedback.
8. A runtime evidence harness proves the direct-control path in the app-hosted
   shell.

Forbidden shortcuts:

- using `build_self_authoring_control_panel.rs` as the normal path for supported
  Editor Lab surfaces;
- shipping hard-coded demo edits such as `Edited in self-authoring` or
  `#5f8cff` as normal workflows;
- moving provider behavior, command execution, project IO, runtime activation,
  or console state into `ui_definition`;
- claiming PM-003 runtime proof from descriptor data, static fixtures, or text
  status panels alone;
- implementing PM-004, PM-005, PM-006, or PM-007 scope under WR-084.

## Implementation Scope

### Editor Lab Surface Contracts

Extend `domain/editor/editor_shell/src/surfaces/editor_definition.rs` with
app-neutral Editor Lab view models and actions. Keep the names responsibility
oriented and avoid catch-all abstractions.

Expected view-model families:

- `EditorLabDefinitionHierarchyViewModel`
- `EditorLabPaletteViewModel`
- `EditorLabCanvasPreviewViewModel`
- `EditorLabInspectorViewModel`
- `EditorLabReviewViewModel`
- `EditorLabDiagnosticsViewModel`
- `EditorLabConsoleViewModel`

The contracts should carry stable row or field ids, labels, selected state,
disabled reasons, diagnostics, routeable actions, source paths where available,
and degraded-state descriptors.

Text-editing controls for rename, UI node text, theme token values, and
workspace draft labels must route through typed `EditorDefinitionSurfaceAction`
values. If retained text-input interactions need to rewrite the payload with
the latest user input, `domain/editor/editor_shell/src/commands/map_interactions.rs`
is in scope for the narrow typed mapping only. That file must not gain
app-owned behavior or document mutation logic.

### Editor Lab Composition

Add responsibility-oriented composition modules under
`domain/editor/editor_shell/src/composition`, for example an `editor_lab`
submodule with hierarchy, palette, canvas, inspector, review, diagnostics, and
console builders.

The builders must:

- use existing retained UI widgets and interaction routing;
- create deterministic widget ids through existing surface scoping helpers;
- return route tables for direct controls;
- avoid nested generic action panels for supported states;
- expose explicit degraded-state UI when the selected document cannot preview
  or a provider path is unavailable.

### App Provider Translation

Refactor `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` through
a Strangler migration:

1. keep current behavior available as a fallback;
2. add view-model builders from `SelfAuthoringWorkspaceState`;
3. switch each supported Editor Design surface to typed composition;
4. keep provider-local action to `ShellCommand` mapping app-owned;
5. add source or behavior guards that fail if supported surfaces regress to the
   generic line/action panel.

`apps/runenwerk_editor/src/shell/self_authoring.rs` may gain narrow read-model
helpers for selected document state, palette candidates, inspector field data,
diagnostics, and recent apply/console facts. It must not become project IO or
operation-history infrastructure.

### Tool Suite And Workspace Profile

Use the PM-002 registry-backed surface metadata path. Update
`apps/runenwerk_editor/src/shell/tool_suites/editor_design_tool_suite.rs` or
`domain/editor/editor_shell/src/workspace/profile.rs` only where needed to make
the Editor Design profile mount the required shell surfaces coherently.

Do not add a parallel surface identity source. Stable keys and registry
metadata remain the normal path; legacy enum mapping remains compatibility.

## Implementation Steps

1. Add app-neutral Editor Lab surface view models and narrow typed actions in
   `domain/editor/editor_shell/src/surfaces/editor_definition.rs`.
2. Add responsibility-oriented Editor Lab composition builders under
   `domain/editor/editor_shell/src/composition` and export only the normal
   builder entry points needed by app providers.
3. Extend retained interaction mapping only where direct text controls need to
   transform `UiInteraction::TextInput` events into typed
   `SurfaceLocalAction::EditorDefinition` payloads.
4. Translate `SelfAuthoringWorkspaceState` into typed hierarchy, palette,
   preview, inspector, review, diagnostics, console, and degraded-state view
   models in `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`.
5. Switch supported Editor Design surfaces to the typed builders while keeping
   the previous line/action panel as a fallback for unsupported compatibility
   cases only.
6. Add focused architecture and runtime-proof tests, then capture closeout
   artifacts in the PM-003 closeout folder.
7. Update roadmap and production evidence only after runtime evidence and
   validations prove the app-hosted shell path.

## Runtime Evidence Contract

WR-084 closeout must include app-hosted runtime evidence proving:

- the Editor Design profile opens with typed Editor Lab surface frames;
- definition hierarchy selection uses direct controls;
- UI hierarchy selection uses direct controls;
- at least one supported edit updates app-owned draft state through the shell
  command boundary;
- retained preview or inspector state changes after the edit;
- diagnostics and command/apply review surfaces present structured data;
- non-previewable or provider-degraded state is explicit and typed;
- preview-console or app-owned console feedback is visible;
- evidence artifacts are stored under
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-003-app-hosted-editor-lab-surface-shell/`.

Because this is a visible product shell milestone, the closeout needs
screenshots or equivalent visual artifacts for the supported states it claims.
The broader scenario matrix, accessibility, performance, and visual-diff suite
remain PM-006.

## Acceptance Criteria

- Normal Editor Design hierarchy, palette, inspector, review, diagnostics, and
  preview-console surfaces are built from typed Editor Lab view models rather
  than generic self-authoring line/action lists.
- `UiCanvas` continues to use deterministic retained preview output for
  previewable UI documents, and non-previewable documents render explicit typed
  degraded state.
- Direct controls route through `SurfaceLocalAction::EditorDefinition`, then
  through app-owned `ShellCommand` dispatch, before mutating
  `SelfAuthoringWorkspaceState`.
- At least one supported edit proves draft-state mutation and a changed
  retained preview or inspector state after dispatch.
- Diagnostics, disabled reasons, provider-degraded state, and preview-console
  feedback are visible as structured surface data.
- Hard-coded demo edits and status-panel-only fallbacks are absent from normal
  supported workflows.
- No provider behavior, project IO, operation history, activation execution, or
  domain mutation authority moves into `domain/ui/ui_definition` or
  `domain/editor/editor_shell`.

## Validation

Minimum validation before WR-084 closeout:

```text
cargo fmt
cargo test -p editor_shell editor_lab
cargo test -p runenwerk_editor editor_lab
cargo test -p runenwerk_editor pm_ui_lab_003
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

- each Editor Lab composition builder has deterministic routes and widget ids;
- every supported Editor Design surface builds a typed surface, not the generic
  self-authoring action panel;
- direct controls route through `SurfaceLocalAction::EditorDefinition` and app
  `ShellCommand` dispatch;
- degraded non-previewable documents and provider failures render explicit
  diagnostics;
- hard-coded demo edit values are absent from normal workflows;
- runtime proof artifact generation covers the app-hosted shell chain.

## Closeout Requirements

- Create
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-003-app-hosted-editor-lab-surface-shell/closeout.md`
  only after the runtime-proof artifact exists and focused tests pass.
- Store PM-003 runtime artifacts under the same closeout directory, including
  screenshots or equivalent retained visual artifacts for every supported state
  claimed by the closeout.
- Update `docs-site/src/content/docs/workspace/roadmap-items.yaml`,
  `docs-site/src/content/docs/workspace/roadmap-archive.yaml`, and
  `docs-site/src/content/docs/workspace/production-tracks.yaml` so WR-084 and
  PM-UI-LAB-003 claim `runtime_proven` only when the evidence supports it.
- Run the roadmap, production, docs, planning, and PUML gates after evidence
  edits and after moving WR-084 out of active roadmap items.
- Rerun `task ai:goal -- --track PT-UI-LAB --scope non-deferred` after closeout
  and follow the next milestone action instead of starting PM-004 from memory.

## Perfectionist Closeout Audit

WR-084 may close PM-003 as `runtime_proven` if the app-hosted shell chain is
proved end to end and all known gaps are explicit. It must not claim
`perfectionist_verified`; the final no-gap audit belongs to
`PT-UI-LAB-PERFECTION` or an equivalent later track after PM-UI-LAB-007.

Known quality gaps that must remain visible if WR-084 closes successfully:

- PM-004 still owns operation history, deterministic visual operations, and
  undo/redo productization.
- PM-005 still owns project IO, user-facing full diff/apply/rollback, migration,
  reload, and failed activation preservation.
- PM-006 still owns screenshot matrices, visual diffing, accessibility,
  performance, and degraded-provider scenario breadth.
- PM-007 still owns public API ergonomics, examples, usage docs, and final
  runtime-proven track closeout.
- A later perfectionist audit must re-check module structure, UI ergonomics,
  runtime evidence depth, and no-gap completion claims across the full track.

## Promotion Evidence

Use this evidence when promoting WR-084 after validation:

```text
Accepted PM-UI-LAB-003 shell design plus WR-084 implementation contract clear the app-hosted Editor Lab shell design gate; dependencies WR-004 and WR-046 are support-only and WR-083 is completed runtime-proven prerequisite evidence.
```

If `task production:plan -- --milestone PM-UI-LAB-003 --roadmap WR-084` reports
promotion metadata blockers after this contract exists, repair only those exact
metadata issues and rerun the planning gates.

## Non-Goals

- No operation-driven visual authoring history or deterministic operation diff
  completion; PM-004 owns it.
- No project package load/save/import/export/reload or complete
  diff/apply/rollback productization; PM-005 owns it.
- No preview scenario matrix, accessibility checks, performance evidence, or
  visual-diff infrastructure; PM-006 owns it.
- No public API usage guides, examples, public API review, or final track
  closeout; PM-007 owns it.
- No game-runtime UI projection execution.
- No ownership move from app providers into `domain/ui/ui_definition`.

## Stop Conditions

Stop implementation if:

- a supported surface can only be implemented by moving app behavior into a
  domain crate;
- registry metadata or command catalog data is bypassed by a new side table;
- retained previews or provider frames are treated as source truth;
- runtime evidence cannot prove the launched app-hosted shell path;
- a PM-004 through PM-007 feature becomes necessary to claim PM-003 complete;
- an ownership or dependency-direction change needs an ADR or accepted design
  update before code can continue.
