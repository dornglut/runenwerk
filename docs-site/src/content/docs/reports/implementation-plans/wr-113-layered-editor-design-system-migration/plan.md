---
title: WR-113 Layered Editor Design System Migration Contract
description: Current-candidate implementation contract for PM-EDITOR-UX-003 layered editor design-system migration over shared token, recipe, state, Story Lab, and evidence contracts.
status: active
owner: editor
layer: domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-story-lab-and-game-ui-ready-foundations-design.md
  - ../../../design/accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md
  - ../../../design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
related_reports:
  - ../wr-112-native-editor-ux-story-lab-and-evidence-harness/plan.md
  - ../../closeouts/pm-editor-ux-002-native-editor-ux-story-lab-and-evidence-harness/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-113 Layered Editor Design System Migration Contract

## Goal

Define the decision-complete implementation contract for `PM-EDITOR-UX-003`
now that `WR-113` is the selected `current_candidate` roadmap row for editor
primitives and product patterns.

This is a design-first contract produced from:

```text
task production:plan -- --milestone PM-EDITOR-UX-003 --roadmap WR-113
```

It clears the implementation-contract action by naming the source-truth chain,
owners, migration shape, non-goals, fitness functions, and closeout bar for the
next implementation slice. This workflow action does not implement product code.
Product code may start only after this contract validates and a later
coordinator run continues the legal `WR-113` implementation action.

Expected production outcome:

- editor primitive and product-pattern styling consumes shared
  `domain/ui/ui_theme` token graph contracts where appropriate;
- editor product pattern structure consumes shared
  `domain/ui/ui_definition` recipe and state contracts where appropriate;
- editor-specific package and adapter semantics remain in
  `domain/editor/editor_definition` and `domain/editor/editor_shell`;
- native proof remains app-owned in `apps/runenwerk_editor` through the Editor
  UX Story Lab runner and evidence manifests;
- app-only styling paths are removed or isolated as temporary compatibility
  shims, not treated as durable source truth.

## Source Of Truth

- Production milestone: `PM-EDITOR-UX-003` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-113` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Active editor UX doctrine:
  `docs-site/src/content/docs/design/active/editor-product-ux-story-lab-and-game-ui-ready-foundations-design.md`.
- Accepted token design:
  `docs-site/src/content/docs/design/accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md`.
- Accepted recipe design:
  `docs-site/src/content/docs/design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md`.
- Completed Story Lab prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-002-native-editor-ux-story-lab-and-evidence-harness/closeout.md`.
- Generic token implementation entrypoint:
  `domain/ui/ui_theme/src/token/mod.rs` module `token`.
- Generic recipe implementation entrypoint:
  `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe`.
- Editor definition package entrypoints:
  `domain/editor/editor_definition/src/theme.rs` module `theme` and
  `domain/editor/editor_definition/src/document.rs` enum
  `EditorDefinitionDocumentContent`.
- Editor shell proof entrypoints:
  `domain/editor/editor_shell/src/story_lab/mod.rs` module `story_lab` and
  `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `tool_surface_readiness` and
  `tool_surface_readiness_for_definition_id`.
- App-owned evidence entrypoint:
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/mod.rs` module
  `editor_ux_story_lab`.

## Readiness

`task production:plan -- --milestone PM-EDITOR-UX-003 --roadmap WR-113`
currently reports:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-112:completed
Next action: write_implementation_contract
```

The earlier design-first blocker is cleared because:

- `WR-112` is completed with `runtime_proven` closeout evidence;
- `PM-EDITOR-UX-002` has native Story Lab, visible-widget scan, surface
  readiness, and app-owned manifest evidence;
- the accepted token and recipe designs already define generic UI source truth;
- this contract records the design-system migration boundary between generic UI
  contracts, editor product semantics, and native app evidence;
- `WR-113` has been promoted to `current_candidate` with accepted promotion
  evidence and still depends only on completed `WR-112`.

This action is still contract-only. The next coordinator run must rerun
`task ai:goal -- --track PT-EDITOR-UX` after validation before making product
code changes.

## Architecture Governance Review

Recommendation: promote the WR-113 design gate from blocked discovery to
ready-next promotion planning after this contract and metadata validate.

Scope:

- `domain/ui/ui_theme/src/token/mod.rs` module `token`;
- `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe`;
- `domain/editor/editor_definition/src/theme.rs` module `theme`;
- `domain/editor/editor_definition/src/document.rs` enum
  `EditorDefinitionDocumentContent`;
- `domain/editor/editor_shell/src/story_lab/mod.rs` module `story_lab`;
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/mod.rs` module
  `editor_ux_story_lab`.

