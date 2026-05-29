---
title: WR-120 UI Designer Workbench Product Track Governance Contract
description: Design-first planning contract for PM-UI-DESIGNER-WB-001, reconciling code truth, planning truth, archived WR-114 evidence, and follow-on UI Designer Workbench productization scope before product code changes.
status: active
owner: editor
layer: workspace / domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/active/ui-designer-interface-lab-platform-design.md
  - ../../../design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-production-readiness-and-evidence-design.md
related_reports:
  - ../wr-114-standalone-ui-designer-workbench/plan.md
  - ../../closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md
  - ../../roadmap-intake/2026-05-25-create-a-new-pt-ui-designer-workbench-pr/proposal.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# WR-120 UI Designer Workbench Product Track Governance Contract

## Goal

Define the design-first governance contract for `PM-UI-DESIGNER-WB-001` and
`WR-120` before any UI Designer Workbench product code starts.

This contract is produced from:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-001 --roadmap WR-120
```

The command currently classifies the next action as `design_first`. This action
does not promote `WR-120`, does not complete `PM-UI-DESIGNER-WB-001`, and does
not authorize product implementation. It records the governance boundary,
code-truth drift, follow-on slice decomposition, validation gates, and closeout
requirements needed before a later coordinator run can decide whether `WR-120`
is ready for promotion or completion.

Expected governance outcome:

- the accepted UI Designer Workbench Product Design is the product source of
  truth for follow-on work;
- the completed `WR-114` / `PM-EDITOR-UX-004` evidence remains historical,
  bounded workbench-route evidence, not proof of the full V1 workbench;
- UI definition and token truth remain in `domain/ui`;
- editor/workbench view models, surface semantics, routes, and shell adapters
  remain in `domain/editor`;
- native app execution, provider fixtures, runtime launch, and evidence remain
  in `apps/runenwerk_editor`;
- follow-on implementation candidates have separate source scopes and must each
  pass production planning, roadmap legality, validation, and closeout.

## Source Of Truth

- Production track and milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml`
  milestone `PM-UI-DESIGNER-WB-001`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` item `WR-120`.
- Accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Completed bounded predecessor evidence:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md`.
- Historical implementation contract:
  `docs-site/src/content/docs/reports/implementation-plans/wr-114-standalone-ui-designer-workbench/plan.md`.
- Current app launch truth:
  `apps/runenwerk_editor/src/runtime/app.rs` enum
  `RunenwerkRuntimeWorkbench`, function `run_material_lab_workbench`, and
  `apps/runenwerk_editor/src/bin/runenwerk_material_lab.rs`.
