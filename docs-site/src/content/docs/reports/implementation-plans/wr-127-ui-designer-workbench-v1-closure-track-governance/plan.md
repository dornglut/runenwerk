---
title: WR-127 UI Designer Workbench V1 Closure Track Governance Contract
description: Design-first governance contract for PM-UI-DESIGNER-WB-V1-CLOSURE-001, reconciling the accepted V1 product contract, completed productization overclaim, current code truth, and follow-on closure scope before product implementation resumes.
status: active
owner: editor
layer: workspace / domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md
  - ../../../design/accepted/ui-designer-production-readiness-and-evidence-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
related_reports:
  - ../../closeouts/pm-ui-designer-wb-008-runtime-proven-closeout-and-handoff/closeout.md
  - ../../roadmap-intake/2026-05-26-add-a-ui-designer-workbench-v1-closure-p/proposal.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# WR-127 UI Designer Workbench V1 Closure Track Governance Contract

## Goal

Define the design-first governance contract for
`PM-UI-DESIGNER-WB-V1-CLOSURE-001` and `WR-127` before any V1 closure product
implementation starts.

This contract is produced from:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-001 --roadmap WR-127
```

The command currently classifies the next action as `design_first`. This action
does not authorize product code. It records the closure governance boundary,
drift findings, owner boundaries, follow-on slice decomposition, validation
gates, and closeout requirements needed before a later coordinator run can
decide whether `WR-127` or follow-on closure rows are ready for promotion or
completion.

Expected governance outcome:

- the accepted UI Designer Workbench Product Design remains the product
  contract for the V1 workflow closure;
- the completed `PT-UI-DESIGNER-WORKBENCH` closeout remains historical
  productization evidence, but it is not sufficient proof of the full accepted
  V1 workflow depth;
- generic UI definition truth remains in `domain/ui`;
- editor/workbench adapters, app-neutral view models, surface semantics, routes,
  and shell composition remain in `domain/editor`;
- app-owned session state, runtime launch, provider wiring, evidence capture,
  and concrete user-facing behavior remain in `apps/runenwerk_editor`;
- game-runtime compatibility is proven only through target-profile
  descriptors, fixture packets, bindings, intents, and evidence workflow;
- concrete game HUD runtime behavior remains downstream of
  `PT-GAME-RUNTIME-UI`;
- follow-on closure work is split into bounded rows with separate scopes,
  design gates, validation, and closeout evidence.

## Source Of Truth

- Production track and milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml` milestone
  `PM-UI-DESIGNER-WB-V1-CLOSURE-001`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` item `WR-127`.
- Accepted product contract:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Completed productization closeout being reconciled:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-008-runtime-proven-closeout-and-handoff/closeout.md`.
- Current standalone launch code truth:
  `apps/runenwerk_editor/src/runtime/app.rs` enum
  `RunenwerkRuntimeWorkbench`, functions
  `build_ui_designer_workbench_headless_app` and
  `run_ui_designer_workbench`, and
  `apps/runenwerk_editor/src/bin/runenwerk_ui_designer.rs`.
- Current app-hosted workbench state and host composition truth:
  `apps/runenwerk_editor/src/shell/workbench_host.rs` module
  `workbench_host`, `apps/runenwerk_editor/src/runtime/resources.rs` module
  `resources`, and `apps/runenwerk_editor/src/editor_app/state.rs` module
  `state`.
- Current UI Designer provider truth:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` function
  `ui_designer_workbench_view_model` and related pane builders.
- Current editor workbench view-model truth:
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`.
- Current editor composition truth:
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  function `build_ui_designer_workbench`.
- Current app-owned evidence vocabulary truth:
  `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs` module
  `editor_lab_evidence`.

## Readiness

`task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-001 --roadmap WR-127`
reports:

```text
Production milestone state: designing
Roadmap planning_state: blocked_deferred
Roadmap blocker: B5
Roadmap dependencies: WR-126:completed
Next action: design_first
Contract target: docs-site/src/content/docs/reports/implementation-plans/wr-127-ui-designer-workbench-v1-closure-track-governance/plan.md
```

`WR-127` is not ready for product implementation. Before any closure
implementation row can start, the closure track still needs:

- this active planning contract;
- architecture-governance review evidence;
- explicit drift findings against the accepted V1 workflows;
- disjoint follow-on implementation candidates with source scopes,
  validations, closeout paths, and stop conditions;