Owner:

- Generic design-system truth belongs to `domain/ui`.
- Editor product packages and adapters belong to `domain/editor`.
- Native execution evidence belongs to `apps/runenwerk_editor`.

Dependency direction:

```text
domain/ui -> domain/editor -> apps/runenwerk_editor
```

`domain/ui` must not import editor shell, app evidence, renderer handles,
provider sessions, or game UI vocabulary. `domain/editor` may adapt generic UI
tokens and recipes into editor product packages. `apps/runenwerk_editor` may
execute stories and write evidence, but it must not become the canonical owner
of generic token or recipe semantics.

ADR need: no new ADR is required while implementation preserves accepted token,
recipe, description-versus-execution, and derived-projection decisions. Require
an ADR or accepted design update before moving styling source truth into app
code, making app evidence authoritative domain state, changing dependency
direction, or making future game-runtime UI depend on editor shell vocabulary.

ATAM-lite:

- Quality attributes in tension: fast editor polish, source-truth integrity,
  Story Lab coverage, app compatibility, and future `game.runtime` profile
  compatibility.
- Chosen option: migrate one product-pattern chain at a time through shared
  token and recipe contracts, with app-only paths isolated as compatibility
  shims until covered by Story Lab evidence.
- Sensitivity points: token provenance loss, recipe expansion that bypasses
  target-profile diagnostics, visible-widget stories that pass without state
  matrix coverage, and direct `ThemeTokens` mutations in app code.
- Risk: a local visual tweak could appear complete while leaving generic token
  or recipe contracts unused.
- Non-risk: keeping native screenshot and manifest generation app-owned, because
  accepted design already assigns native evidence execution to the app.

Migration shape: use a Strangler Fig migration for old app/editor styling
paths. Freeze direct app-only style edits behind compatibility adapters, route
the first primitive or product-pattern family through token and recipe
contracts, prove parity and Story Lab evidence, then switch additional
patterns and remove obsolete compatibility paths only when guards are present.

Fitness functions:

- token provenance, layer ordering, target-profile, and preview-only activation
  tests in `domain/ui/ui_theme/src/token/mod.rs`;
- recipe expansion, slot, token-family, accessibility, and target-profile tests
  in `domain/ui/ui_definition/src/component_recipe/mod.rs`;
- editor package adapter and compatibility tests in
  `domain/editor/editor_definition/src/theme.rs` and nearby editor-definition
  modules;
- story catalog and state matrix tests in
  `domain/editor/editor_shell/src/story_lab`;
- native Story Lab manifest tests in
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab`.

Ownership mode: stream-aligned editor product migration with complicated
subsystem support from the UI theme, UI definition, editor shell, and app
evidence owners.

## Critical Review Gate

Source truth:

- Generic token truth is `domain/ui/ui_theme/src/token/mod.rs` module `token`.
  Resolved token payloads, preview output, app theme handles, screenshots, and
  evidence manifests are projections or runtime products, not source truth.
- Generic recipe truth is
  `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe`. Expanded recipe instances, editor story fixtures, and app
  runner snapshots are projections.
- Editor product semantics are
  `domain/editor/editor_definition/src/theme.rs` module `theme`,
  `domain/editor/editor_definition/src/document.rs` enum
  `EditorDefinitionDocumentContent`, and editor shell Story Lab metadata.
- Native execution proof is app-owned under
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab`, but app artifacts must
  not become canonical design-system truth.

Required source-to-runtime chain:

```text
Editor package document
  -> typed editor token/recipe adapter
  -> generic ThemeTokenGraph and UiRecipeLibrary declarations
  -> token/recipe/state resolution with provenance and diagnostics
  -> editor Story Lab story and state matrix requirements
  -> native EditorUxStoryLabRunner execution
  -> EditorUxEvidenceManifest artifacts and blocking diagnostics
```

The implementation must not stop at descriptor registration, status panels,
prepared data, or screenshot labels. A migrated pattern is not complete until
tests prove that its typed editor adapter is consumed by generic token/recipe
resolution and the app-owned Story Lab manifest reports the migrated
token/recipe/state evidence or an explicit platform-impossible diagnostic.

Typed contracts that must be used or extended before introducing new strings or
ad hoc maps:

- `ThemeTokenGraph`, `ThemeTokenDeclaration`,
  `ThemeTokenResolveRequest`, and token provenance diagnostics;
- `UiRecipeDeclaration`, `UiRecipeLibrary`,
  `UiRecipeExpansionRequest`, and recipe diagnostics;