- Current UI Designer provider truth:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` function
  `ui_designer_workbench_view_model`.
- Current editor workbench view-model truth:
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`.
- Current editor composition truth:
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`.

## Readiness

`task production:plan -- --milestone PM-UI-DESIGNER-WB-001 --roadmap WR-120`
reports:

```text
Production milestone state: designing
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-114:completed
Next action: design_first
```

`WR-120` is not ready for product implementation. Before promotion or
completion, the row still needs:

- this active planning contract;
- architecture-governance review evidence;
- a code-truth and planning-truth drift classification;
- explicit follow-on implementation candidates with disjoint source scopes;
- a promotion decision, if a later workflow decides governance should move out
  of `blocked_deferred`;
- a bounded closeout that updates roadmap and production evidence truthfully.

## Architecture Governance Review

Recommendation: keep `WR-120` as governance/design work until the active
product design, roadmap row, and this contract have passed validation. Do not
implement product code under `WR-120`.

Owner:

- Generic UI package, document, Canonical UI IR, recipe, target profile, token,
  binding, and evidence descriptor vocabulary belongs to `domain/ui`.
- Editor workbench product semantics, view models, readiness states, routes,
  shell layout, and editor-host adapters belong to `domain/editor`.
- Concrete launch paths, app-owned session state, provider fixtures, runtime
  execution, screenshots, manifests, and platform-impossible evidence belong to
  `apps/runenwerk_editor`.
- Workspace planning metadata belongs to
  `docs-site/src/content/docs/workspace`.

Dependency direction:

```text
foundation -> domain/ui -> domain/editor -> engine/runtime -> apps/runenwerk_editor
```

Forbidden dependencies:

- `domain/ui` must not import editor shell, app provider, runtime launch,
  screenshot, native window, or game HUD implementation vocabulary.
- `domain/editor` must not own generic UI definition truth or game-runtime HUD
  semantics.
- `apps/runenwerk_editor` must not make app session state authoritative UI
  package or document truth.
- `game.runtime` compatibility descriptors must not import editor shell policy,
  workbench host state, or editor command routes.

ADR need: no ADR is required for `WR-120` while it only records governance and
preserves current ownership. Require an ADR or accepted design update before
adding a game UI owner crate, changing dependency direction, making preview or
evidence products authoritative, or moving generic UI truth into app code.

ATAM-lite:

- Quality attributes in tension: authoring ergonomics, deterministic source
  truth, native runtime proof, performance/resize responsiveness, and future
  game-runtime compatibility.
- Chosen option: keep this row as planning governance, then split product work
  into bounded follow-on rows that prove source-to-runtime behavior
  incrementally.
- Sensitivity points: standalone launch can be confused with an editor route;
  view-model panes can be mistaken for a complete catalog/canvas/inspector;
  Story Lab evidence can be mistaken for V1 package persistence; game-runtime
  descriptors can be mistaken for HUD runtime behavior.
- Risks: stale production prose could skip missing package, catalog, operation,
  persistence, performance, and game-runtime seam work.
- Non-risks: preserving `WR-114` as bounded evidence, because the new track
  explicitly does not reopen or rewrite that closeout.
- Evidence needed: a later `WR-120` closeout must link this contract,
  validation output, drift classification, follow-on candidate scopes, and
  updated production/roadmap metadata.

Migration shape: use a Strangler Fig migration for replacing current
self-authoring workbench exposure. Keep
`apps/runenwerk_editor/src/shell/providers/self_authoring.rs` as a named
compatibility path, add new UI Designer product modules beside it, route one
product workflow through the new path, prove parity and evidence, then remove
or hide legacy debug/self-authoring affordances only after guards exist.

Fitness functions required before implementation claims:

- launch guard: standalone UI Designer opens the Designer workbench, not full
  editor or Material Lab;
- source-truth guard: app session state cannot become canonical package or
  document truth;
- operation guard: visual edits produce Canonical UI IR diffs and deterministic
  textual patches;
- surface guard: normal workflows do not expose generic debug action lists;
- evidence guard: evidence packets include package id, document id, source
  version, target profile, scenario id, diagnostics, performance counters, and
  freshness;
- game-runtime seam guard: game-runtime descriptors do not import editor shell
  vocabulary;
- resize/performance guard: resize and canvas interactions do not rebuild
  unchanged catalog or diagnostics projections.

Ownership mode: stream-aligned editor product work with complicated-subsystem
support from `domain/ui`, `domain/editor`, engine runtime, and app evidence
owners.

## Code Truth And Planning Truth Drift

| Claim | Current classification | Evidence |
|---|---|---|
| Standalone UI Designer app exists | Stale or contradictory for the new product track | `apps/runenwerk_editor/src/runtime/app.rs` enum `RunenwerkRuntimeWorkbench` has `FullEditor` and `MaterialLab`; `apps/runenwerk_editor/src/bin` has `runenwerk_material_lab.rs` and no `runenwerk_ui_designer.rs`. |
| Material Lab provides the standalone launch pattern | Code-backed and evidence-backed as a pattern only | `apps/runenwerk_editor/src/runtime/app.rs` function `run_material_lab_workbench` and `apps/runenwerk_editor/src/bin/runenwerk_material_lab.rs`. |
| UI Designer workbench panes and readiness models exist | Code-backed but bounded to earlier workbench route evidence | `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module `editor_definition`, `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs` module `build_editor_lab_surface`, and the completed `WR-114` closeout. |
| Current app-visible UI Designer path is a full V1 product workbench | Design-backed only for the new product track | `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` function `ui_designer_workbench_view_model` builds a workbench view model, but V1 package, catalog, direct operation, persistence, source-versioned evidence, and performance contracts are not complete. |
| Shared component catalog workflow is complete | Design-backed only | The accepted product design defines catalog requirements; no app-hosted searchable recipe catalog with target-profile filtering is proven for V1 productization. |
| Editor tool panel workflow is complete | Design-backed only | Direct insert/move/edit through typed operations, deterministic diff, apply/reject, rollback, reload, diagnostics, and evidence remain follow-on work. |
| Game runtime compatibility is proven | Design-backed only for the new track | The accepted product design requires descriptor compatibility proof only; no game HUD runtime implementation is authorized or claimed. |
| Scenario evidence and performance baselines are complete | Code-backed only for bounded predecessor evidence, missing V1 product evidence | `WR-114` evidence proves a standalone workbench route, but not source-versioned V1 package evidence, resize baselines, or product performance counters. |
| `WR-114` / `PM-EDITOR-UX-004` may be reused as proof | Code-backed and evidence-backed only as historical bounded evidence | The completed closeout explicitly leaves graph, shell polish, all-surface migration, game UI readiness, and final no-gap certification to later milestones. |

