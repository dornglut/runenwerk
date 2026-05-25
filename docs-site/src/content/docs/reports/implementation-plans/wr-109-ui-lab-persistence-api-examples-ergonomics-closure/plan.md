---
title: WR-109 UI Lab Persistence API Examples Ergonomics Closure Contract
description: Promotion and implementation-readiness contract for PM-UI-LAB-PERF-005 persistence, structural diff/apply, rollback, public API, guides, and examples ergonomics closure.
status: active
owner: editor
layer: app/domain/docs
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/accepted/ui-lab-persistence-api-examples-ergonomics-closure-design.md
  - ../../../design/accepted/ui-lab-perfectionist-audit-design.md
related_reports:
  - ../../closeouts/pm-ui-lab-perf-004-direct-manipulation-ux-closure/closeout.md
  - ../../closeouts/pm-ui-lab-005-persistence-project-io-diff-apply-and-rollback/closeout.md
  - ../../closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/closeout.md
  - ../../roadmap-intake/2026-05-25-pm-ui-lab-perf-005-persistence-diff-appl/proposal.yaml
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-109 UI Lab Persistence API Examples Ergonomics Closure Contract

## Goal

Implement `PM-UI-LAB-PERF-005` by closing the bounded persistence,
structural diff/apply, rollback review, public API, guide, and examples
ergonomics gap for the Editor Lab.

WR-109 is now a `current_candidate` row selected by
`task ai:goal -- --track PT-UI-LAB-PERFECTION` for the bounded implementation
contract. Product code changes are allowed only inside the implementation scope
below, with PM006 final certification left untouched except for explicit
known-gap references in closeout evidence.

## Source Of Truth

- Production milestone: `PM-UI-LAB-PERF-005` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-109` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted PM005 design:
  `docs-site/src/content/docs/design/accepted/ui-lab-persistence-api-examples-ergonomics-closure-design.md`.
- Accepted no-gap doctrine:
  `docs-site/src/content/docs/design/accepted/ui-lab-perfectionist-audit-design.md`.
- Completed PM004 direct-manipulation closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-004-direct-manipulation-ux-closure/closeout.md`.
- Supporting completed persistence closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-005-persistence-project-io-diff-apply-and-rollback/closeout.md`.
- Supporting completed API/docs/examples closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/closeout.md`.

Current implementation sources to inspect before code changes:

- `apps/runenwerk_editor/src/shell/editor_lab_project`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/editor_app`
- `apps/runenwerk_editor/src/runtime`
- `apps/runenwerk_editor/src/shell/tests.rs`
- `domain/editor/editor_definition/src/lib.rs`
- `domain/editor/editor_definition/src/prelude.rs`
- `domain/editor/editor_definition/src/workflow.rs`
- `domain/ui/ui_definition/src/lib.rs`
- `domain/ui/ui_definition/src/prelude.rs`
- `domain/ui/ui_definition/src/workflow.rs`
- `docs-site/src/content/docs/domain/ui/ui-definition-usage.md`
- `docs-site/src/content/docs/domain/editor/editor-definition-usage.md`

## Current-Candidate Readiness

`task production:plan -- --milestone PM-UI-LAB-PERF-005 --roadmap WR-109`
reported:

- action: `write_implementation_contract`;
- roadmap state: `current_candidate`;
- current-candidate eligibility: eligible;
- dependency state: `WR-108:completed`;
- milestone link: yes.

The current implementation action is honest because:

- PM004 is completed with product-surface direct-manipulation closeout evidence.
- PM005 has an accepted persistence, diff/apply, public API, guide, and examples
  ergonomics design.
- WR-109 is bounded to PM005 and explicitly excludes PM006 final no-gap
  certification.
- The row carries supporting completed PM-UI-LAB-005 and PM-UI-LAB-007 evidence
  without reopening the completed PT-UI-LAB track.
- WR-109 was promoted to `current_candidate` with the evidence below, so this
  contract may plan only PM005 persistence, structural review, recovery, public
  API, usage guide, and example ergonomics implementation.

Recorded promotion evidence:

```text
Accepted PM-UI-LAB-PERF-005 persistence API examples ergonomics closure design plus completed WR-108 direct-manipulation UX closeout, completed PM-UI-LAB-005 persistence rollback evidence, and completed PM-UI-LAB-007 public API docs examples evidence clear WR-109 for current-candidate implementation planning; PM005 remains limited to persistence, structural diff/apply, rollback review, public API, guide, and examples ergonomics without starting PM006 final no-gap certification.
```

## Architecture Decisions

Source-truth decisions:

- `apps/runenwerk_editor` owns project IO, filesystem paths, live activation,
  failed activation preservation, rollback execution, provider sessions,
  runtime evidence, and artifact writing.
- `domain/editor/editor_definition` owns runtime-neutral editor documents,
  validation, `EditorLabOperation`, operation reports, and public workflow
  entry points that do not depend on the native app runtime.
- `domain/ui/ui_definition` owns behavior-free UI authoring definitions,
  validation, normalization, visual layout, persistence activation contracts,
  diagnostics, and focused public workflow entry points.
- Docs and examples are projections over public contracts and must teach the
  same normal workflow proven by runtime and product-surface evidence.
- PM005 may tighten discoverability, structural reporting, and examples, but it
  must not move app execution or provider state into domain crates.

Forbidden shortcuts:

- descriptor-only, console-only, RON-blob-only, or docs-only evidence for
  persistence, diff/apply, rollback, or public API claims;
- teaching private, test-only, or app-internal APIs as the normal public usage
  path;
- broad public API rewrites unrelated to normal Editor Lab usage;
- moving project IO, activation execution, rollback, provider sessions, or app
  history into `domain/ui/ui_definition`;
- claiming `perfectionist_verified` or closing final no-gap certification in
  PM005.

## Implementation Scope

Expected implementation files:

- `apps/runenwerk_editor/src/shell/editor_lab_project`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/editor_app`
- `apps/runenwerk_editor/src/runtime`
- `apps/runenwerk_editor/src/shell/tests.rs`
- `domain/editor/editor_definition/src/lib.rs`
- `domain/editor/editor_definition/src/prelude.rs`
- `domain/editor/editor_definition/src/workflow.rs`
- `domain/ui/ui_definition/src/lib.rs`
- `domain/ui/ui_definition/src/prelude.rs`
- `domain/ui/ui_definition/src/workflow.rs`
- `docs-site/src/content/docs/domain/ui/ui-definition-usage.md`
- `docs-site/src/content/docs/domain/editor/editor-definition-usage.md`