- `EditorThemeDefinition` and versioned editor definition documents;
- `EditorUxStory`, story state matrix descriptors, and surface readiness
  classifications;
- `EditorUxEvidenceManifest` and app-owned missing-evidence diagnostics.

Forbidden fallbacks:

- app-only style packages treated as canonical token or recipe definitions;
- direct `ThemeTokens` mutations without a named compatibility adapter and
  migration test;
- hardcoded defaults that bypass token provenance or recipe diagnostics;
- descriptor-only Story Lab entries that lack state matrix coverage;
- hidden success when native evidence cannot run. Use a typed
  platform-impossible report instead.

Architecture guard tests must cover token provenance, recipe expansion,
target-profile diagnostics, editor package compatibility, Story Lab state
matrices, visible-widget scan coverage, and app manifest missing-evidence
failure. These guards prevent descriptor-only, prepared-data-only,
fallback-only, or unconsumed-contract completion claims.

Expected completion quality is `runtime_proven` if the implementation produces
native Story Lab manifest evidence for at least the migrated design-system path.
Use `bounded_contract` if the implementation lands only a narrower adapter or
compatibility slice without native evidence. `perfectionist_verified` is
forbidden for `PM-EDITOR-UX-003`.

## Implementation Scope

The implementation slice should migrate design-system layers, not perform final
visual polish.

Expected generic UI work:

- `domain/ui/ui_theme/src/token/mod.rs` module `token`: extend or reuse
  `ThemeTokenGraph`, `ThemeTokenDeclaration`, `ThemeTokenResolveRequest`, and
  `resolve_theme_tokens` so editor product packages can resolve state, mode,
  accessibility, and target-profile styling with provenance.
- `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe`: extend or reuse `UiRecipeDeclaration`,
  `UiRecipeLibrary`, `UiRecipeExpansionRequest`, and recipe diagnostics so
  editor product patterns can declare structure, required token families, state
  variants, accessibility, layout, and focus/navigation behavior.
- `domain/ui/ui_widgets/src/story.rs` module `story`: keep primitive stories
  backend-neutral while adding state coverage only through generic widget and
  scan contracts.

Expected editor domain work:

- `domain/editor/editor_definition/src/theme.rs` module `theme`: introduce the
  editor package adapter from `EditorThemeDefinition` or its successor into
  typed `ui_theme` token declarations, while preserving compatibility with
  existing persisted schema version 1 documents.
- `domain/editor/editor_definition/src/document.rs` enum
  `EditorDefinitionDocumentContent`: add new content variants only if the
  migration needs typed editor design-system package documents. Do not widen the
  enum for app-only preview state.
- `domain/editor/editor_shell/src/story_lab/story.rs` struct `EditorUxStory`:
  add design-system state or token/recipe expectations only as editor product
  proof metadata.
- `domain/editor/editor_shell/src/story_lab/catalog.rs` function
  `default_editor_ux_stories`: require migrated primitive and product-pattern
  stories to declare state matrix coverage for token, recipe, density,
  high-contrast, reduced-motion, disabled, focused, warning, error, overflow,
  and long-label variants where applicable.
- `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `tool_surface_readiness` and
  `tool_surface_readiness_for_definition_id`: keep readiness classification
  separate from styling source truth.

Expected app work:

- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/runner.rs` struct
  `EditorUxStoryLabRunner`: execute migrated state matrices and report missing
  token/recipe/state evidence as blocking manifest diagnostics.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs` struct
  `EditorUxEvidenceManifest`: require product stories affected by this migration
  to include design-system evidence artifacts or typed platform-impossible
  reports.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` module `self_authoring`
  and `apps/runenwerk_editor/src/shell/applied_editor_definition/activation.rs`
  module `activation`: keep compatibility shims explicit when old theme
  formation or direct `ThemeTokens` application remains temporarily necessary.

## Non-Goals

- Do not implement game-runtime HUD behavior or a game UI owner crate.
- Do not move generic token, recipe, or target-profile truth into
  `apps/runenwerk_editor`.
- Do not move native screenshots, local artifacts, provider fixtures, or
  accessibility/performance runner output into `domain/ui`.
- Do not claim standalone UI Designer workbench completion; that remains
  `PM-EDITOR-UX-004`.
- Do not claim graph canvas or node editor productization; that remains
  `PM-EDITOR-UX-005`.
- Do not claim all-surface certification or final no-gap verification; those
  remain `PM-EDITOR-UX-007` and `PM-EDITOR-UX-009`.
- Do not remove persisted editor definition compatibility without a migration
  dry-run, diff, rollback, and closeout evidence.