This drift classification is the minimum planning truth for `WR-120`. Later
closeout must preserve it or replace it with fresher code-backed evidence.

## Follow-On Candidate Decomposition

These are planning candidates, not active roadmap rows. Each must pass roadmap
intake, production planning, design gates, validation, and closeout before it
can authorize product code.

| Candidate | Production milestone | Source scope | Required proof |
|---|---|---|---|
| WR-121 V1 Package Document Session And Evidence Model | `PM-UI-DESIGNER-WB-002` | `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`, future package/session/evidence design docs, and implementation-plan/closeout docs only | Accepted package, document, session, source-version, persistence, diagnostics, and evidence-packet ownership. |
| WR-122 Standalone App Shell And Embedded Host Parity | `PM-UI-DESIGNER-WB-003` | `apps/runenwerk_editor/src/runtime/app.rs`, `apps/runenwerk_editor/src/bin/runenwerk_ui_designer.rs`, `apps/runenwerk_editor/src/editor_app/state.rs`, `apps/runenwerk_editor/src/shell/workbench_host.rs`, and `domain/editor/editor_shell/src/workspace/state.rs` | Standalone launch and embedded Editor Design host consume the same app-neutral package/session/evidence model. |
| WR-123 Catalog Hierarchy Canvas Inspector V1 | `PM-UI-DESIGNER-WB-004` | New product modules under `domain/editor/editor_shell/src/surfaces/`, `domain/editor/editor_shell/src/composition/`, and `apps/runenwerk_editor/src/shell/providers/` | Catalog, hierarchy, canvas, inspector, diagnostics, and diff/review surfaces stay synchronized by source version and avoid debug action panels. |
| WR-124 Operation Diff Apply And Rollback | `PM-UI-DESIGNER-WB-005` | Operation and diff modules under `domain/ui/ui_definition`, editor operation adapters under `domain/editor`, and app command bridges under `apps/runenwerk_editor` | Insert, move, reorder, layout, token, binding, and accessibility edits produce typed operations, deterministic patches, apply/reject, undo/redo, rollback, and reload evidence. |
| WR-125 Scenario Evidence And Performance Baselines | `PM-UI-DESIGNER-WB-006` | Evidence modules under `apps/runenwerk_editor/src/shell/`, editor scenario fixtures under `domain/editor/editor_shell/src/story_lab/`, and closeout artifact folders | Source-versioned evidence packets, diagnostics snapshots, resize/canvas/catalog/diagnostics counters, and explicit capture behavior. |
| WR-126 Game Runtime Compatibility Seam | `PM-UI-DESIGNER-WB-007` | `domain/ui` target-profile descriptors, active game-runtime UI design docs, compatibility fixtures, and evidence docs | `game.runtime` descriptors prove package, recipe, fixture, view-model, intent, safe-area, input, localization, accessibility, diagnostics, and evidence compatibility without HUD runtime behavior. |
| WR-127 Runtime Proven Closeout And Handoff | `PM-UI-DESIGNER-WB-008` | Closeout reports, usage docs, examples, roadmap/production metadata, generated planning docs, and handoff notes | Runtime-proven closeout with truthful known gaps, docs, examples, downstream game-runtime UI handoff, and no perfectionist claim unless a separate audit proves no gaps. |

