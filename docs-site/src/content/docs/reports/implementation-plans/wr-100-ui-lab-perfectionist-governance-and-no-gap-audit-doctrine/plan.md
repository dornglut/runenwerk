---
title: WR-100 UI Lab Perfectionist Governance And No-Gap Audit Doctrine Contract
description: Design-first governance contract for PM-UI-LAB-PERF-001 code-truth reconciliation, evidence doctrine, hard blockers, and follow-on no-gap WR candidates.
status: active
owner: editor
layer: workspace/domain/app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/accepted/ui-lab-perfectionist-audit-design.md
  - ../../../design/active/ui-lab-productization-design.md
  - ../../../design/accepted/ui-lab-command-catalog-and-surface-registry-design.md
  - ../../../design/accepted/ui-lab-operation-driven-visual-authoring-design.md
  - ../../../design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md
  - ../../../design/accepted/ui-lab-preview-lab-runtime-evidence-design.md
  - ../../../design/accepted/ui-lab-api-docs-examples-runtime-closeout-design.md
related_reports:
  - ../../closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md
  - ../../closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/closeout.md
  - ../../roadmap-intake/2026-05-24-pt-ui-lab-perfection-no-gap-audit/proposal.yaml
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-100 UI Lab Perfectionist Governance And No-Gap Audit Doctrine Contract

## Goal

Clear the `PM-UI-LAB-PERF-001` design-first blocker by making the Editor Lab V1
no-gap audit executable without starting app or domain implementation.

This contract defines the audit doctrine, code-truth matrix, evidence matrix,
hard blockers, follow-on WR candidates, validation gates, and closeout rules for
`PT-UI-LAB-PERFECTION`. It is a governance/design slice only. Product code may
start only after a later milestone has an accepted design gate, a linked WR row,
and a fresh `task production:plan -- --milestone "<PM-ID>" --roadmap "<WR-ID>"`
implementation contract.

## Source Of Truth

- Production track: `PT-UI-LAB-PERFECTION` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Production milestone: `PM-UI-LAB-PERF-001`.
- Roadmap row: `WR-100` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted no-gap design:
  `docs-site/src/content/docs/design/accepted/ui-lab-perfectionist-audit-design.md`.
- Runtime-proven input closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/closeout.md`.
- Runtime evidence input closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md`.

Source-truth decisions:

- `domain/ui/ui_definition` owns behavior-free generic UI definition contracts,
  validation, visual layout operations, preview descriptors, persistence
  activation descriptors, production readiness descriptors, and public workflow
  helpers.
- `domain/editor/editor_definition` owns runtime-neutral editor definition
  documents, command/menu/toolbar/surface/shortcut vocabulary, Editor Lab
  operations, deterministic diffs, validation, focused workflow helpers, and
  examples.
- `domain/editor/editor_shell` owns app-neutral shell composition, surface
  contracts, view models, route projections, tool-suite registry contracts, and
  retained UI projection structure.
- `apps/runenwerk_editor` owns concrete app command execution, provider
  sessions, project IO, live activation, rollback, runtime evidence capture,
  artifact writing, and unsupported/platform-impossible diagnostics.
- Production and roadmap YAML sources own planning state. Generated Markdown,
  PUML, indexes, and registers are derived outputs.

## Readiness

`task production:plan -- --milestone PM-UI-LAB-PERF-001 --roadmap WR-100`
classifies the next action as `design_first`.

The row is not implementation-ready because:

- `WR-100` is `planning_state=current_candidate`, but blocker `B3` is above the
  B2 implementation gate.
- `PM-UI-LAB-PERF-001` is a design milestone and must not edit app or domain
  runtime code.
- `PM-UI-LAB-PERF-002` through `PM-UI-LAB-PERF-006` are still `designing` and
  depend on accepted no-gap doctrine.
- Later milestones require the no-gap audit design to be accepted before
  runtime evidence, source-of-truth, UX, API, or final certification work
  begins.

The legal WR-100 action is therefore to complete governance/design evidence:
code-truth reconciliation, no-gap evidence doctrine, blocker matrix, follow-on
candidate matrix, and closeout instructions.

## Code-Truth Matrix

