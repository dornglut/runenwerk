---
title: UI Lab Operation-Driven Visual Authoring Design
description: Accepted design for PM-UI-LAB-004 operation-driven visual authoring, deterministic diffs, history, undo/redo, validation, and diagnostics.
status: accepted
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-24
related_adrs:
  - ../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/accepted/0006-editor-surface-provider-plugin-seam.md
  - ../../adr/accepted/0010-graph-substrate-canvas-boundary.md
  - ../../adr/accepted/0012-capability-workbench-clean-break.md
related_designs:
  - ../active/ui-lab-productization-design.md
  - ./ui-lab-app-hosted-editor-lab-surface-shell-design.md
  - ./ui-lab-command-catalog-and-surface-registry-design.md
  - ./ui-designer-visual-layout-and-interface-composition-design.md
  - ./ui-designer-view-model-capability-and-intent-binding-design.md
  - ./ui-designer-persistence-migration-diff-and-activation-design.md
related_reports:
  - ../../reports/closeouts/pm-ui-lab-003-app-hosted-editor-lab-surface-shell/closeout.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# UI Lab Operation-Driven Visual Authoring Design

## Status

This is the accepted implementation design for `PM-UI-LAB-004`.

It clears the design gate for operation-driven visual authoring only. It does
not authorize product code until a linked WR row is selected, production
planning produces a decision-complete implementation contract, and roadmap
promotion gates pass.

## Goal

Editor Lab direct controls must stop mutating draft definitions through
ad-hoc app methods. Supported canvas, hierarchy, and inspector edits need to
flow through typed operations that produce deterministic diffs, validation
diagnostics, edit history, undo/redo availability, and retained preview
updates.

The product path is:

```text
Editor Lab direct control
  -> app-owned interaction intent
  -> EditorLabOperation envelope
  -> owning domain operation reducer
  -> deterministic operation diff and diagnostics
  -> app-owned history entry
  -> retained preview or structured inspector refresh
  -> runtime evidence artifact
```

This milestone productizes operation-driven authoring inside Editor Lab V1. It
does not implement project package IO, persisted operation logs, complete
diff/apply/rollback, screenshot matrices, public API examples, or no-gap
certification.

## Current Code Truth

`PM-UI-LAB-003` completed the app-hosted shell and direct-control route chain.
The current surfaces can select authored documents, select UI nodes, edit
supported text fields, edit theme colors, change workspace layout tabs, show
diagnostics, show a review surface, and render retained preview or typed
degraded state.

The current mutation path is still command-shaped rather than operation-shaped:

- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` defines
  `EditorDefinitionSurfaceAction` and app-neutral Editor Lab view models, but
  the actions describe direct commands, not reviewable operation envelopes.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` translates
  app-owned `SelfAuthoringWorkspaceState` into PM-003 direct-control surfaces.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` owns draft documents and
  direct mutation methods such as `set_selected_ui_node_text`,
  `set_selected_theme_color`, `add_selected_workspace_layout_tab`, and
  `split_selected_workspace_layout_root`. Those methods do not yet produce an
  operation id, deterministic diff, undo record, or operation-specific
  diagnostics report.
- `domain/ui/ui_definition/src/visual_layout/operation.rs`,
  `apply.rs`, `diff.rs`, and `diagnostic.rs` already provide generic visual
  layout operation contracts, deterministic layout diffs, target-profile
  checks, and source-mapped diagnostics for UI template tree edits.
- `domain/editor/editor_definition` validates editor definition documents, but
  it does not yet expose editor-definition-specific edit operation contracts
  for theme, workspace, menu, shortcut, binding, surface registry, or panel
  registry documents.
- `domain/editor/editor_core/src/history.rs` and
  `apps/runenwerk_editor/src/editor_runtime/history` show existing history
  vocabulary and scene-runtime undo/redo patterns. They are scene transaction
  history, not reusable Editor Lab definition history.

The implementation must use these foundations without making retained
previews, provider output, runtime widget ids, scene history, or project files
the source of truth.

## Architecture Governance Result

Architecture governance was run for this scope:

```text
task ai:architecture-governance -- --task "PM-UI-LAB-004 Operation-Driven Visual Authoring design gate" --scope "docs-site/src/content/docs/design/active/ui-lab-productization-design.md; docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md; docs-site/src/content/docs/reports/closeouts/pm-ui-lab-003-app-hosted-editor-lab-surface-shell/closeout.md; domain/ui/ui_definition/src/visual_layout; domain/editor/editor_definition; domain/editor/editor_shell/src/surfaces/editor_definition.rs; apps/runenwerk_editor/src/shell/self_authoring.rs; apps/runenwerk_editor/src/shell/providers/self_authoring.rs; domain/editor/editor_core/src/history.rs; apps/runenwerk_editor/src/editor_runtime/history"
```

Governance decisions:

- DDD owner: the `editor` bounded context owns Editor Lab authoring,
  operation history, undo/redo, validation presentation, and runtime proof.
- Supporting owner: `domain/ui/ui_definition` owns generic, behavior-free UI
  visual layout operation mechanics and deterministic layout diffs.
- Supporting owner: `domain/editor/editor_definition` owns editor-definition
  document vocabulary, editor-specific operation kinds, operation validation,
  and editor-definition diffs.
- Supporting owner: `domain/editor/editor_shell` owns app-neutral surface
  contracts, operation review view models, and retained composition helpers.
- App owner: `apps/runenwerk_editor` owns operation dispatch over app draft
  state, interaction-to-operation translation, history stacks, undo/redo
  commands, preview refresh, runtime evidence capture, and console feedback.
- Clean Architecture direction: domain crates may define and validate
  operation contracts, but concrete draft mutation, runtime preview refresh,
  app console output, and history persistence remain app-owned unless a later
  design deliberately extracts a reusable editor-definition reducer.
- Source truth invariant: authored definition documents and app-owned draft
  state are source truth. Operations, diffs, retained previews, shell view
  models, and evidence artifacts are derived, reviewable facts.
- Translation boundary: UI layout operations cross into `ui_definition` only
  for generic UI template tree edits. Editor workspace, theme, menu, shortcut,
  binding, surface registry, panel registry, selection, and history behavior
  cross through editor-owned operation contracts or app-owned session commands.
- ADR need: no new ADR is required while PM-004 preserves the existing
  description-versus-execution, derived projection, provider seam, graph
  canvas, and capability workbench decisions. A new ADR or accepted design
  update is required before creating a global editor operation engine, moving
  app draft mutation into `ui_definition`, treating retained previews as
  source truth, or reusing scene-runtime history as a generic definition
  history authority.
- ATAM-lite priority order: source-truth correctness and ownership first,
  deterministic diffs second, undo/redo reliability third, runtime evidence
  fourth, editing breadth fifth. Editing breadth may not bypass operation
  review or ownership boundaries.
- Ownership mode: stream-aligned editor product work with
  complicated-subsystem support from UI-definition, editor-definition, and
  editor-shell owners.

## Operation Boundary

`EditorLabOperation` is the operation facade for Editor Lab V1. It is a typed
envelope, not a generic universal command bus.

Each operation records:

- stable operation id;
- target authored document id;
- target document kind and expected schema version;
- selected target path or editor-definition target id;
- expected stable authored id or document revision token where applicable;
- target profile and surface context;
- source interaction provenance;
- operation kind;
- preview-only flag when deterministic activation is unavailable;
- source location when available.

The first operation families are:

| Family | Owner | Scope |
|---|---|---|
| UI visual layout operation | `domain/ui/ui_definition` | Insert, remove, move, reorder, wrap, unwrap, split ratio, stack axis, and template-reference layout edits over UI templates. |
| UI authored value operation | `domain/editor/editor_definition` or app reducer using UI contracts | Text/value edits for selected authored UI nodes until generic UI value operation contracts exist. |
| Editor theme operation | `domain/editor/editor_definition` | Theme color, spacing, typography, and radius token edits. |
| Editor workspace layout operation | `domain/editor/editor_definition` | Split roots, tab stack edits, active tab selection, tab labels, tool surface references, and host fractions. |
| Editor menu operation | `domain/editor/editor_definition` | Menu item label, command reference, availability reference, order, insert, remove, and nesting edits. |
| Editor shortcut operation | `domain/editor/editor_definition` | Shortcut chord, command reference, context, order, insert, remove, and conflict diagnostics. |
| Editor command binding operation | `domain/editor/editor_definition` | Command binding route target, capability requirements, undoable flag, insert, remove, and duplicate diagnostics. |
| Editor surface or panel registry operation | `domain/editor/editor_definition` | Surface/panel label, provider family, capability, document-kind compatibility, workspace compatibility, and registry insertion/removal. |

Selection is not persisted definition truth by default. The app may represent
document, node, field, and operation selection through `EditorLabSelectionState`
and session-scoped actions so the UI is deterministic and testable, but those
selection changes are not definition-history entries unless a later design
explicitly makes them persisted workspace state.

## Operation Reports And Diffs

Every operation reducer returns an `EditorLabOperationReport` with:

- operation id and target document id;
- accepted, rejected, or preview-only status;
- updated draft document when the operation is accepted;
- deterministic operation diff;
- diagnostics with severity, code, message, path, target profile, source
  operation id, activation impact, and suggested fix;
- selection update when the operation changes the active target;
- retained preview refresh summary when a previewable document changes;
- history entry id and undo/redo availability when accepted.

`EditorLabOperationDiff` wraps existing `UiVisualLayoutDiff` for generic UI
layout edits and adds editor-definition-specific diff entries for theme,
workspace, menu, shortcut, binding, surface, and panel registry edits. Diffs
must be deterministic enough for textual review. If an edit cannot produce a
deterministic diff, it remains preview-only and cannot enter the applied
history stack.

Diff entries include:

- change kind;
- target path or stable target id;
- before value;
- after value;
- owning operation family;
- source document id;
- optional source location.

## History And Undo/Redo

PM-004 history is app-owned Editor Lab definition history, not scene runtime
transaction history and not project persistence.

`EditorLabOperationHistory` stores accepted operation reports and enough
before/after data to undo and redo draft mutations deterministically inside the
current app session. The first implementation may use bounded before/after
document snapshots for reliability, then refine to inverse operations if
needed after evidence exists.

History invariants:

- rejected and preview-only operations do not enter undo history;
- a new accepted operation clears redo history;
- undo restores the previous draft document state and pushes a redo entry;
- redo reapplies the stored after state and pushes an undo entry;
- undo/redo refresh retained preview or inspector data for the selected
  document;
- history entries include operation id, label, document id, document kind,
  diff summary, diagnostic summary, and timestamps only if timestamps are
  deterministic or omitted from review diffs;
- PM-004 history is session-local. Persisted operation logs and reload across
  app restart remain PM-005 unless explicitly narrowed in the implementation
  contract.

## App Provider And Shell Integration

PM-003 direct controls remain the product UI. PM-004 changes what those
controls dispatch:

1. direct controls emit app-neutral surface actions;
2. provider translation builds an `EditorLabOperation` or session selection
   action;
3. app dispatch validates the operation preconditions;
4. the owning domain reducer or app-owned reducer returns an operation report;
5. app state updates draft documents and history only on accepted reports;
6. shell view models show operation diffs, diagnostics, selected target, and
   undo/redo availability;
7. runtime evidence proves retained preview or inspector state after apply,
   undo, and redo.

`domain/editor/editor_shell/src/surfaces/editor_definition.rs` may gain
operation review, history, and selection view models, but it must not own draft
mutation, app history stacks, or runtime preview refresh.

`apps/runenwerk_editor/src/shell/self_authoring.rs` may be refactored from
direct mutation methods into operation reducers or thin reducer adapters. The
normal PM-004 path should be operation-first; old direct methods may remain
private compatibility helpers while the Strangler migration is in progress.

## Visual Authoring Ergonomics

PM-004 must address the authoring friction left after PM-003:

- edits must be reviewable before and after execution through a visible
  operation diff;
- canvas/hierarchy/inspector controls must produce the same operation report
  shape for the same edit;
- disabled operations must show typed reasons before dispatch;
- invalid operations must fail closed with diagnostics and preserve the draft;
- undo/redo must be visible from the Editor Lab shell and must update preview
  or inspector state;
- operation history must be scoped to the selected authored document and
  explain what changed without requiring log spelunking;
- text, theme, workspace, menu, shortcut, and binding edits must not rely on
  hard-coded demo values or ad-hoc status lines.

## Migration Strategy

Use a Strangler migration over the PM-003 direct-control shell:

1. Add operation contracts and operation reports beside the existing
   `EditorDefinitionSurfaceAction` route path.
2. Add an app-owned operation dispatcher and session-local operation history.
3. Route one existing supported edit through operations while keeping the old
   direct method private and covered by tests.
4. Convert UI node text, theme token, workspace layout, menu, shortcut, and
   binding edits to operation reducers.
5. Add operation review/history/undo/redo view-model data to the existing
   review and inspector surfaces.
6. Remove the normal supported direct-mutation route after operation parity is
   proven.

The migration must not rename large app modules just to improve aesthetics.
Split files only when stable responsibilities emerge, for example
`apps/runenwerk_editor/src/shell/self_authoring/operations.rs`,
`history.rs`, and `selection.rs`.

## Runtime Evidence Requirements

PM-004 closeout may claim `runtime_proven` only when evidence proves:

- the app-hosted Editor Lab opens the PM-003 shell path;
- a hierarchy or inspector edit creates an `EditorLabOperation`;
- a UI visual layout edit uses the `ui_definition` visual operation path where
  the operation is generic UI layout behavior;
- an editor-specific edit uses editor-owned operation contracts or app-owned
  reducer adapters, not `ui_definition`;
- accepted operations produce deterministic diffs;
- invalid operations preserve the draft and surface typed diagnostics;
- operation history records accepted edits and excludes rejected edits;
- undo and redo update draft state and retained preview or structured
  inspector state;
- supported canvas, hierarchy, and inspector entry points round-trip through
  the same operation report shape;
- runtime artifacts are stored under the PM-004 closeout folder.

Screenshots or equivalent retained visual artifacts are required for the
visible operation review, diagnostics, undo, and redo states claimed by the
closeout. Broader screenshot matrices, visual diffing, accessibility checks,
and performance evidence remain `PM-UI-LAB-006`.

## Required Fitness Functions

The linked PM-004 WR must include focused validation for:

- generic UI visual layout operation reuse from `domain/ui/ui_definition`;
- editor-specific operation kind validation in the editor-owned boundary;
- deterministic operation diff text for repeated identical edits;
- direct controls routing to operation reports rather than direct draft
  mutation;
- rejected operations preserving draft state and emitting typed diagnostics;
- history stack behavior for apply, undo, redo, redo clearing, and document
  selection changes;
- retained preview or inspector refresh after accepted, undo, and redo paths;
- unsupported operation disabled reasons and provider-degraded states;
- absence of project IO, persisted operation logs, full apply/rollback, or
  game-runtime projection execution in PM-004.

Minimum validation for the linked implementation row:

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

## WR Candidate

The next roadmap row should be a bounded implementation slice, tentatively
`WR-085: UI Lab operation-driven visual authoring core`.

Primary write scopes:

- `domain/ui/ui_definition/src/visual_layout`
- `domain/editor/editor_definition/src`
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs`
- `domain/editor/editor_shell/src/composition`
- `domain/editor/editor_shell/src/commands/map_interactions.rs`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/tests.rs`
- `docs-site/src/content/docs/reports/implementation-plans/wr-085-ui-lab-operation-driven-visual-authoring-core/plan.md`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

Expected dependencies:

- `WR-004` for editor surface guard support.
- `WR-046` for UI Designer doctrine and target-boundary ratification.
- `WR-083` for completed command catalog and surface registry source of truth.
- `WR-084` for completed app-hosted Editor Lab shell surfaces and
  direct-control route evidence.

The WR row should start as `ready_next`. Product code still requires
`task production:plan -- --milestone PM-UI-LAB-004 --roadmap WR-085` and the
implementation contract it produces.

## Non-Goals

PM-004 does not implement:

- app project packages, save/load/import/export/reload, persisted operation
  logs, complete diff/apply, activation failure preservation, or rollback
  productization owned by `PM-UI-LAB-005`;
- preview scenario matrices, visual diffing, accessibility checks, performance
  evidence, or broad degraded-provider evidence owned by `PM-UI-LAB-006`;
- focused public API usage guides, examples, public API ergonomics review, or
  final runtime-proven track closeout owned by `PM-UI-LAB-007`;
- game-runtime UI projection execution;
- a global command engine, global operation bus, or cross-domain mutable
  registry;
- moving editor execution behavior, app history, project IO, or runtime
  preview refresh into `domain/ui/ui_definition`.

## Stop Conditions

Stop before implementation if PM-004 would:

- require `ui_definition` to own editor semantics, project IO, app history,
  command execution, or runtime preview refresh;
- treat retained preview output, provider frames, or screenshot artifacts as
  source truth;
- reuse scene-runtime history as authoritative Editor Lab definition history
  without an accepted design or ADR;
- bypass deterministic diffs for accepted operations;
- make undo/redo available for operations that cannot be restored
  deterministically;
- implement PM-005 through PM-007 scope under the visual authoring milestone;
- require durable ownership or dependency-direction changes without a new ADR
  or accepted design update.

## Acceptance Bar

PM-004 can move from `designing` to `ready_next` when:

- this accepted design exists;
- a bounded WR row exists for `WR-085`;
- the production milestone links this accepted design and WR row;
- production, roadmap, docs, PUML, and planning validators pass;
- `task ai:goal -- --track PT-UI-LAB --scope non-deferred` reports the next
  legal PM-004 action after design acceptance.
