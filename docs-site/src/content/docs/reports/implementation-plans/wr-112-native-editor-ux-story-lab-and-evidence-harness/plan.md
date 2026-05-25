---
title: WR-112 Native Editor UX Story Lab And Evidence Harness Contract
description: Promotion and implementation-readiness contract for PM-EDITOR-UX-002 native Editor UX Story Lab, typed story catalog, visible-widget scan, and app-owned native evidence harness.
status: active
owner: editor
layer: domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md
  - ../../../design/accepted/ui-designer-production-readiness-and-evidence-design.md
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
related_reports:
  - ../../closeouts/pm-editor-ux-001-governance-truth-audit-and-track-activation/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-112 Native Editor UX Story Lab And Evidence Harness Contract

## Goal

Implement `PM-EDITOR-UX-002` by creating the native Editor UX Story Lab and
evidence harness that every later `PT-EDITOR-UX` milestone must consume.

This contract records the accepted design-first decisions, promotion evidence,
and current implementation-readiness plan requested by
`task production:plan -- --milestone PM-EDITOR-UX-002 --roadmap WR-112`. It
does not authorize product code changes by itself. Product implementation may
start only after this implementation contract validates and the next
`task ai:goal -- --track PT-EDITOR-UX` run still selects the implementation
action for promoted `WR-112`.

Expected production outcome:

- typed editor UX stories, args, interactions, scenario matrices, visible-widget
  scans, and evidence manifests exist at the correct ownership layers;
- primitive widget, editor product pattern, registered surface, and host
  scenario stories can execute through the native editor app;
- certified stories fail hard when visible widgets lack bounds,
  label/accessibility metadata, focus reachability, overflow policy, or required
  state coverage;
- evidence remains app-owned and concrete, while generic UI and editor shell
  truth remain in their domain crates.

## Source Of Truth

- Production milestone: `PM-EDITOR-UX-002` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-112` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Active design:
  `docs-site/src/content/docs/design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md`.
- Completed governance input:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-001-governance-truth-audit-and-track-activation/closeout.md`.
- Existing runtime-neutral UI readiness input:
  `domain/ui/ui_definition/src/production_readiness/mod.rs`.
- Existing retained UI and primitive widget contracts:
  `domain/ui/ui_tree/src/tree/node.rs`,
  `domain/ui/ui_tree/src/computed_layout.rs`, and
  `domain/ui/ui_widgets/src/lib.rs`.
- Existing editor surface registry:
  `domain/editor/editor_shell/src/workspace/surface_contract.rs`.
- Existing app-owned evidence substrate:
  `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs`.

## Implementation Readiness