| Area | Current repo truth | No-gap blocker | Owning future milestone |
|---|---|---|---|
| Runtime evidence | `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs` and the PM006 artifacts record retained visual, diagnostics, accessibility, performance, degraded-provider, reload, apply, and rollback evidence. | Native screenshots, GPU visual diffing, native focus traversal, pixel contrast sampling, native screenshot timing, and GPU visual-diff timing remain unsupported-check diagnostics. Unsupported is acceptable only when the runtime cannot expose the check. | `PM-UI-LAB-PERF-002` |
| Command catalog | `apps/runenwerk_editor/src/shell/command_catalog/mod.rs`, `command_resolution.rs`, `toolbar_adapter.rs`, and `dispatch_shell_command.rs` provide catalog and dispatch paths. | Labels, disabled reasons, toolbar/menu/keybinding projections, route fallbacks, and dispatch targets still need a no-gap audit proving one normal catalog source. | `PM-UI-LAB-PERF-003` |
| Surface registry | `domain/editor/editor_shell/src/tool_suite`, `workspace/surface_contract.rs`, `workspace/state.rs`, app provider registration, and legacy compatibility paths provide tool-surface metadata. | `ToolSurfaceDefinitionRegistry` must be proven as the normal source for identity, capability, retention, provider family, creation policy, and routing. Legacy enum authority must be isolated to migration and persistence edges. | `PM-UI-LAB-PERF-003` |
| Direct manipulation | `domain/editor/editor_shell/src/surfaces/editor_definition.rs`, `composition/build_editor_lab_surface.rs`, `apps/runenwerk_editor/src/shell/self_authoring.rs`, and `providers/self_authoring.rs` provide retained Editor Lab controls and operation-driven paths. | The normal workflow must not depend on debug action lists, status panels, or text-only review. Hierarchy, palette, canvas, inspector, diagnostics, operation diff, undo, redo, and preview refresh need product-surface evidence. | `PM-UI-LAB-PERF-004` |
| Persistence and apply | `apps/runenwerk_editor/src/shell/editor_lab_project/mod.rs`, `apps/runenwerk_editor/src/shell/applied_editor_definition/activation.rs`, and `domain/ui/ui_definition/src/persistence_activation/mod.rs` provide project package, activation, and runtime-neutral descriptor paths. | Structural diff/apply review, reload, failed activation preservation, rollback, and package diagnostics must expose typed product evidence rather than coarse serialized rows or console-only status. | `PM-UI-LAB-PERF-005` |
| Public APIs and examples | `domain/ui/ui_definition/src/{prelude.rs,workflow.rs}`, `domain/editor/editor_definition/src/{prelude.rs,workflow.rs}`, and both public examples were added by PM007. | Broad compatibility exports still exist, and no-gap certification must review whether normal imports, guides, examples, and app handoff remain focused after the implementation milestones. | `PM-UI-LAB-PERF-005` |
| Module structure | Existing code is split by editor shell, editor definition, UI definition, and app runtime evidence owners. | The audit must catch catch-all growth, hidden app behavior in domain crates, legacy owner leakage, and source-of-truth duplication before final certification. | `PM-UI-LAB-PERF-003`, `PM-UI-LAB-PERF-006` |
| Final certification | Production and roadmap state list the perfectionist track as active with known future gaps. | `perfectionist_verified` is illegal until every prerequisite milestone is completed, every linked WR row has valid evidence, generated planning state agrees, and `known_quality_gaps` is empty. | `PM-UI-LAB-PERF-006` |

## Evidence Matrix