Expected evidence and docs after implementation:

- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-005-persistence-diff-apply-api-and-examples-ergonomics/closeout.md`
- PM005 runtime/product-surface artifacts under that closeout directory if the
  focused tests generate retained, package, diff/apply, rollback, or API review
  evidence files.
- Roadmap and production metadata/rendered docs after closeout.

Expected current-candidate implementation changes:

- Audit the existing package save/reload/import/export, invalid package,
  activation, reload-last-applied, rollback, public API, usage guide, and
  example chain before changing code.
- Repair structural review/report gaps before docs polish.
- Expose structural draft-versus-applied diff rows and activation reports
  through product surfaces and public contracts.
- Prove failed activation input preservation, prior-valid-state preservation,
  reload-last-applied, and rollback diagnostics.
- Tighten `lib.rs`, `prelude.rs`, workflow entry points, usage guides, and
  examples only where normal usage remains hard to discover or inconsistent.
- Preserve compatibility paths deliberately unless an accepted migration gate
  authorizes a breaking public API change.

## Implementation Steps

1. Inspect the project IO, activation, rollback, domain workflow, public export,
   usage guide, and example sources listed above before code changes.
2. Map the existing completed PM-UI-LAB-005 and PM-UI-LAB-007 behavior to the
   stricter PM005 evidence matrix, naming any missing product-surface,
   structural report, or public-discoverability proof.
3. Repair project package and diff/apply reporting so normal save, reload,
   invalid package, draft-versus-applied review, apply, reject, and activation
   paths expose typed structural evidence.
4. Repair reload-last-applied and rollback review so successful and failed
   recovery paths preserve source truth and surface typed diagnostics.
5. Tighten public exports, preludes, workflows, guides, and examples so normal
   users can follow the same public path proven by tests and runtime evidence.
6. Add focused tests and artifact generation for PM005; close out only after
   focused validation passes and roadmap/production/generated docs agree.

## Acceptance Criteria

- Project package save, reload, import/export, invalid package preservation,
  and typed package diagnostics are deterministic and product-visible.
- Draft-versus-applied state produces structural diff rows, review state,
  reject preservation, accepted apply, and activation reports.
- Reload-last-applied and rollback preserve source truth, fail closed without
  hidden snapshots, and expose typed diagnostics.
- Public `ui_definition` and `editor_definition` entry points are focused on
  normal workflows without hiding advanced APIs or requiring glob-export
  guesswork.
- Usage guides and examples compile or run through preferred public APIs and
  agree with runtime evidence.
- PM006 final no-gap certification scope remains untouched.

## Validation

Implementation validation:

```text
cargo fmt
cargo test -p ui_definition
cargo test -p editor_definition
cargo test -p runenwerk_editor editor_lab_project
cargo test -p runenwerk_editor pm_ui_lab_perf_005
cargo test -p runenwerk_editor editor_lab
cargo test -p runenwerk_editor editor_definition_activation
cargo run -p ui_definition --example ui_definition_workflow
cargo run -p editor_definition --example editor_definition_workflow
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
`docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-005-persistence-diff-apply-api-and-examples-ergonomics/closeout.md`
only after focused implementation tests and metadata gates pass.

The closeout must include:

- reproduction commands;
- package, structural diff/apply, activation, reload-last-applied, rollback,
  public API, guide, and example evidence;
- exact proof that examples and guides use public APIs instead of private,
  test-only, or app-internal shortcuts;
- proof that `ui_definition` remains behavior-free and app runtime behavior
  stays app-owned;
- validation output;
- remaining known quality gap that final no-gap certification belongs to PM006;
- roadmap archive and production milestone updates only after validation.

## Perfectionist Closeout Audit

WR-109 may close PM005 at `runtime_proven` only if product-surface, runtime,
public API, guide, and example evidence prove the full persistence and public
usage chain. It must not claim `perfectionist_verified`.

The only expected remaining gap after PM005 is final no-gap certification in
PM006.

## Stop Conditions

Stop implementation if:

- WR-109 is not promoted to `current_candidate` before product code changes;
- structural package, diff/apply, rollback, or API truth ownership is unclear;
- examples require private, test-only, or app-internal APIs for normal usage;
- app runtime behavior, project IO, activation execution, rollback, provider
  sessions, or app history would have to move into `ui_definition`;
- evidence would be descriptor-only, console-only, RON-blob-only, or docs-only;
- a breaking public API migration is required without an accepted migration
  design or ADR;
- the row starts PM006 final no-gap certification scope.
