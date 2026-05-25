---
title: WR-114 Standalone UI Designer Workbench Contract
description: Current-candidate implementation contract for PM-EDITOR-UX-004 standalone UI Designer workbench over editor-owned product contracts, generic UI Designer source truth, and app-owned native evidence.
status: active
owner: editor
layer: domain/editor / app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md
  - ../../../design/active/ui-designer-interface-lab-platform-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../../../design/accepted/ui-designer-production-readiness-and-evidence-design.md
related_reports:
  - ../wr-113-layered-editor-design-system-migration/plan.md
  - ../../closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-114 Standalone UI Designer Workbench Contract

## Goal

Define the decision-complete implementation contract for `PM-EDITOR-UX-004`
now that `WR-114` is the selected `current_candidate` roadmap row for the
standalone UI Designer workbench.

This contract is produced from:

```text
task production:plan -- --milestone PM-EDITOR-UX-004 --roadmap WR-114
```

It clears the implementation-contract action by naming the source-truth chain,
owners, typed contracts, non-goals, migration path away from legacy
self-authoring panels, fitness functions, validation, and closeout bar for the
next bounded implementation slice. This workflow action does not implement
product code. Product code may start only after this contract validates and a
later coordinator run continues the legal `WR-114` implementation action.

Expected production outcome:

- authors use a real workbench surface with canvas, hierarchy, inspector,
  properties, diagnostics, preview, token/recipe/binding views, scenario
  matrices, and readiness evidence;
- generic UI Designer source truth remains in `domain/ui` contracts;
- editor workbench product semantics remain in `domain/editor`;
- native execution, provider fixtures, screenshots or platform-impossible
  reports, and evidence manifests remain app-owned in `apps/runenwerk_editor`;
- legacy text/action self-authoring controls remain temporary compatibility
  paths and cannot masquerade as the standalone UI Designer product.

## Source Of Truth

- Production milestone: `PM-EDITOR-UX-004` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-114` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Active editor UX doctrine:
  `docs-site/src/content/docs/design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md`.
- Active UI Designer platform doctrine:
  `docs-site/src/content/docs/design/active/ui-designer-interface-lab-platform-design.md`.
- Accepted Canonical UI IR design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`.
- Accepted visual layout design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`.
- Accepted binding design:
  `docs-site/src/content/docs/design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md`.
- Accepted fixture and scenario design:
  `docs-site/src/content/docs/design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md`.
- Accepted readiness/evidence design:
  `docs-site/src/content/docs/design/accepted/ui-designer-production-readiness-and-evidence-design.md`.
- Completed prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md`.
- Editor shell product entrypoints:
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`,
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`, and
  `domain/editor/editor_shell/src/workspace/profile.rs` module `profile`.
- App compatibility entrypoints:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`,
  `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`, and
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/mod.rs` module
  `editor_ux_story_lab`.

## Readiness

`task production:plan -- --milestone PM-EDITOR-UX-004 --roadmap WR-114`
currently reports:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-113:completed
Next action: write_implementation_contract
```

The earlier design-first and promotion blockers are cleared because:

- `WR-113` is completed with `runtime_proven` design-system closeout evidence;
- `PM-EDITOR-UX-003` has a migrated token/recipe/state Story Lab path with
  app-owned manifest proof;
- the active UI Designer platform design defines source-truth boundaries for
  generic UI/interface definitions;
- completed Editor Lab V1 evidence provides a compatibility substrate, but does
  not itself satisfy the standalone UI Designer workbench product milestone;
- `WR-114` has been promoted to `current_candidate` with accepted promotion
  evidence and still depends only on completed `WR-113`.

This action is still contract-only. The next coordinator run must rerun
`task ai:goal -- --track PT-EDITOR-UX` after validation before making product
code changes.

## Architecture Governance Review

Recommendation: execute a bounded vertical workbench implementation only after
this contract validates and `task ai:goal -- --track PT-EDITOR-UX` still
selects `PM-EDITOR-UX-004` and `WR-114`.

Scope:

- `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`;
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`;
- `domain/editor/editor_shell/src/workspace/profile.rs` module `profile`;
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`;
- `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`;
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/mod.rs` module
  `editor_ux_story_lab`.

Owner:

- Generic UI Designer definitions, recipes, bindings, target profiles,
  readiness descriptors, and visual-layout operations belong to `domain/ui`.