Common generated planning files may be touched during closeout render/check
loops, but source code scopes above must remain separated. If a future row needs
to edit another row's source scope, stop and split or re-plan before promotion.

## Implementation Scope For WR-120

Allowed under `WR-120`:

- this planning contract;
- active product-design updates that clarify governance or acceptance gates;
- roadmap/production metadata that records the track, blockers, write scopes,
  validations, dependencies, and generated planning views;
- a later bounded `WR-120` closeout proving the governance action.

Forbidden under `WR-120`:

- adding a standalone UI Designer binary;
- adding app runtime workbench variants;
- implementing package persistence, catalog, canvas, inspector, operations,
  diff/apply, scenario evidence, performance counters, or game-runtime seams;
- editing `WR-114` closeout claims except through a separate approved
  closeout-correction row;
- moving generic UI truth into app code;
- claiming `runtime_proven` or `perfectionist_verified` for this track.

## Acceptance Criteria

This contract action is complete when:

- `docs-site/src/content/docs/reports/implementation-plans/wr-120-ui-designer-workbench-product-track-governance/plan.md`
  exists with `status: active`;
- `WR-120` write scopes include this contract and the deferred-row source it
  edits;
- architecture ownership, ADR need, migration shape, fitness functions, and
  stop conditions are recorded;
- code-truth/planning-truth drift is classified without treating planning prose
  as product evidence;
- follow-on candidates are decomposed with separate source scopes;
- documentation, roadmap, production, planning, PUML, and whitespace
  validations pass or failures are reported without proceeding.

## Stop Conditions

Stop instead of promoting or implementing when:

- `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` no longer selects
  `PM-UI-DESIGNER-WB-001` as the first legal action;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-001 --roadmap WR-120`
  no longer classifies the action as design/governance work;
- active design or completed `WR-114` closeout gates fail validation;
- a future product slice requires a forbidden dependency direction;
- ownership of package, document, session, evidence, operation, or runtime
  launch truth becomes unclear;
- generated roadmap or production checks fail and the failure is not an exact
  metadata repair for `WR-120`;
- any product implementation file changes under this governance row.

## Closeout Requirements

The later `WR-120` closeout must include:

- this contract path;
- architecture-governance review summary;
- the final drift classification or a fresher replacement;
- final follow-on candidate rows or explicit deferral notes;
- validation transcript summary for production, roadmap, planning, docs, PUML,
  and whitespace checks;
- updated `WR-120` roadmap state and production milestone evidence only when
  gates are true;
- a rerun of `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` proving the
  next legal action after governance closeout.

Expected completion quality for `WR-120`: `bounded_contract` or
`not_applicable`. `runtime_proven` and `perfectionist_verified` are forbidden
for this governance-only row.

## Perfectionist Closeout Audit

No perfectionist audit is intended for `WR-120`. The row must keep known gaps
visible:

- no standalone app shell;
- no V1 package/document/session implementation;
- no product catalog, hierarchy, canvas, inspector, diagnostics, or review
  surface implementation;
- no operation/diff/apply/rollback implementation;
- no scenario evidence or performance baseline implementation;
- no game-runtime compatibility seam proof;
- no runtime-proven product closeout.

Any later no-gap claim belongs to a separate completed audit path after all
`PM-UI-DESIGNER-WB-002` through `PM-UI-DESIGNER-WB-008` evidence gates are
satisfied.