`task production:plan -- --milestone PM-EDITOR-UX-002 --roadmap WR-112`
reported:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap dependencies: WR-111:completed
Next action: write_implementation_contract
```

Implementation planning is honest because:

- `PM-EDITOR-UX-001` is completed with verified governance, source-truth,
  evidence-matrix, and track-activation closeout evidence.
- `WR-111` is completed and archived as the direct prerequisite for WR-112.
- The active editor product UX Story Lab design is present and remains the
  governing design gate for this row.
- `WR-112` is promoted to `current_candidate` with accepted promotion evidence
  and bounded write scopes.
- WR-112 now has bounded write scopes, explicit ownership decisions, and
  fail-closed validation expectations for the Story Lab implementation.
- No further design, ADR, or dependency gate is required before the bounded
  implementation slice, as long as implementation preserves the ownership
  decisions in this contract.

Promotion history:

```text
Accepted editor product UX Story Lab and game-UI-ready foundations design, verified PM-EDITOR-UX-001 governance closeout, completed WR-111 track activation prerequisite, and active WR-112 promotion/readiness contract clear the native Editor UX Story Lab implementation for current-candidate planning; generic UI truth remains in domain/ui, editor product semantics remain in domain/editor, and native evidence execution remains app-owned in apps/runenwerk_editor.
```

Do not run product-code implementation if a later `task ai:goal` rerun no
longer selects `execute_next_wr_implementation_contract`, if WR-112 leaves
`current_candidate`, or if the active design/ownership gates change.

## Architecture Governance Decision

Architecture governance classifies this slice as editor product work with
generic UI support and app-owned native evidence.

Ownership decisions:

- `domain/ui/ui_tree` owns backend-neutral retained widget tree contracts and
  the visible-widget scan vocabulary that can inspect `UiNode` plus computed
  layout/accessibility/focus metadata.
- `domain/ui/ui_widgets` owns primitive widget constructors and their primitive
  story adapters, but it must not own editor product semantics or native
  screenshot execution.
- `domain/ui/ui_definition` owns generic readiness, target profile, fixture,
  scenario, and evidence descriptor vocabulary. It remains behavior-free.
- `domain/editor/editor_shell` owns `EditorUxStoryCatalog`,
  `EditorUxStoryId`, `EditorUxStoryKind`, `EditorUxStoryArgs`,
  `EditorUxStoryInteraction`, `EditorUxScenarioMatrix`, and
  `ToolSurfaceReadiness` for editor product stories, product patterns, surface
  classification, provider routing expectations, and shell view-model
  coverage.
- `apps/runenwerk_editor` owns native Story Lab execution, provider fixtures,
  launched-editor runners, local screenshot/artifact capture, accessibility and
  performance capture, and concrete `EditorUxEvidenceManifest` files.

Dependency direction remains:

```text
domain/ui -> domain/editor -> apps/runenwerk_editor
```

`domain/ui` must not depend on editor shell, app providers, native screenshots,
runtime sessions, renderer handles, or game UI semantics. `domain/editor` may
depend on generic UI contracts but must not write app artifacts. App code may
execute evidence but must not become generic UI or editor product source truth.

ADR decision: no ADR is required for this first Story Lab contract if the
implementation preserves those owners. Require an ADR or accepted design update
before moving native evidence into `domain/ui`, making app evidence
authoritative domain state, changing persistent public formats, adding a game UI
owner crate, or making game-runtime UI depend on editor shell vocabulary.

ATAM-lite priority order:

1. ownership and dependency correctness;
2. fail-closed visible-widget diagnostics;
3. native reproducible evidence;
4. story author ergonomics and discoverability;
5. performance of scan and evidence generation.

Team Topologies label: stream-aligned editor product work with
complicated-subsystem support from UI substrate, editor shell, and app evidence
owners.

## Implementation Contract Decisions

The implementation must introduce the Story Lab as a proof substrate, not as
surface polish. The first code slice should make missing stories and weak
evidence fail before any later polish row can claim completion.

Source-truth chain:

1. Generic retained widget contracts expose widget kind, identity, layout
   bounds, and backend-neutral metadata needed for visible-widget scans.
2. Primitive widget constructors register primitive stories through adapter
   functions, not through app-only screenshots or manual notes.
3. Editor shell story catalog entries declare editor UX story IDs, story kind,
   args, interactions, scenario matrix axes, required surface readiness, and
   expected diagnostics.
4. App-owned runners materialize stories into retained UI, execute interactions,
   capture local-native evidence where possible, and write manifests.
5. Manifest validation fails when required scenarios, artifacts, accessibility,
   focus, overflow, state coverage, freshness, or concrete native evidence are
   missing.

Forbidden shortcuts:

- descriptor-only stories with no executable retained UI or app runner;
- retained-preview-only proof when local native capture is available;
- screenshots without story IDs, args, interactions, scenarios, and manifests;
- app-owned story catalogs that become the normal source of editor product
  readiness;
- generic UI contracts that import editor shell or app evidence vocabulary;
- `ToolSurfaceReadiness` stored only as comments, labels, or display text;
- normal product workflows that expose visible placeholder surfaces or generic
  action panels as if they were certified product surfaces.

## Implementation Scope

Expected domain UI files and modules:

- `domain/ui/ui_tree/src/inspection/mod.rs`: add backend-neutral
  `VisibleWidgetScan`, `VisibleWidgetScanIssue`, and
  `VisibleWidgetScanRequirement` contracts.
- `domain/ui/ui_tree/src/tree/node.rs`: expose only the metadata needed by the
  scan; do not make app evidence or editor semantics part of `UiNode`.
- `domain/ui/ui_tree/src/lib.rs`: export the inspection contracts through the
  crate's focused public surface.
- `domain/ui/ui_widgets/src/lib.rs` and primitive widget files such as
  `button.rs`, `label.rs`, `text_input.rs`, `numeric_input.rs`, `tabs.rs`,
  `select.rs`, `table.rs`, `tree_widget.rs`, and `product_surface.rs`: add
  primitive story adapters only when they remain generic and backend-neutral.

Expected editor domain files and modules:

- `domain/editor/editor_shell/src/story_lab/mod.rs`: add the editor-owned Story
  Lab module boundary.
- `domain/editor/editor_shell/src/story_lab/catalog.rs`: add
  `EditorUxStoryCatalog`, registration, lookup, and validation.
- `domain/editor/editor_shell/src/story_lab/story.rs`: add `EditorUxStoryId`,
  `EditorUxStoryKind`, `EditorUxStoryArgs`, and `EditorUxStoryInteraction`.
- `domain/editor/editor_shell/src/story_lab/scenario.rs`: add
  `EditorUxScenarioMatrix`, required state axes, density/viewport/input axes,
  and expected diagnostic references.
- `domain/editor/editor_shell/src/story_lab/readiness.rs`: add
  `ToolSurfaceReadiness` with `Product`, `FallbackOnly`, `Diagnostic`, and
  `HiddenUntilProductized`.
- `domain/editor/editor_shell/src/workspace/surface_contract.rs`: connect
  registered surfaces to readiness classification without changing surface
  source-truth ownership.
- `domain/editor/editor_shell/src/lib.rs`: export only the common Story Lab
  contracts needed by app runners and tests.

Expected app files and modules:

- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/mod.rs`: add the
  app-owned native Story Lab runner boundary.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/fixtures.rs`: create
  provider and host scenario fixtures.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/runner.rs`: execute
  stories, interactions, and scenario matrices.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/visible_widget_scan.rs`:
  adapt app-generated retained UI and layout/focus data into the generic scan.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs`: produce
  `EditorUxEvidenceManifest` values by reusing and extending the existing
  `editor_lab_evidence` substrate where appropriate.
- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs`: extend only
  app-owned artifact and manifest validation vocabulary that is genuinely
  shared by Editor Lab and Editor UX Story Lab evidence.
- `apps/runenwerk_editor/src/shell/tests.rs`: add focused tests for runner,
  manifest, missing-artifact, and fail-closed scan behavior.

## Implementation Steps

1. Confirm `task production:plan -- --milestone PM-EDITOR-UX-002 --roadmap
   WR-112` still reports `write_implementation_contract` for promoted
   `WR-112`.
2. Rerun `task ai:goal -- --track PT-EDITOR-UX` after this contract validates
   and confirm the next legal action remains
   `execute_next_wr_implementation_contract`.
3. Inspect the current UI tree, primitive widget, editor shell surface
   registry, and app-owned `editor_lab_evidence` modules before adding Story
   Lab contracts so existing public APIs and evidence helpers are reused where
   they fit.
4. Add backend-neutral visible-widget scan contracts in `domain/ui/ui_tree`
   first, with tests that fail on missing bounds, labels, accessibility
   metadata, focus reachability, overflow policy, and state coverage.
5. Add generic primitive story adapters in `domain/ui/ui_widgets` only where
   they remain editor-agnostic and backend-neutral.
6. Add the editor-owned story catalog, story ID/args/interaction/scenario
   matrix, and `ToolSurfaceReadiness` contracts in
   `domain/editor/editor_shell`, then connect registered surfaces to readiness
   classification.
7. Add the app-owned native Story Lab runner, fixtures, visible-widget scan
   adapter, and evidence manifest generation in `apps/runenwerk_editor`,
   reusing `editor_lab_evidence` for shared artifact validation instead of
   duplicating it.
8. Generate or refresh local-native evidence manifests only from app-owned
   runners, then record artifact paths and any typed platform-impossible
   results in the PM002 closeout.
9. Update roadmap and production metadata only after focused validation and
   evidence generation pass.

## Acceptance Criteria

- `EditorUxStoryCatalog`, `EditorUxStoryId`, `EditorUxStoryKind`,
  `EditorUxStoryArgs`, `EditorUxStoryInteraction`,
  `EditorUxScenarioMatrix`, `EditorUxEvidenceManifest`, and
  `ToolSurfaceReadiness` exist in the owners named above.
- Primitive widget, product pattern, registered surface, and host scenario
  stories can be listed and executed by the app runner.
- Visible-widget scans fail for missing layout bounds, missing labels or
  accessibility metadata on interactive widgets, unreachable focus targets,
  missing overflow policy, stale artifacts, and missing required state coverage.
- Native evidence manifests require retained UI artifacts and local-native
  screenshots where supported, or typed platform-impossible evidence when a
  check cannot run locally.
- Later milestones can consume the Story Lab without adding app-only style
  truth, descriptor-only proof, or normal-flow placeholders.

## Validation

Focused implementation validation:

```text
cargo fmt --all -- --check
cargo test -p ui_tree visible_widget
cargo test -p ui_widgets story
cargo test -p editor_shell story_lab
cargo test -p editor_shell surface_readiness
cargo test -p runenwerk_editor editor_ux_story_lab
cargo test -p runenwerk_editor editor_ux_evidence
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
task ai:goal -- --track PT-EDITOR-UX
```

Run `./quiet_editor_gate.sh` if the implementation touches editor runtime or
shell behavior outside the focused Story Lab paths. Use `./quiet_full_gate.sh`
only for broad closeout or final certification.

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/pm-editor-ux-002-native-editor-ux-story-lab-and-evidence-harness/closeout.md`
only after focused validation and evidence generation pass.