- Editor workbench product semantics, view models, routes, command intents, and
  surface readiness belong to `domain/editor`.
- Native execution evidence, provider behavior, screenshot/platform-impossible
  artifacts, and app command bridging belong to `apps/runenwerk_editor`.

Dependency direction:

```text
domain/ui -> domain/editor -> apps/runenwerk_editor
```

`domain/ui` must not import editor shell, app providers, screenshots, runtime
sessions, renderer handles, or editor command vocabulary. `domain/editor` may
adapt generic UI Designer contracts into editor workbench product patterns.
`apps/runenwerk_editor` may host and prove the workbench, but it must not own
canonical UI Designer truth.

ADR need: no new ADR is required while implementation preserves accepted UI
Designer and editor UX source-truth boundaries. Require an ADR or accepted
design update before making app-owned Designer project/session state canonical,
moving generic UI Designer source truth into editor shell, changing dependency
direction, or making future game-runtime UI depend on editor shell vocabulary.

ATAM-lite:

- Quality attributes in tension: authoring ergonomics, source-truth integrity,
  native evidence, migration safety, and future `game.runtime` compatibility.
- Chosen option: build a standalone editor-owned workbench shell over existing
  generic UI Designer contracts and retire legacy self-authoring controls by
  strangling them behind named compatibility adapters.
- Sensitivity points: canvas-only demos without operation round-trips,
  hierarchy/inspector status panels without typed edit intents, app-only
  project state becoming canonical, and evidence that proves retained debug
  output without native workbench scenarios.
- Risk: a surface can look like a Designer while still routing through old
  text/action panels.
- Non-risk: keeping native evidence app-owned, because the active editor UX
  doctrine assigns evidence execution to the app.

Migration shape: use a Strangler Fig migration. Keep current Editor Lab and
self-authoring paths as compatibility inputs, introduce a standalone Designer
workbench surface with typed canvas/hierarchy/inspector/properties contracts,
prove the normal authoring chain in Story Lab, then hide or remove legacy
text/action affordances only after parity, rollback, and evidence exist.

Fitness functions:

- editor shell tests for canvas, hierarchy, inspector, property panel,
  diagnostics, route, focus, keyboard, split/dock, and scenario matrix
  contracts;
- app tests for `ui_designer` native evidence, provider fixture execution,
  screenshot or typed platform-impossible reports, and manifest validation;
- guard tests that fail if normal UI Designer workflows route through generic
  self-authoring action lists;
- planning validators for docs, roadmap, production, and PUML after metadata
  changes.

Ownership mode: stream-aligned editor workbench product work with
complicated-subsystem support from generic UI Designer contracts and app
evidence owners.

## Critical Review Gate

Source truth:

- Generic UI Designer source truth is in `domain/ui` accepted designs and
  contracts. UI IR, recipe declarations, token graphs, target profiles,
  binding descriptors, visual-layout operations, and readiness descriptors are
  source contracts.
- Editor workbench product truth is in `domain/editor/editor_shell`, especially
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`,
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  function `build_editor_lab_surface`, and
  `domain/editor/editor_shell/src/workspace/profile.rs` module `profile`.
  These own editor workbench view models, routes, surface mounting, and product
  semantics.
- App execution truth is in `apps/runenwerk_editor`, especially
  `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`,
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`, and
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab` module
  `editor_ux_story_lab`. These own IO, command dispatch, provider frames,
  screenshot or platform-impossible artifacts, and evidence manifests.
- Retained debug output, screenshots, provider snapshots, prepared view models,
  app project packages, and evidence manifests are runtime products or proof,
  not generic UI Designer source truth.

Required source-to-runtime chain:

```text
Generic UI Designer contracts
  -> editor-owned workbench view model and route contracts
  -> typed canvas, hierarchy, inspector, property, diagnostics, preview, and scenario panes
  -> app-owned command bridge and provider frame execution
  -> Editor UX Story Lab story/state matrix execution
  -> native screenshot or typed platform-impossible artifact
  -> accessibility, interaction, diagnostics, performance, and readiness manifest evidence
