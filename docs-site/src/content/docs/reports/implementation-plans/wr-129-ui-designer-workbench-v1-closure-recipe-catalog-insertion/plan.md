---
title: WR-129 UI Designer Workbench V1 Closure Recipe Catalog Insertion Contract
description: Design-first implementation contract for PM-UI-DESIGNER-WB-V1-CLOSURE-003 recipe catalog insertion, source-versioned authoring surfaces, typed diagnostics, and deterministic diff projection.
status: active
owner: editor
layer: domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-workbench-product-design.md
related_reports:
  - ../wr-127-ui-designer-workbench-v1-closure-track-governance/plan.md
  - ../wr-128-ui-designer-workbench-v1-closure-package-session-source-truth/plan.md
  - ../../closeouts/pm-ui-designer-wb-v1-closure-002-package-session-source-truth/closeout.md
  - ../../roadmap-intake/2026-05-26-ui-designer-workbench-v1-closure-recipe-/proposal.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# WR-129 UI Designer Workbench V1 Closure Recipe Catalog Insertion Contract

## Goal

Define the decision-complete design-first contract for
`PM-UI-DESIGNER-WB-V1-CLOSURE-003` and `WR-129` before any recipe insertion
product code starts.

This contract is produced from:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-003 --roadmap WR-129
```

The command currently classifies the next action as `design_first`:

```text
Production milestone state: designing
Roadmap planning_state: blocked_deferred
Roadmap blocker: B5
Roadmap dependencies: WR-123:completed, WR-128:completed
Milestone links WR item: yes
Next action: design_first
```

This action does not authorize product code. It records the bounded
source-to-workbench chain needed before `WR-129` can be promoted for
implementation.

After the design-first contract and WR metadata were accepted, the current
readiness report is:

```text
Production milestone state: ready_next
Roadmap planning_state: ready_next
Roadmap blocker: B2
Roadmap dependencies: WR-123:completed, WR-128:completed
Milestone links WR item: yes
Next action: write_promotion_contract
Promotion preflight: promotable
Suggested command: task roadmap:promote -- --id WR-129 --state current_candidate --evidence "<accepted evidence>"
```

`WR-129` is ready for promotion only as the PM-003 recipe insertion and
authoring-surface closure row. Before product code starts, the coordinator must
run the suggested promotion command with accepted evidence, rerun
`task ai:goal`, and then follow the implementation action reported for the
current state.

Accepted promotion evidence:

```text
Accepted PM-UI-DESIGNER-WB-V1-CLOSURE-003 design-first recipe insertion contract at docs-site/src/content/docs/reports/implementation-plans/wr-129-ui-designer-workbench-v1-closure-recipe-catalog-insertion/plan.md; WR-123 and WR-128 prerequisites are completed, design gates are accepted, write scopes are bounded, and production:plan reports WR-129 promotable.
```

Promotion must not imply full operation parity, scenario evidence, performance
baselines, final closeout, or concrete game HUD runtime behavior.

After promotion, the implementation-readiness report is:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-123:completed, WR-128:completed
Milestone links WR item: yes
Next action: write_implementation_contract
```

This document is the decision-complete implementation contract for the
current-candidate row. The next coding pass may implement only the PM-003
recipe catalog insertion and source-versioned authoring-surface slice
described here, then must run focused tests, close out PM-003, archive WR-129,
and rerun `task ai:goal` before any PM-004 work.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml` milestone
  `PM-UI-DESIGNER-WB-V1-CLOSURE-003`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` item `WR-129`.
- Completed prerequisites:
  `WR-123` product-surface projection evidence and `WR-128`
  package/session source-truth evidence.
- Accepted recipe source truth:
  `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe`, especially `UiRecipeLibrary`,
  `UiRecipeDeclaration`, `UiRecipeExpansionRequest`,
  `UiRecipeExpansionReport`, and `expand_ui_recipe`.
- App-neutral authoring surface contracts:
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`, especially `EditorDefinitionSurfaceAction`,
  `UiDesignerWorkbenchViewModel`, `UiDesignerWorkbenchPaneViewModel`, and
  `EditorLabCatalogItemViewModel`.
- App-neutral retained composition:
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`, especially
  `build_ui_designer_workbench` and `push_catalog_items`.
- App provider projection:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`, especially `ui_designer_workbench_view_model`,
  `token_recipe_preview_pane`, `component_catalog_items`,
  `ui_hierarchy_view_model`, `canvas_preview_view_model`, and
  `style_inspector_view_model`.
- App-owned source-versioned session:
  `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`, especially `SelfAuthoringWorkspaceState` and
  `apply_editor_lab_operation`.

## Ownership And Invariants

Owners:

- `domain/ui/ui_definition` owns generic recipe declaration, target-profile
  compatibility, slot compatibility, deterministic expansion into Canonical UI
  IR nodes, stable authored ids, and typed recipe diagnostics.
- `domain/editor/editor_shell` owns app-neutral catalog, hierarchy, canvas,
  inspector, diagnostics, and diff projection view models.
- `apps/runenwerk_editor` owns concrete workbench provider state, selected
  document/node session state, author workflow orchestration, and evidence
  capture.

Invariants:

- A recipe declaration is source truth only in `domain/ui/ui_definition`.
- Catalog rows in `domain/editor` and `apps/runenwerk_editor` are projections
  of recipe contracts plus current workbench selection and target profile.
- Compatible insertion expands a recipe through `expand_ui_recipe` before it
  mutates draft package state.
- Inserted nodes receive stable authored ids and update the selected
  source-versioned package/document path.
- Hierarchy, canvas, inspector, diagnostics, and diff preview project the same
  source version after insertion.
- Invalid insertion preserves current draft state, records typed diagnostics,
  and does not enter operation history.

## Implementation Scope

Allowed for a later promoted `WR-129` implementation:

- add a typed recipe-insert action to
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` if existing
  `EditorDefinitionSurfaceAction` variants cannot express recipe insertion
  without stringly fallback;