The closeout must include:

- the exact story catalog and scenario matrix modules;
- validation commands and results;
- retained UI, visible-widget scan, accessibility, interaction, and native
  screenshot or platform-impossible artifact paths;
- evidence manifest path and freshness policy;
- roadmap and production metadata updates;
- explicit known quality gaps for PM-EDITOR-UX-003 through PM-EDITOR-UX-009.

`PM-EDITOR-UX-002` may close as `runtime_proven` only if the native runner and
manifest validation prove executable stories and concrete app-owned evidence.
It must not claim `perfectionist_verified`; design-system migration,
standalone UI Designer, graph productization, shell polish, all-surface wave,
game UI readiness, and final no-gap certification remain future milestones.

## Perfectionist Closeout Audit

This row is a prerequisite to perfectionist verification, not the final audit.
The implementation must make future perfectionist claims auditable by recording:

- every known story coverage gap;
- every visible-widget scan failure as a typed diagnostic;
- every unsupported native check as a typed platform-impossible result with a
  reproduction command;
- every remaining product surface readiness gap in roadmap and production
  metadata.

Final `perfectionist_verified` remains reserved for `PM-EDITOR-UX-009` after
all hard zero-budget gates pass with empty known quality gaps.

## Stop Conditions

Stop before implementation if:

- `WR-112` is not promoted to `current_candidate` through the roadmap
  workflow;
- ownership would put native screenshots, app provider fixtures, or evidence
  artifact writing in `domain/ui` or `domain/editor`;
- generic UI contracts would need to depend on editor shell or app state;
- `ToolSurfaceReadiness` cannot be enforced by typed tests;
- the visible-widget scan would be descriptor-only, retained-only without
  layout/focus/accessibility metadata, or screenshot-only without widget
  identity;
- implementation would start PM-EDITOR-UX-003 design-system migration,
  PM-EDITOR-UX-004 UI Designer workbench, PM-EDITOR-UX-005 graph productization,
  or any later milestone.
