---
title: WR-108 UI Lab Direct Manipulation UX Closure Contract
description: Current-candidate implementation contract for PM-UI-LAB-PERF-004 direct-manipulation Editor Lab UX closure.
status: active
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/accepted/ui-lab-direct-manipulation-ux-closure-design.md
  - ../../../design/accepted/ui-lab-perfectionist-audit-design.md
related_reports:
  - ../../closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md
  - ../../closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md
  - ../../roadmap-intake/2026-05-25-pm-ui-lab-perf-004-direct-manipulation-e/proposal.yaml
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-108 UI Lab Direct Manipulation UX Closure Contract

## Goal

Implement `PM-UI-LAB-PERF-004` by closing the bounded direct-manipulation UX
gap for the Editor Lab.

WR-108 is now a `current_candidate` row selected by
`task ai:goal -- --track PT-UI-LAB-PERFECTION` for the bounded implementation
contract. Product code changes are allowed only inside the implementation scope
below, with PM005 persistence/API/examples and PM006 final certification left
untouched except for explicit known-gap references in closeout evidence.

## Source Of Truth

- Production milestone: `PM-UI-LAB-PERF-004` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-108` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted PM004 design:
  `docs-site/src/content/docs/design/accepted/ui-lab-direct-manipulation-ux-closure-design.md`.
- Accepted no-gap doctrine:
  `docs-site/src/content/docs/design/accepted/ui-lab-perfectionist-audit-design.md`.
- Completed PM003 command and surface source-truth closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md`.
- Supporting earlier operation-driven authoring closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md`.

Current implementation sources to inspect before code changes:

- `domain/editor/editor_definition/src/operation.rs`
- `domain/editor/editor_definition/src/workflow.rs`
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs`
- `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/tests.rs`

## Current-Candidate Readiness

`task production:plan -- --milestone PM-UI-LAB-PERF-004 --roadmap WR-108`
reported the row as current-candidate eligible after promotion:

- action: `write_implementation_contract`;
- current-candidate eligibility: eligible;
- dependency state: `WR-107:completed`.

The current implementation action is honest because:

- PM003 is completed with runtime-proven command catalog and surface registry
  source-truth closeout evidence.
- PM004 has an accepted direct-manipulation UX closure design.
- WR-108 has bounded direct-manipulation write scopes and explicit non-goals.
- WR-108 was promoted to `current_candidate` with the evidence below, so this
  contract may plan only PM004 direct-manipulation product-surface
  implementation.

Recorded promotion evidence:

```text
Accepted PM-UI-LAB-PERF-004 direct manipulation UX closure design plus completed WR-107 command and surface source-truth closeout clear WR-108 for current-candidate implementation planning; PM004 remains limited to hierarchy, palette, canvas, inspector, diagnostics, operation diff, preview console, undo, redo, and runtime product-surface evidence without starting persistence/API or final certification scope.
```

## Architecture Decisions

Source-truth decisions:

- `domain/editor/editor_definition` owns typed `EditorLabOperation`, workflow,
  operation report, and undo/redo semantics that are runtime-neutral and
  reusable outside the native app shell.
- `domain/editor/editor_shell` owns app-neutral Editor Lab surface composition,
  retained view models, direct-manipulation affordance structure, and product
  surface contracts.
- `apps/runenwerk_editor` owns native shell execution, command dispatch,
  provider sessions, runtime refresh, captured evidence, and app-specific
  diagnostics.
- `domain/ui/ui_definition` remains behavior-free and does not become an
  operation executor, history owner, provider host, or preview runtime.
- PM004 consumes PM003 command/surface truth and must not reintroduce fallback
  command, surface, route, or disabled-reason ownership.

Forbidden shortcuts:

- replacing normal direct manipulation with debug-style action lists, raw text
  panels, or status-only assertions;
- claiming retained rows alone as native runtime proof when an app-owned
  product-surface artifact is required;
- moving editor operation execution, history, provider behavior, preview
  refresh, or diagnostics ownership into `ui_definition`;
- starting PM005 persistence, diff/apply public API, examples, or PM006 final
  certification work;
- broadening WR-108 into a reusable cross-domain operation bus without a new
  accepted design or ADR gate.

## Implementation Scope

Expected implementation files:

- `domain/editor/editor_definition/src/operation.rs`
- `domain/editor/editor_definition/src/workflow.rs`
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs`
- `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/tests.rs`

Expected evidence and docs after implementation:

- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-004-direct-manipulation-ux-closure/closeout.md`
- Product-surface runtime evidence artifacts under that closeout directory if
  the focused tests generate retained or native evidence files.