## Acceptance Criteria

- UI primitives that this slice touches resolve styling through shared token
  graph declarations or an explicitly named compatibility shim.
- Editor product patterns that this slice touches declare structure through
  shared recipe/state contracts or an explicitly named compatibility shim.
- Story Lab coverage names the token, recipe, state, density, accessibility,
  focus, overflow, warning/error, and long-label variants expected for migrated
  patterns.
- App-owned evidence manifests fail when a migrated product story lacks
  design-system evidence, state matrix coverage, visible-widget scan coverage,
  or an explicit platform-impossible report.
- Future `game.runtime` compatibility is preserved through target-profile ids
  and diagnostics, not editor shell or app dependencies.
- Legacy direct styling paths are either removed or isolated behind named
  compatibility adapters with tests that prevent them from becoming canonical.

## Implementation Steps

1. Rerun
   `task production:plan -- --milestone PM-EDITOR-UX-003 --roadmap WR-113`
   and confirm it still reports `WR-113` as `current_candidate` with next
   action `write_implementation_contract`.
2. Validate this contract, then rerun
   `task ai:goal -- --track PT-EDITOR-UX`. Do not start product code in the
   same contract-only action.
3. In the implementation action, inspect current `ui_theme`, `ui_definition`,
   editor definition, Story Lab,
   and app evidence modules before code changes.
4. Add or adapt the smallest editor design-system package path that proves one
   primitive or product-pattern family consumes shared token and recipe
   contracts.
5. Add focused generic UI tests first, then editor adapter tests, then Story Lab
   and app manifest tests.
6. Keep compatibility shims named and temporary. Do not delete old paths until
   parity, migration, rollback, and story evidence exist.
7. Run focused validation, write closeout evidence, update roadmap and
   production metadata, render/check generated docs, and rerun
   `task ai:goal -- --track PT-EDITOR-UX`.

## Validation

Required validation for this contract-only action:

```text
task production:plan -- --milestone PM-EDITOR-UX-003 --roadmap WR-113
task docs:validate
task planning:validate
task puml:validate
git diff --check
task ai:goal -- --track PT-EDITOR-UX
```

Expected implementation validation after the next action selects code work:

```text
cargo test -p ui_theme
cargo test -p ui_definition
cargo test -p editor_definition
cargo test -p editor_shell story_lab
cargo test -p editor_shell
cargo test -p runenwerk_editor editor_ux
RUNENWERK_WRITE_PM_EDITOR_UX_003_EVIDENCE=1 cargo test -p runenwerk_editor pm_editor_ux_003 -- --nocapture
```

Use `./quiet_full_gate.sh` for broad closeout if implementation changes shared
UI, editor shell, app evidence, or validation infrastructure.

## Stop Conditions

Stop before implementation if:

- `task ai:goal -- --track PT-EDITOR-UX` no longer selects `PM-EDITOR-UX-003`;
- `WR-113` is not ready for the required roadmap action;
- any design gate for token, recipe, target-profile, or editor UX Story Lab
  doctrine is missing or not accepted/active as required;
- implementation would make `domain/ui` depend on editor or app code;
- implementation would make app-owned styling or screenshot artifacts canonical
  generic UI truth;
- migration would break persisted editor definition compatibility without a
  migration/rollback plan;
- Story Lab evidence would be descriptor-only, retained-only, or missing state
  matrix coverage for a product claim.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md
```

The closeout must state:

- changed files and owning modules;
- token graph and recipe contracts consumed by migrated patterns;
- editor adapter and compatibility-shim decisions;
- Story Lab story/state matrix coverage added;
- app-owned evidence artifacts generated;
- validation commands and results;
- known quality gaps that remain owned by `PM-EDITOR-UX-004` through
  `PM-EDITOR-UX-009`.

Expected completion quality is `runtime_proven` only if native Story Lab
manifest evidence proves the migrated design-system path. Use
`bounded_contract` if the closeout only lands a narrower adapter or contract
slice without native evidence.

## Perfectionist Closeout Audit

`PM-EDITOR-UX-003` must not claim `perfectionist_verified`. The final no-gap
audit remains `PM-EDITOR-UX-009`.

The closeout must keep visible gaps for:

- standalone UI Designer workbench;
- graph canvas and node editor productization;
- shell and product pattern polish;
- all registered visible surface migration;
- game UI readiness seam;
- final local-native no-gap certification.

Only `PM-EDITOR-UX-009` may remove those gaps after final native screenshots,
accessibility, interaction, visual/performance, roadmap, production, and full
validation evidence agree.