| Evidence target | Minimum acceptable proof | Invalid proof | Owner |
|---|---|---|---|
| Native screenshot or typed-impossible result | Runtime artifact with command, backend/window constraints, artifact path, and typed result; typed-impossible only when the current runtime cannot expose native capture. | Retained debug text pretending to be native pixels. | `apps/runenwerk_editor` |
| GPU visual diff | Pixel artifact or explicit platform-impossible diagnostic with reproduction notes and unchanged/changed region intent. | Descriptor-only or status-panel diff. | `apps/runenwerk_editor`, renderer support if required |
| Focus traversal and contrast | Native or retained accessibility artifact with element identity, route labels, disabled reasons, and sampling limits. | Human-readable notes without artifact paths. | `apps/runenwerk_editor`, `editor_shell` |
| Command source truth | Tests and audit notes proving menu, toolbar, palette, keybinding, routing, enablement, labels, disabled reasons, and dispatch use catalog data. | Parallel hard-coded labels or fallback routes that pass only one surface. | `apps/runenwerk_editor` |
| Surface source truth | Registry tests and audit notes proving surface identity, capabilities, retention, provider family, creation policy, and routing metadata come from registry definitions. | Legacy enum switch tables as the normal path. | `domain/editor/editor_shell`, `apps/runenwerk_editor` |
| Direct manipulation | Runtime evidence for hierarchy, palette, canvas, inspector, diagnostics, operation diff, undo, redo, and preview refresh through normal controls. | Action-list, text-panel, or provider-only evidence. | `apps/runenwerk_editor`, `editor_shell`, `editor_definition`, `ui_definition` |
| Persistence diff/apply | Saved package, reload, structural diff, rejected apply, accepted apply, failed activation preservation, rollback, and diagnostics artifacts. | Coarse serialized document rows without structural review. | `apps/runenwerk_editor`, `editor_definition`, `ui_definition` |
| Public API ergonomics | Compile-backed examples, usage docs, prelude/workflow review, and app handoff boundaries. | Docs that use private shortcuts or internal-only helpers as normal usage. | `ui_definition`, `editor_definition`, docs |
| Final closeout | Closeout report linking runtime artifacts, API review, examples, docs, roadmap, production state, and drift-check evidence. | A completion-quality claim without empty known gaps and valid linked evidence. | workspace/editor |

## Follow-On WR Candidate Matrix

The IDs below are reserved candidate labels for planning conversation only.
They are not active roadmap rows until accepted through the roadmap workflow.

| Candidate | Milestone | Dependency | Disjoint write scope | Validation and runtime evidence |
|---|---|---|---|---|
| `WR-105` Runtime Evidence Platform Closure | `PM-UI-LAB-PERF-002` | `WR-100` completed and no-gap design accepted | `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs`, `apps/runenwerk_editor/src/shell/tests.rs`, PM002 closeout artifacts, production and roadmap YAML/rendered docs | `cargo test -p runenwerk_editor pm_ui_lab_perf_002`; artifact-writing command for native or typed-impossible evidence; `task docs:validate`; roadmap/production render, validate, check |
| `WR-106` Command And Surface Source Of Truth Closure | `PM-UI-LAB-PERF-003` | `WR-100` completed and no-gap design accepted | `apps/runenwerk_editor/src/shell/command_catalog`, command projection/dispatch modules, `domain/editor/editor_shell/src/tool_suite`, workspace surface contracts, provider registration tests, PM003 closeout | `cargo test -p editor_shell`; `cargo test -p runenwerk_editor command`; `cargo test -p runenwerk_editor surface`; module-structure audit; roadmap/production gates |
| `WR-107` Direct Manipulation Editor Lab UX Closure | `PM-UI-LAB-PERF-004` | `WR-106` completed | `domain/editor/editor_definition/src/operation.rs`, `workflow.rs`, `domain/editor/editor_shell/src/surfaces/editor_definition.rs`, Editor Lab composition, app self-authoring/provider/dispatch tests, PM004 artifacts | `cargo test -p editor_definition operation`; `cargo test -p editor_shell editor_lab`; `cargo test -p runenwerk_editor editor_lab_operation`; runtime evidence for normal hierarchy/palette/canvas/inspector workflows |
| `WR-108` Persistence Diff Apply API And Examples Ergonomics | `PM-UI-LAB-PERF-005` | `WR-107` completed | `apps/runenwerk_editor/src/shell/editor_lab_project`, app activation/apply paths, `domain/editor/editor_definition` workflow/diff APIs, `domain/ui/ui_definition` usage docs/examples when public ergonomics change, PM005 closeout | `cargo test -p ui_definition`; `cargo test -p editor_definition`; `cargo test -p runenwerk_editor editor_lab_project`; public examples; runtime evidence for save/reload/diff/apply/failure/rollback |
| `WR-109` Final No-Gap Certification Closeout | `PM-UI-LAB-PERF-006` | `WR-108` completed | Final closeout report, audit matrix, production/roadmap YAML, generated indexes/registers/diagrams, drift-check artifacts | Final production and roadmap render/validate/check; `task planning:validate`; `task docs:validate`; `git diff --check`; empty `known_quality_gaps`; all linked evidence paths completed |