```

The implementation must not stop at descriptor registration, retained preview
labels, status panes, prepared data, or generic action lists. A workbench path is
not complete until tests prove the normal authoring flow consumes typed
workbench view models and operation intents, routes through app-owned command
bridges, and emits Story Lab manifest evidence for the standalone workbench
scenario or an explicit platform-impossible diagnostic.

Typed contracts that must be used or extended before introducing new strings or
ad hoc maps:

- `EditorLabSurfaceViewModel` and related workbench view-model structs in
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs`;
- `EditorDefinitionSurfaceAction` and `ShellCommand::ApplyEditorLabOperation`
  command boundaries for operation dispatch;
- `EditorLabOperation`, `EditorLabOperationKind`,
  `EditorLabOperationStatus`, and operation diff families in
  `domain/editor/editor_definition`;
- `EditorUxStory`, `EditorUxScenarioMatrix`, readiness descriptors, and
  design-system evidence metadata in `domain/editor/editor_shell/src/story_lab`;
- `EditorUxEvidenceManifest`, `EditorUxEvidenceRun`, visible-widget scans, and
  app-owned evidence diagnostics in
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab`.

Forbidden fallbacks:

- treating `build_self_authoring_control_panel` or generic text/action lists as
  the standalone UI Designer workbench;
- claiming canvas, hierarchy, inspector, or property editing through labels
  without typed selection, edit intent, diagnostics, and evidence paths;
- app-owned project/session state becoming canonical generic UI Designer truth;
- hidden success when native evidence cannot run. Use a typed
  platform-impossible report instead;
- removing legacy self-authoring compatibility before parity, rollback, and
  Story Lab evidence exist.

Architecture guard tests must cover workbench route mounting, canvas/hierarchy
selection, inspector/property edit intent dispatch, legacy action-list bypass,
Story Lab state matrix coverage, visible-widget scan evidence, app manifest
missing-evidence failure, and platform-impossible evidence handling. These
guards prevent descriptor-only, status-panel-only, prepared-data-only,
fallback-only, or unconsumed-contract completion claims.

Expected completion quality is `runtime_proven` if the implementation produces
native Story Lab manifest evidence for the standalone workbench path. Use
`bounded_contract` if the implementation lands only a narrower view-model,
adapter, or compatibility slice without native evidence. `perfectionist_verified`
is forbidden for `PM-EDITOR-UX-004`.

## Implementation Scope

The implementation slice should create the standalone workbench product chain,
not final visual perfection or all-surface certification.

Expected editor domain work:

- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`: introduce UI Designer workbench view models for canvas,
  hierarchy, inspector, property panels, token/recipe/binding previews,
  diagnostics, and scenario/readiness panes.
- `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`: keep existing Editor Lab compatibility but
  route normal UI Designer workbench composition through product-grade controls
  instead of text/action fallback panels.
- `domain/editor/editor_shell/src/workspace/profile.rs` module `profile`: keep
  Designer workbench mounting stable-key based and separate from legacy
  self-authoring naming.
- `domain/editor/editor_shell/src/story_lab` module `story_lab`: add UI
  Designer workbench stories and scenario matrices for canvas, hierarchy,
  inspector, property editing, diagnostics, preview, overflow, dense documents,
  keyboard/focus, high contrast, reduced motion, reduced data, and degraded
  providers.

Expected app work in the implementation action:

- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`: retain compatibility providers only as adapters and expose
  the new workbench scenarios through app-owned provider frames.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`: keep project IO, apply, rollback, and command execution
  app-owned, but do not make it canonical UI Designer source truth.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab` module
  `editor_ux_story_lab`: require workbench stories to emit native screenshots
  or typed platform-impossible reports plus accessibility, interaction,
  diagnostics, and performance evidence where applicable.

## Non-Goals

- Do not implement product code in this contract-only action.
- Do not claim graph canvas or node editor productization; that remains
  `PM-EDITOR-UX-005`.
- Do not claim shell-wide pattern polish; that remains `PM-EDITOR-UX-006`.
- Do not claim all-surface certification; that remains `PM-EDITOR-UX-007`.
- Do not implement game-runtime HUD behavior or add a game UI owner crate.
- Do not move generic UI Designer source truth into `apps/runenwerk_editor`.
- Do not treat legacy self-authoring text/action panels as the standalone UI
  Designer workbench.
- Do not remove compatibility paths until parity, migration, rollback, and
  Story Lab evidence exist.

## Acceptance Criteria

- The standalone UI Designer workbench has explicit owning modules for canvas,
  hierarchy, inspector, property editing, diagnostics, preview, scenario
  matrices, and readiness evidence.
- Normal authoring workflows route through typed editor workbench view models,
  operation intents, and app command bridges, not generic text/action lists.
- Story Lab coverage names workbench product states, including empty, dense,
  overflow, disabled, warning, error, degraded provider, long-label,
  keyboard/focus, high-contrast, reduced-motion, and viewport variants.
- App-owned evidence manifests fail when workbench stories lack native or typed
  platform-impossible proof, interaction reports, accessibility reports, or
  design-system/readiness evidence.
- Future `game.runtime` compatibility remains at target-profile and evidence
  descriptor seams only.

## Implementation Steps

1. Rerun
   `task production:plan -- --milestone PM-EDITOR-UX-004 --roadmap WR-114`
   and confirm it still reports `WR-114` as `current_candidate` with next
   action `write_implementation_contract`.
2. Validate this contract, then rerun
   `task ai:goal -- --track PT-EDITOR-UX`. Do not start product code in the
   same contract-only action.
3. In the implementation action, inspect current editor shell UI Designer,
   Editor Lab, self-authoring,
   workspace profile, and app provider modules before code changes.
4. Add the smallest vertical workbench chain that proves route mounting,
   canvas/hierarchy selection, inspector/property edit intents, diagnostics,
   app command bridge behavior, Story Lab state matrix coverage, visible-widget
   scan coverage, and evidence manifest behavior without claiming final surface
   polish.
5. Add focused editor-shell tests first, then app provider/evidence tests, then
   PM-EDITOR-UX-004 evidence artifact generation.
6. Keep compatibility shims explicit and temporary.
7. Run focused validation, write closeout evidence, update roadmap and
   production metadata, render/check generated docs, and rerun
   `task ai:goal -- --track PT-EDITOR-UX`.

## Validation

Required validation for this contract-only action:

```text
task production:plan -- --milestone PM-EDITOR-UX-004 --roadmap WR-114
task docs:validate
task planning:validate
task puml:validate
git diff --check
task ai:goal -- --track PT-EDITOR-UX
```

Expected implementation validation after the next action selects code work:

```text
cargo test -p editor_shell ui_designer
cargo test -p editor_shell story_lab
cargo test -p editor_shell
cargo test -p runenwerk_editor ui_designer
cargo test -p runenwerk_editor editor_ux
RUNENWERK_WRITE_PM_EDITOR_UX_004_EVIDENCE=1 cargo test -p runenwerk_editor pm_editor_ux_004 -- --nocapture
```

Use `./quiet_full_gate.sh` for broad closeout if implementation changes shared
editor shell, app evidence, Story Lab, or validation infrastructure.

## Stop Conditions

Stop before implementation if:

- `task ai:goal -- --track PT-EDITOR-UX` no longer selects
  `PM-EDITOR-UX-004`;
- `WR-114` is not ready for the required roadmap action;
- implementation would make `domain/ui` depend on editor or app code;
- app-owned Designer project/session state would become canonical source truth;
- the workbench would be canvas-only, status-panel-only, descriptor-only, or
  generic self-authoring action-list proof;
- migration would remove legacy paths without parity, rollback, and Story Lab
  evidence;
- native evidence cannot produce screenshot or typed platform-impossible
  reports for product workbench scenarios.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md
```

The closeout must state:

- changed files and owning modules;
- workbench view models, route contracts, and command/operation boundaries;
- compatibility-shim decisions for legacy self-authoring paths;
- Story Lab stories and state matrices added;
- app-owned evidence artifacts generated;
- validation commands and results;
- known quality gaps that remain owned by `PM-EDITOR-UX-005` through
  `PM-EDITOR-UX-009`.

Expected completion quality is `runtime_proven` only if native Story Lab
manifest evidence proves the standalone workbench path. Use `bounded_contract`
if the implementation lands only a narrower adapter or contract slice without
native evidence.

## Perfectionist Closeout Audit

`PM-EDITOR-UX-004` must not claim `perfectionist_verified`. The final no-gap
audit remains `PM-EDITOR-UX-009`.

The closeout must keep visible gaps for:

- graph canvas and node editor productization;
- shell and product pattern polish;
- all registered visible surface migration;
- game UI readiness seam;
- final local-native no-gap certification.

Only `PM-EDITOR-UX-009` may remove those gaps after final native screenshots,
accessibility, interaction, visual/performance, roadmap, production, and full
validation evidence agree.
