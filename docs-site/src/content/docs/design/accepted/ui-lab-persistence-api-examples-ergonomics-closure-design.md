---
title: UI Lab Persistence API Examples Ergonomics Closure Design
description: Accepted design for PM-UI-LAB-PERF-005 persistence, structural diff/apply, rollback review, public API, prelude, usage guide, and examples ergonomics closure.
status: accepted
owner: editor
layer: app/domain/docs
canonical: true
last_reviewed: 2026-05-25
related_adrs:
  - ../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/superseded/0006-editor-surface-provider-plugin-seam.md
  - ../../adr/superseded/0012-capability-workbench-clean-break.md
related_designs:
  - ./ui-lab-perfectionist-audit-design.md
  - ./ui-lab-persistence-project-io-diff-apply-rollback-design.md
  - ./ui-lab-api-docs-examples-runtime-closeout-design.md
  - ./ui-lab-runtime-evidence-platform-closure-design.md
  - ./ui-lab-command-surface-source-truth-closure-design.md
  - ./ui-lab-direct-manipulation-ux-closure-design.md
related_reports:
  - ../../reports/closeouts/pm-ui-lab-005-persistence-project-io-diff-apply-and-rollback/closeout.md
  - ../../reports/closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/closeout.md
  - ../../reports/closeouts/pm-ui-lab-perf-004-direct-manipulation-ux-closure/closeout.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# UI Lab Persistence API Examples Ergonomics Closure Design

## Status

Accepted for `PM-UI-LAB-PERF-005`.

This design clears only the PM005 no-gap design gate. It does not authorize
product code until roadmap intake selects a linked WR row, `task
production:plan` creates a decision-complete implementation contract, and WR
roadmap gates allow implementation.

## Goal

The completed PT-UI-LAB track already implemented project IO, diff/apply,
rollback, public API docs, examples, and runtime-proven closeout evidence.
PM005 in the perfectionist track asks a stricter question: whether persistence,
structural diff/apply, rollback review, public APIs, preludes, guides, and
examples form one discoverable normal workflow without hidden shortcuts or
conflicting source truth.

The target normal workflow is:

```text
author direct Editor Lab change
  -> inspect operation report and product-surface diagnostics
  -> save or reload project package with typed diagnostics
  -> review structural diff/apply and activation reports
  -> reject, apply, reload last applied, or rollback with typed evidence
  -> use public APIs, prelude exports, guide, and examples that teach the same path
```

## Current Code Truth

Completed inputs:

- `PM-UI-LAB-005` implemented app-owned project packages, document store,
  apply review, activation reports, failed activation preservation,
  reload-last-applied, and rollback.
- `PM-UI-LAB-007` implemented public API ergonomics review, usage docs,
  examples, and final PT-UI-LAB runtime-proven closeout.
- `PM-UI-LAB-PERF-002` through `PM-UI-LAB-PERF-004` added typed no-gap
  runtime evidence, command/surface source-truth audits, and direct
  manipulation product-surface evidence.

Remaining no-gap blockers:

- persistence evidence must prove structural package state and typed failures,
  not only a saved RON blob or console line;
- diff/apply review must expose structural rows and activation reports through
  product surfaces and public contracts;
- rollback and reload-last-applied must preserve source truth and failure
  inputs with typed diagnostics;
- public entry points must make the normal `ui_definition` and
  `editor_definition` workflows easy to discover without glob-export guesswork;
- examples and guides must use the preferred public APIs rather than app
  internals, test-only helpers, or stale shortcuts;
- docs, examples, public API review, runtime artifacts, roadmap state, and
  production state must agree before PM006 final certification.

## Architecture Governance

Architecture governance for this design-only action:

```text
task ai:architecture-governance -- --task "PM-UI-LAB-PERF-005 persistence diff apply API and examples ergonomics design" --scope "Editor Lab persistence, structural diff/apply, rollback review, public API, prelude, usage guide, and examples ergonomics closure; design-only action, no product code"
```

Governance decisions:

- DDD owner: the `editor` bounded context owns Editor Lab persistence,
  structural review, activation reports, rollback, API ergonomics, and example
  truth.
- App owner: `apps/runenwerk_editor` owns concrete project IO, filesystem
  paths, live activation, failed activation preservation, rollback execution,
  provider sessions, artifact writing, and runtime evidence.
- Domain owner: `domain/editor/editor_definition` owns runtime-neutral editor
  documents, validation, `EditorLabOperation`, operation reports, and any
  package/review DTOs that do not know app runtime or filesystem concerns.
- Generic UI owner: `domain/ui/ui_definition` owns behavior-free UI authoring,
  validation, normalization, visual layout, persistence activation contracts,
  diagnostics, and focused public workflow entry points.
- Docs/examples owner: docs and examples are projections over public contracts;
  they must not invent runtime behavior or teach internal-only APIs as the
  normal path.
- Clean Architecture direction: `ui_definition` and `editor_definition` may
  expose public contracts, but app runtime behavior, provider sessions, and
  project IO stay in app-owned modules.
- ADR need: no new ADR while the implementation preserves the existing
  description/execution, projection, provider seam, and capability workbench
  boundaries. Add an ADR or accepted design update before moving project IO,
  activation execution, rollback, or app history into domain crates.