## Implementation Scope

WR-100 may change only governance/design artifacts:

- `docs-site/src/content/docs/reports/implementation-plans/wr-100-ui-lab-perfectionist-governance-and-no-gap-audit-doctrine/plan.md`
- `docs-site/src/content/docs/design/accepted/ui-lab-perfectionist-audit-design.md`
- `docs-site/src/content/docs/design/accepted/README.md`
- `docs-site/src/content/docs/design/active/README.md`
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-pt-ui-lab-perfection-no-gap-audit`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`
- generated production and roadmap registers/diagrams created by the render tasks

No `domain/`, `engine/`, `apps/`, shader, example, benchmark, or runtime code
belongs to WR-100.

## Acceptance Criteria

- Architecture governance decisions are recorded in the no-gap design and this
  contract.
- The code-truth matrix names current source files and remaining blockers for
  runtime evidence, command catalog, surface registry, direct manipulation,
  persistence/diff/apply, APIs/examples, module structure, and final closeout.
- The evidence matrix distinguishes native/runtime proof, retained evidence,
  typed platform-impossible diagnostics, and invalid descriptor/status-only
  evidence.
- Follow-on WR candidates are disjoint, ordered by milestone dependency, and
  name validation and runtime evidence expectations.
- The contract path is part of WR-100 write scopes before closeout relies on
  it.
- No app, domain, engine, shader, benchmark, or product runtime code changes
  are made by WR-100.

## Stop Conditions

Stop before implementation if:

- a later milestone tries to use WR-100 as permission to edit product code;
- the no-gap design is still active when a later milestone requires accepted
  doctrine;
- a proposed implementation moves provider sessions, project IO, activation,
  rollback, runtime evidence, screenshots, accessibility runners, or artifact
  writing into `domain/ui/ui_definition`;
- retained previews, descriptors, status panels, prepared data, or console
  lines are proposed as final proof for a runtime-visible claim;
- command, surface, operation, persistence, diff/apply, API, or evidence paths
  have two normal sources of truth;
- candidate write scopes overlap in a way that prevents one-bounded-slice
  implementation;
- any required validation, closeout evidence, production check, roadmap check,
  planning check, or docs check fails.

## Closeout Requirements

WR-100 can close only after:

- this contract exists and is linked from WR-100 write scopes;
- the active no-gap design includes or links the code-truth matrix, evidence
  matrix, hard blockers, and follow-on candidate matrix;
- PM-UI-LAB-PERF-001 has a completed closeout report under
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-001-governance-audit-doctrine-and-code-truth-matrix/closeout.md`;
- the no-gap doctrine is accepted or the follow-on milestones remain blocked
  with honest design-gate language;
- `WR-100` is completed only with `completion_quality=bounded_contract` or
  `not_applicable`, never `runtime_proven` or `perfectionist_verified`;
- `PM-UI-LAB-PERF-001` remains honest about unimplemented future milestones in
  `known_quality_gaps`;
- `task production:render`, `task production:validate`,
  `task production:check`, `task roadmap:render`, `task roadmap:validate`,
  `task roadmap:check`, `task planning:validate`, `task puml:validate`, and
  `task docs:validate` pass.

After closeout, rerun:

```text
task ai:goal -- --track PT-UI-LAB-PERFECTION
```

If `PM-UI-LAB-PERF-002` still reports `wait_for_dependency_completion`, stop
and repair the exact missing PM001 completion evidence before any
implementation.

## Perfectionist Closeout Audit

WR-100 cannot claim runtime proof or no-gap completion. Its completion quality
is at most `bounded_contract` because every runtime/product milestone remains
future scope.

The final `PT-UI-LAB-PERFECTION` `perfectionist_verified` claim is legal only
when:

- `PM-UI-LAB-PERF-001` through `PM-UI-LAB-PERF-006` are completed with valid
  evidence gates;
- every linked WR row has completed closeout evidence and honest
  `completion_quality`;
- source truth and derived runtime products agree for commands, surfaces,
  operations, persistence, diff/apply, APIs, examples, docs, and artifacts;
- module-structure review has no unresolved ownership leakage;
- all generated roadmap and production docs are fresh;
- `known_quality_gaps` is empty.