- Roadmap and production metadata/rendered docs after closeout.

Expected contract changes after promotion:

- Add typed operation-report coverage for hierarchy, palette, canvas,
  inspector, diagnostics, operation diff, preview console, undo, and redo.
- Route normal authoring controls through `EditorLabOperation` paths rather than
  debug action rows or ad hoc shell mutation.
- Keep selection, diagnostics, operation history, undo, redo, and preview
  refresh deterministic across domain, shell, and app boundaries.
- Add app-owned product-surface evidence proving normal author workflows use
  direct controls and operation reports.
- Preserve the PM003 command catalog and surface registry source-truth
  boundaries while adding PM004 UX behavior.

## Implementation Steps

1. Inspect the current typed operation, workflow, shell composition,
   self-authoring provider, command dispatch, and app test surfaces listed
   above before changing code.
2. Extend `domain/editor/editor_definition/src/operation.rs` and
   `domain/editor/editor_definition/src/workflow.rs` only where typed
   operation reports or deterministic workflow state are missing for PM004
   direct-manipulation evidence.
3. Extend `domain/editor/editor_shell/src/surfaces/editor_definition.rs` and
   `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
   so the retained product surface exposes hierarchy, palette, canvas,
   inspector, diagnostics, operation diff, preview console, undo, and redo as
   normal authoring affordances.
4. Extend `apps/runenwerk_editor/src/shell/self_authoring.rs`,
   `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`, and
   `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` so app-owned
   execution routes normal direct controls through typed operations and
   provider refresh paths.
5. Add focused tests in `apps/runenwerk_editor/src/shell/tests.rs` and the
   owning domain crates to prove product-surface direct manipulation,
   deterministic reports, undo, redo, diagnostics, preview refresh, and
   PM003 source-truth preservation.
6. Run the focused validation commands, create the PM004 closeout only after
   they pass, archive WR-108 only after the closeout exists, then rerun
   roadmap, production, planning, PUML, docs, and diff checks.

## Acceptance Criteria

- Canvas, hierarchy, palette, and inspector edits round-trip through typed
  `EditorLabOperation` paths.
- Diagnostics, validation, operation diff, preview console, selection, undo,
  redo, and preview refresh are deterministic and product-surface visible.
- Normal authoring workflows do not require debug-style action lists or raw
  document text panels.
- Product-surface runtime evidence proves hierarchy, palette, canvas,
  inspector, diagnostics, operation diff, preview console, undo, and redo.
- PM005 persistence/API/examples scope and PM006 final no-gap certification
  scope remain untouched.
- `ui_definition` remains behavior-free.

## Validation

Implementation validation:

```text
cargo fmt
cargo test -p ui_definition visual_layout
cargo test -p editor_definition operation
cargo test -p editor_shell editor_lab
cargo test -p runenwerk_editor editor_lab_operation
cargo test -p runenwerk_editor direct_manipulation
cargo test -p runenwerk_editor pm_ui_lab_perf_004
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
task ai:goal -- --track PT-UI-LAB-PERFECTION
```

Contract-only validation for this action:

```text
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
task ai:goal -- --track PT-UI-LAB-PERFECTION
```

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-004-direct-manipulation-ux-closure/closeout.md`
only after focused implementation tests and metadata gates pass.

The closeout must include:

- reproduction commands;
- product-surface evidence for hierarchy, palette, canvas, inspector,
  diagnostics, operation diff, preview console, undo, and redo;
- exact proof that normal authoring workflows no longer depend on debug-style
  action lists or raw text panels;
- proof that command and surface ownership still follows PM003 source truth;
- validation output;
- remaining known quality gaps that belong to PM005 and PM006;
- roadmap archive and production milestone updates only after validation.

## Perfectionist Closeout Audit

WR-108 may close PM004 at `runtime_proven` only if product-surface tests prove
direct-manipulation authoring behavior through typed operations and runtime
evidence. It must not claim `perfectionist_verified`.

Remaining gaps after PM004 are expected:

- persistence, diff/apply, public API, and examples ergonomics closure remains
  PM005;
- final no-gap certification remains PM006.

## Stop Conditions

Stop implementation if:

- WR-108 is not promoted to `current_candidate` before product code changes;
- direct-manipulation closure requires a new operation ownership model without
  an accepted design update or ADR;
- product-surface evidence cannot prove normal author workflows;
- implementation requires moving editor operation execution, app history,
  provider behavior, preview refresh, or diagnostics into `ui_definition`;
- the row starts PM005 persistence/API/examples scope or PM006 final
  certification scope.