- a bounded closeout that updates roadmap and production evidence truthfully;
- a rerun of `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` proving
  the next legal action.

## Architecture Governance Review

Recommendation: keep `WR-127` as governance/design work until this contract,
the accepted product design, and the closure track metadata have passed
validation. Do not implement product code under `WR-127`.

Owner:

- Generic UI package, document, Canonical UI IR, recipe, target profile, token,
  binding, fixture, persistence, activation, and readiness vocabulary belongs
  to `domain/ui` and `domain/ui/ui_theme`.
- Editor workbench product semantics, app-neutral view models, readiness
  states, routes, surface composition, and shell adapters belong to
  `domain/editor/editor_definition` and `domain/editor/editor_shell`.
- Concrete launch paths, app-owned session state, provider fixtures, runtime
  execution, screenshots, manifests, performance captures, and
  platform-impossible evidence belong to `apps/runenwerk_editor`.
- Workspace planning metadata belongs to
  `docs-site/src/content/docs/workspace`.
- Game-runtime HUD behavior belongs outside this closure track and remains
  downstream of `PT-GAME-RUNTIME-UI`.

Dependency direction:

```text
foundation -> domain/ui -> domain/editor -> engine/runtime -> apps/runenwerk_editor
```

Forbidden dependencies:

- `domain/ui` must not import editor shell, app provider, runtime launch,
  screenshot, native window, or game HUD implementation vocabulary.
- `domain/editor` must not own generic UI definition truth or concrete
  game-runtime HUD behavior.
- `apps/runenwerk_editor` must not make provider cache, runtime resources, or
  app session state the canonical UI package/document truth.
- game-runtime compatibility descriptors must not import editor shell policy,
  workbench host state, or editor command routes.

ADR need: no ADR is required for this governance contract while it preserves
current ownership and dependency direction. Require an ADR or accepted design
update before adding a game UI owner crate, changing dependency direction,
making preview or evidence products authoritative, or moving generic UI truth
into app code.

ATAM-lite:

- Quality attributes in tension: authoring ergonomics, deterministic source
  truth, direct manipulation, typed diagnostics, runtime evidence, performance,
  and future game-runtime compatibility.
- Chosen option: keep this row as planning governance, then split product
  closure into bounded follow-on rows that prove source-versioned behavior
  incrementally.
- Sensitivity points: standalone launch can be mistaken for product closure;
  pane/view-model presence can be mistaken for full catalog/canvas/inspector
  workflows; retained evidence can be mistaken for source-versioned scenario
  proof; game-runtime descriptors can be mistaken for HUD runtime behavior.
- Risks: status-panel-only, descriptor-only, prepared-data-only, or
  closeout-only evidence could repeat the overclaim this track exists to fix.
- Non-risks: preserving the completed productization closeouts as historical
  evidence, because this closure track explicitly adds stricter V1 workflow
  gates instead of rewriting history.
- Evidence needed: a later `WR-127` closeout must link this contract,
  validation output, drift classification, follow-on candidate scopes, and
  updated production/roadmap metadata.

Migration shape: use Strangler Fig only for replacing or hiding older
self-authoring/debug exposure. Keep the existing self-authoring provider path
as a compatibility boundary, add product-specific package/session/catalog/
operation/evidence modules beside it, route one author workflow through the
new path, prove parity and source-versioned evidence, then remove or hide
legacy status/debug affordances only after guards exist.

Fitness functions required before implementation claims:

- source-truth guard: package/document ids, schema versions, source versions,
  draft snapshots, applied snapshots, and rollback points remain explicit and
  reconstructable;
- catalog guard: recipe insertion comes from typed recipe declarations, target
  profile compatibility, slot diagnostics, token requirements, and
  accessibility requirements;
- surface guard: hierarchy, canvas, inspector, diagnostics, and diff surfaces
  project the same source version;
- operation guard: visual edits produce typed operation reports,
  deterministic diffs, fail-closed diagnostics, history, undo/redo, reload, and
  rollback evidence;
- evidence guard: packets include package id, document id, source version,
  target profile, scenario id, diagnostics, performance counters, artifact
  freshness, and unsupported reasons;
- game-runtime seam guard: game-runtime descriptors, fixtures, bindings, and
  intents do not import editor shell vocabulary or implement HUD runtime
  behavior;
- performance guard: resize, canvas selection, catalog projection, diagnostics
  projection, and evidence capture are measured through product paths.