- project catalog rows from real `UiRecipeDeclaration` metadata, including
  target profile, slot compatibility, token families, supported states,
  accessibility requirements, provenance, readiness, and disabled reasons;
- route compatible insertions through `UiRecipeExpansionRequest::activate` and
  `expand_ui_recipe`;
- update `SelfAuthoringWorkspaceState` through the same source-versioned draft
  package/document path used by PM-002;
- refresh hierarchy, canvas, inspector, diagnostics, and diff preview from the
  updated source version;
- add focused tests for compatible insertion, incompatible target profiles,
  preview-only activation blocking, slot diagnostics, stable ids, and surface
  projection parity.

Forbidden under `WR-129`:

- full operation apply/rollback parity, undo/redo policy, or complete
  operation taxonomy; that remains `PM-UI-DESIGNER-WB-V1-CLOSURE-004`;
- scenario matrix, game.runtime compatibility evidence, native evidence
  freshness, or performance baselines; that remains
  `PM-UI-DESIGNER-WB-V1-CLOSURE-005`;
- final runtime-proven product closeout and handoff; that remains
  `PM-UI-DESIGNER-WB-V1-CLOSURE-006`;
- moving generic recipe, Canonical UI IR, token, or document source truth into
  `apps/runenwerk_editor`;
- implementing concrete game HUD runtime behavior.

## Implementation Steps

1. Inspect the current recipe contracts in
   `domain/ui/ui_definition/src/component_recipe/mod.rs` and existing focused
   tests for `expand_ui_recipe`.
2. Inspect current catalog projection in
   `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
   `component_catalog_items` and confirm which rows are hard-coded status
   projection versus real recipe metadata.
3. Decide whether `EditorDefinitionSurfaceAction` needs a dedicated
   `InsertRecipe` action carrying recipe id, target profile, document id, and
   insertion slot/node context.
4. Add or refine app-neutral view-model fields only in
   `domain/editor/editor_shell/src/surfaces/editor_definition.rs` when the
   catalog row or insertion result cannot be expressed by the current typed
   contracts.
5. Implement insertion in `SelfAuthoringWorkspaceState` through a typed method
   that expands a recipe, validates diagnostics, updates draft package state,
   and records the new source version.
6. Project post-insertion hierarchy, canvas, inspector, diagnostics, and diff
   preview from the updated package/document state.
7. Keep any deterministic diff preview bounded to proving changed source
   version and authored ids; full apply/reject/rollback parity is deferred to
   PM-004.
8. Add focused tests in the owning crates and app provider tests before any
   roadmap closeout.

## Validation

Focused validation for a later implementation:

```text
cargo test -p ui_definition component_recipe
cargo test -p editor_shell ui_designer
cargo test -p editor_shell editor_lab
cargo test -p runenwerk_editor ui_designer
cargo test -p runenwerk_editor self_authoring
```

Planning and closeout validation:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-003 --roadmap WR-129
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
task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE
```

Run `./quiet_full_gate.sh` only if implementation expands beyond the bounded
recipe insertion and authoring-surface closure slice.

## Acceptance Criteria

This design-first contract action is complete when:

- this file exists with `status: active`;
- `WR-129` write scopes include this contract path;
- `PM-UI-DESIGNER-WB-V1-CLOSURE-003` links `WR-129`;
- accepted design gates, owner boundaries, non-goals, validation, and closeout
  requirements are recorded;
- roadmap, production, docs, planning, PUML, and whitespace validations pass.

A later implementation closeout is complete only when:

- catalog entries come from recipe contracts rather than hard-coded status rows;
- compatible insertion expands through `domain/ui` recipe contracts into the
  active package/document state;
- unsupported recipes and slots produce typed diagnostics and preserve draft
  state;
- hierarchy, canvas, inspector, diagnostics, and diff preview expose the same
  source version after insertion;
- focused tests prove normal author insertion workflow evidence;
- known gaps for PM-004, PM-005, PM-006, and PT-GAME-RUNTIME-UI remain explicit.

## Stop Conditions

Stop before product code changes if:

- `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` no longer selects
  PM-003 / WR-129 work;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-003 --roadmap WR-129`
  reports anything other than the expected promotion or implementation action;
- insertion requires making app provider state generic recipe source truth;
- insertion cannot preserve stable authored ids or deterministic source-version
  projection;
- a requested behavior belongs to operation parity, scenario evidence,
  performance, final closeout, or concrete game-runtime HUD implementation.

## Closeout Requirements

The later closeout path must be:

```text
docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-v1-closure-003-recipe-catalog-insertion-and-authoring-surface/closeout.md
```

Closeout must update `production-tracks.yaml`, move `WR-129` to the roadmap
archive as completed, include focused validation output, list remaining
quality gaps, and rerun:

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
task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE
```

Expected completion quality is `runtime_proven` only if the normal author
workflow proves source-versioned insertion and synchronized authoring surfaces.
Use `bounded_contract` if the implementation only accepts the design contract.

## Perfectionist Closeout Audit

No perfectionist audit is intended for `WR-129`. The row must keep these known
quality gaps visible until downstream milestones complete:

- PM-004 owns full operation diff/apply/rollback parity.
- PM-005 owns scenario matrix, game.runtime compatibility workflow, evidence
  packets, and performance baselines.
- PM-006 owns runtime-proven final product closeout and handoff.
- Concrete game HUD runtime behavior remains downstream of
  PT-GAME-RUNTIME-UI.