- ATAM-lite priority order: source-truth correctness first, recoverability
  second, discoverability third, documentation/example agreement fourth,
  compatibility fifth.
- Ownership mode: stream-aligned editor product work with complicated-subsystem
  support from UI definition, editor definition, and docs owners.

## Closure Contract

PM005 closes only the normal persistence and public-usage workflow:

- project package save, reload, import, export, invalid package preservation,
  and migration diagnostics;
- structural draft-versus-applied diff rows, apply review, reject, apply, and
  activation reports;
- failed activation input preservation, previous-state preservation,
  reload-last-applied, rollback, and rollback failure diagnostics;
- focused public entry points and preludes for normal `ui_definition` and
  `editor_definition` workflows;
- usage guide and examples that compile or run through public APIs;
- public API ergonomics review that reconciles `lib.rs`, `prelude.rs`,
  examples, docs indexes, and closeout artifacts.

PM005 must not move app runtime behavior into `ui_definition` or claim final
no-gap certification. PM006 owns the final audit and
`perfectionist_verified` claim.

## Evidence Matrix

The implementation closeout may claim `runtime_proven` only when evidence
proves all of these states:

| Evidence target | Required proof |
|---|---|
| Project package workflow | Save, reload, import, export, invalid package preservation, and typed migration or schema diagnostics. |
| Structural diff/apply workflow | Draft versus applied state produces structural diff rows, review state, reject preservation, accepted apply, and activation report. |
| Rollback workflow | Reload-last-applied and rollback preserve source truth, fail closed without snapshots, and expose typed diagnostics. |
| Public API workflow | Focused exports or preludes make normal UI definition and editor definition workflows discoverable without internal shortcuts. |
| Guide and examples workflow | Usage guide and examples compile or run through preferred public APIs and agree with runtime evidence. |
| Product-surface workflow | Command Diff, diagnostics, and relevant Editor Lab surfaces expose package, review, activation, and rollback state without relying on console-only proof. |

## Implementation Shape

Use a Strangler migration over the existing completed PT-UI-LAB paths:

1. audit current package, review, activation, rollback, public API, guide, and
   examples against the PM005 evidence matrix;
2. repair structural review/report gaps before adding public docs polish;
3. tighten public exports and preludes only where they make normal usage easier
   without hiding advanced modules;
4. update examples and usage guides to demonstrate the preferred public path;
5. write or refresh runtime artifacts proving persistence, diff/apply,
   rollback, public API, guide, and examples agree;
6. keep compatibility APIs available unless an accepted migration explicitly
   changes them.

Acceptable module splits, if needed, should follow owning responsibilities:
`editor_lab_project`, `self_authoring/project_io`, `self_authoring/review`,
`self_authoring/recovery`, or domain `usage`/`prelude` modules. Do not add
catch-all `utils`, `helpers`, or `_internal` modules.

## Required Fitness Functions

The linked implementation WR must include focused validation for:

```text
cargo fmt
cargo test -p ui_definition
cargo test -p editor_definition
cargo test -p runenwerk_editor editor_lab_project
cargo test -p runenwerk_editor pm_ui_lab_perf_005
cargo test -p runenwerk_editor editor_lab
cargo test -p runenwerk_editor editor_definition_activation
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

Tests must prove:

- project package serialization and reload are deterministic;
- invalid packages and failed activations preserve typed inputs and
  diagnostics;
- structural diff/apply rows are product-visible and source-truth preserving;
- reject, apply, reload-last-applied, and rollback mutate only their owning
  state;
- public API examples compile or run through focused exports;
- `ui_definition` remains behavior-free;
- PM006 final no-gap certification stays out of PM005 scope.

## Roadmap Candidate

Roadmap intake after this design should create a bounded WR row for PM005. It
may be one full-slice row only if the write scopes stay clear; otherwise split
project/review/runtime evidence from public API/docs/examples.

Primary write scopes should include whichever of these the intake selects:

- `apps/runenwerk_editor/src/shell/editor_lab_project/`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/editor_app/`
- `apps/runenwerk_editor/src/runtime/`
- `domain/editor/editor_definition/src/`
- `domain/ui/ui_definition/src/`
- Editor Lab examples and docs-site usage guides
- PM005 implementation plan, closeout, roadmap, production, and generated
  planning docs

## Non-Goals

PM005 does not implement:

- final no-gap audit, drift certification, or `perfectionist_verified` owned
  by `PM-UI-LAB-PERF-006`;
- game-runtime UI projection;
- native screenshot/GPU visual-diff breadth beyond the already typed runtime
  evidence inputs;
- broad public API rewrites unrelated to normal Editor Lab usage;
- app runtime behavior, project IO, provider sessions, or rollback execution
  in `domain/ui/ui_definition`.

## Stop Conditions

Stop before implementation if:

- ownership of package, review, rollback, or public API truth is unclear;
- examples require private or test-only APIs;
- app runtime behavior would have to move into `ui_definition` or
  `editor_definition`;
- persistence or diff/apply evidence would be descriptor-only, console-only,
  or docs-only;
- a breaking public API migration is required without an accepted migration
  design or ADR;
- the row starts PM006 final certification scope.