Ownership mode: stream-aligned editor product work with complicated-subsystem
support from `domain/ui`, `domain/editor`, engine runtime, and app evidence
owners.

## Drift Findings

The completed UI Designer Workbench productization track made a
`runtime_proven` claim. The closure track treats that claim as historical
evidence with known gaps until current code and runtime artifacts prove the
accepted V1 workflow depth below.

| Accepted V1 workflow claim | Current classification | Required closure proof |
|---|---|---|
| One editable package/document/session source truth drives standalone and embedded hosts | Code-backed in parts, missing closure-grade proof | Package id, document id, schema version, source version, draft/applied snapshots, rollback points, reload behavior, and diagnostics must be source-versioned and reconstructable. |
| Standalone UI Designer launch and embedded Editor Design host use the same product contract | Code-backed in parts | Standalone binary and host composition must prove the same package/session/evidence model as embedded `Editor Design`. |
| Recipe catalog insertion is a normal author workflow | Design-backed or status-backed until proven | Searchable compatible recipe entries, slot/token/state/accessibility requirements, insertion reports, and disabled reasons must drive source-versioned document changes. |
| Hierarchy, canvas, inspector, diagnostics, and diff panes stay synchronized | Code-backed in parts, missing closure-grade evidence | All visible panes must project the same source version and expose typed diagnostics before dispatch. |
| Visual operations provide deterministic diff/apply/reject/undo/redo/reload/rollback parity | Code-backed in parts, missing closure-grade evidence | Every supported gesture must produce typed operation reports, deterministic diffs, fail-closed diagnostics, history, and recovery evidence. |
| Scenario matrix and evidence packets prove editor.workbench and game.runtime compatibility | Code-backed in parts, missing closure-grade evidence | Scenario packets must include target profile, source package provenance, diagnostics, performance descriptors, artifact freshness, and explicit unsupported reasons. |
| Performance baselines prove product paths, not synthetic summaries | Missing closure-grade proof | Baselines must measure frame build, canvas projection, catalog projection, diagnostics projection, resize relayout, scenario preview, and cache counters. |
| game.runtime is compatible without implementing HUD runtime behavior | Design-backed until proven | Compatibility proof must stay descriptor/fixture/binding/intent/evidence-only and hand concrete HUD runtime behavior to `PT-GAME-RUNTIME-UI`. |
| Runtime-proven closeout is honest and has known gaps | Not yet proven for the closure track | Final closure must link completed evidence for every accepted V1 workflow and list downstream gaps without claiming no-gap certification. |

This drift matrix is the minimum planning truth for `WR-127`. Later closeout
must preserve it or replace it with fresher code-backed and runtime-backed
evidence.

## Follow-On Candidate Decomposition

These are closure candidates, not active roadmap rows. Each must pass roadmap
intake or promotion, production planning, design gates, validation, and
closeout before it can authorize product code.

| Candidate | Production milestone | Source scope | Required proof |
|---|---|---|---|
| WR-next Package Session Source Truth Closure | `PM-UI-DESIGNER-WB-V1-CLOSURE-002` | `domain/ui/ui_definition`, `domain/editor/editor_definition`, app session modules under `apps/runenwerk_editor/src/shell/`, persistence/evidence docs, implementation-plan and closeout docs | Package/document ids, schema versions, source versions, draft/applied/rollback snapshots, reload, diagnostics, and reconstructable app session state. |
| WR-next Recipe Catalog Insertion And Authoring Surface Closure | `PM-UI-DESIGNER-WB-V1-CLOSURE-003` | `domain/ui` recipe contracts, `domain/editor/editor_shell/src/surfaces/`, `domain/editor/editor_shell/src/composition/`, `apps/runenwerk_editor/src/shell/providers/` | Searchable compatible recipes, insertion, hierarchy, canvas, inspector, diagnostics, and diff projections over one source version. |
| WR-next Operation Diff Apply Rollback Parity Closure | `PM-UI-DESIGNER-WB-V1-CLOSURE-004` | Operation/diff modules in `domain/ui/ui_definition`, editor operation adapters in `domain/editor`, app command bridges in `apps/runenwerk_editor` | Typed operation reports, deterministic diffs, apply/reject, undo/redo, reload, rollback, fail-closed diagnostics, and recovery evidence. |
| WR-next Scenario Matrix Game Runtime And Evidence Closure | `PM-UI-DESIGNER-WB-V1-CLOSURE-005` | Evidence modules under `apps/runenwerk_editor/src/shell/`, target-profile fixtures in `domain/ui` and `domain/editor`, performance/evidence artifact folders, active game-runtime UI design docs | editor.workbench and game.runtime scenario packets, read-only fixtures, validated intents, diagnostics snapshots, artifact freshness, unsupported reasons, and measured performance baselines. |
| WR-next Runtime-Proven Product Closeout And Handoff | `PM-UI-DESIGNER-WB-V1-CLOSURE-006` | Closeout reports, app usage docs, examples, roadmap/production metadata, generated planning docs, and handoff notes | Runtime-proven closure with completed workflow evidence, truthful known gaps, downstream `PT-GAME-RUNTIME-UI` handoff, and no perfectionist claim unless a separate completed audit proves no gaps. |

Common generated planning files may be touched during render/check loops, but
source code scopes above must remain separated. If a future row needs to edit
another row's source scope, stop and split or re-plan before promotion.

## Implementation Scope For WR-127

Allowed under `WR-127`:

- this planning contract;
- design or planning updates that clarify V1 closure governance and acceptance
  gates;
- roadmap/production metadata that records blockers, write scopes,
  validations, dependencies, evidence gates, and generated planning views;
- a later bounded closeout proving the governance action.

Forbidden under `WR-127`:

- adding or changing product runtime code;
- changing package persistence, catalog insertion, canvas, inspector,
  operations, diff/apply, scenario evidence, performance counters, or
  game-runtime compatibility implementation;
- rewriting the completed `PT-UI-DESIGNER-WORKBENCH` closeouts except through a
  separate approved correction row;
- moving generic UI truth into app code;
- claiming `runtime_proven` or `perfectionist_verified` for the governance row.

## Acceptance Criteria

This contract action is complete when:

- `docs-site/src/content/docs/reports/implementation-plans/wr-127-ui-designer-workbench-v1-closure-track-governance/plan.md`
  exists with `status: active`;
- `WR-127` write scopes include this contract and the deferred-row source it
  edits;
- architecture ownership, ADR need, migration shape, fitness functions, and
  stop conditions are recorded;
- drift findings name each accepted V1 workflow claim that is not yet proven by
  code and runtime evidence;
- follow-on closure candidates are decomposed with separate source scopes;
- documentation, roadmap, production, planning, PUML, and whitespace
  validations pass or failures are reported without proceeding.

## Stop Conditions

Stop instead of promoting or implementing when:

- `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` no longer selects
  `PM-UI-DESIGNER-WB-V1-CLOSURE-001` as the first legal action;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-001 --roadmap WR-127`
  no longer classifies the action as design/governance work;
- accepted product design or completed `PM-UI-DESIGNER-WB-008` closeout gates
  fail validation;
- a future product slice requires forbidden dependency direction;
- ownership of package, document, session, evidence, operation, runtime launch,
  or game-runtime compatibility truth becomes unclear;
- generated roadmap or production checks fail and the failure is not an exact
  metadata repair for `WR-127`;
- any product implementation file changes under this governance row.

## Closeout Requirements

The later `WR-127` closeout must include:

- this contract path;
- architecture-governance review summary;
- final drift classification or a fresher replacement;
- final follow-on candidate rows or explicit deferral notes;
- validation transcript summary for docs, production, roadmap, planning, PUML,
  and whitespace checks;
- updated `WR-127` roadmap state and production milestone evidence only when
  gates are true;
- a rerun of `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` proving the
  next legal action after governance closeout.

Expected completion quality for `WR-127`: `bounded_contract` or
`not_applicable`. `runtime_proven` and `perfectionist_verified` are forbidden
for this governance-only row.

## Perfectionist Closeout Audit

No perfectionist audit is intended for `WR-127`. The row must keep known gaps
visible:

- package/session source truth closure remains incomplete;
- recipe catalog insertion and authoring surface closure remains incomplete;
- operation diff/apply/rollback parity closure remains incomplete;
- scenario matrix, game-runtime compatibility workflow, evidence packets, and
  performance baseline closure remain incomplete;
- runtime-proven product closeout and handoff remain incomplete;
- concrete game HUD runtime behavior belongs to `PT-GAME-RUNTIME-UI`;
- no no-gap certification is claimed.

Any later no-gap claim belongs to a separate completed audit path after all
`PM-UI-DESIGNER-WB-V1-CLOSURE-002` through
`PM-UI-DESIGNER-WB-V1-CLOSURE-006` evidence gates are satisfied.
