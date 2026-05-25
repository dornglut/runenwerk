---
title: WR-116 Shell And Product Pattern Polish Design And Promotion Contract
description: Design-first and promotion-readiness contract for PM-EDITOR-UX-006 shell and product pattern polish before product implementation.
status: active
owner: editor
layer: domain/editor / app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md
related_reports:
  - ../wr-113-layered-editor-design-system-migration/plan.md
  - ../wr-114-standalone-ui-designer-workbench/plan.md
  - ../wr-115-graph-canvas-and-node-editor-productization/plan.md
  - ../../closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md
  - ../../closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md
  - ../../closeouts/pm-editor-ux-005-graph-canvas-and-node-editor-productization/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-116 Shell And Product Pattern Polish Design And Promotion Contract

## Goal

Clear the design-first blocker for `PM-EDITOR-UX-006` and prepare `WR-116` for
promotion planning. This action is planning and metadata only. It does not
change editor product code.

This contract is produced from:

```text
task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116
```

Expected production outcome for the later implementation slice:

- inspector, palette, diagnostics, preview, table, tree, tab, toolbar, status,
  split, dock, empty, loading, error, degraded, overflow, keyboard, and focus
  patterns have reusable Story Lab evidence;
- pattern evidence is typed, reusable, and editor-owned rather than app-only
  styling or ad hoc provider output;
- provider-hosted execution, native captures or typed platform-impossible
  reports, timing, focus, accessibility, and diagnostics evidence remain owned
  by `apps/runenwerk_editor`;
- the slice does not claim all registered surfaces, game-runtime UI readiness,
  or final no-gap certification. Those remain `PM-EDITOR-UX-007`,
  `PM-EDITOR-UX-008`, and `PM-EDITOR-UX-009`.

## Source Of Truth

- Production milestone: `PM-EDITOR-UX-006` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-116` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Active editor UX doctrine:
  `docs-site/src/content/docs/design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md`.
- Completed design-system prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md`.
- Completed standalone UI Designer prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md`.
- Completed graph canvas prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-005-graph-canvas-and-node-editor-productization/closeout.md`.
- Current compact control policy:
  `domain/editor/editor_shell/src/composition/surface_control_polish.rs`
  module `surface_control_polish`.
- Current inspector composition:
  `domain/editor/editor_shell/src/composition/build_inspector_panel.rs`
  function `build_inspector_panel`.
- Current toolbar compatibility entrypoint:
  `domain/editor/editor_shell/src/composition/build_toolbar.rs` function
  `build_toolbar`.
- Current console/diagnostics-style composition:
  `domain/editor/editor_shell/src/composition/build_console_panel.rs`
  function `build_console_panel`.
- Current shell chrome, split, dock, route, and tab-stack composition:
  `domain/editor/editor_shell/src/composition/build_editor_shell.rs` module
  `build_editor_shell`.
- Registered surface readiness source:
  `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `tool_surface_readiness` and
  `tool_surface_readiness_for_definition_id`.
- Editor UX Story Lab source:
  `domain/editor/editor_shell/src/story_lab` module `story_lab`.
- App-owned provider execution and frame resolution:
  `apps/runenwerk_editor/src/shell/providers/mod.rs` module `providers`,
  especially `EditorSurfaceProviderRegistry::resolve_frame`.
- App-owned native evidence:
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab` module
  `editor_ux_story_lab`.

## Readiness

`task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116`
reported:

```text
Production milestone state: designing
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-113:completed, WR-114:completed, WR-115:completed
Next action: design_first
```

The dependencies are now complete, but implementation remains illegal until this
design-first contract validates, `PM-EDITOR-UX-006` is moved to `ready_next`,
`WR-116` is moved out of deferred planning, and `task ai:goal -- --track
PT-EDITOR-UX` is rerun. The next legal action after that rerun should be
promotion-readiness planning, not product code.

## Architecture Governance Review

Recommendation: clear the design blocker and then run promotion planning before
implementation. Do not implement product code in this action.

DDD owner:

- `domain/editor/editor_shell` owns editor product pattern vocabulary, view
  models, retained composition adapters, surface readiness, Story Lab scenario
  metadata, and reusable shell pattern contracts.
- `domain/ui` owns generic retained UI primitives, layout/input contracts,
  recipe expansion, target profiles, and graph substrate. It must not grow
  editor-specific inspector, palette, diagnostics, workbench, or provider
  semantics.
- `apps/runenwerk_editor` owns app provider execution, native Story Lab runs,
  provider fixtures, command dispatch, screenshot or platform-impossible
  artifacts, focus/accessibility/timing reports, and evidence manifests.

Vocabulary and invariants:

- Product pattern means a reusable editor UX shape with typed state coverage,
  focus and keyboard behavior, overflow policy, diagnostics behavior, and
  app-owned native evidence.
- Certified pattern evidence is not a substitute for all-surface
  certification. It can be reused by `PM-EDITOR-UX-007`, but it does not close
  every registered visible surface.
- Normal product workflows must not expose generic action/text panels,
  misleading placeholders, or provider line dumps where a certified product
  pattern is claimed.
- App evidence is proof, not editor-shell source truth. Domain/editor contracts
  must remain inspectable without importing the app.

Dependency direction:

```text
domain/ui -> domain/editor/editor_shell -> apps/runenwerk_editor
```

`domain/ui` must not import `domain/editor` or app provider code.
`domain/editor/editor_shell` may consume generic UI primitives and define
editor product pattern view models. `apps/runenwerk_editor` may host, execute,
and prove those patterns, but it must not become the source of reusable pattern
truth.

Translation boundaries:

- Generic retained widgets, layout, recipe, focus, and scan contracts translate
  into editor product patterns inside `domain/editor/editor_shell`.
- Editor product pattern stories translate into app-owned native evidence inside
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab`.
- Provider frames translate runtime state into retained UI, but reusable pattern
  rules stay in the editor shell.

ADR need: no new ADR is required for a bounded pattern-polish slice that
preserves the current dependency direction and source-truth split. Require an
ADR or accepted design update before moving reusable editor pattern truth into
`apps/runenwerk_editor`, moving app-native evidence capture into `domain/ui` or
`domain/editor`, changing command or surface source-truth authority, or making
future game-runtime UI depend on editor shell vocabulary.

ATAM-lite:

- Quality attributes in tension: visual polish, reusable product pattern
  contracts, app-native evidence, keyboard/focus correctness, accessibility,
  diagnostics honesty, and scope control before all-surface certification.
- Chosen option: certify reusable shell and product patterns first, then let
  `PM-EDITOR-UX-007` apply those patterns to every registered visible surface.
- Sensitivity points: polishing one provider without a reusable pattern,
  counting screenshots without typed story state coverage, claiming all-surface
  readiness from pattern examples, and hiding focus/accessibility failures
  behind native capture fallbacks.
- Risk: WR-024 `Editor Shell Polish` has overlapping terminology. WR-116 is
  stricter: it is the PT-EDITOR-UX pattern certification slice and must consume
  current shell/chrome contracts rather than reopen unrelated Interaction V2
  policy.

Fitness functions before implementation:

- `cargo test -p editor_shell story_lab`
- `cargo test -p editor_shell shell`
- `cargo test -p editor_shell inspector`
- `cargo test -p runenwerk_editor editor_ux`
- `cargo test -p runenwerk_editor shell`
- `cargo test -p runenwerk_editor providers`
- `task docs:validate`
- `task planning:validate`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task puml:validate`
- `git diff --check`

Ownership mode: stream-aligned editor shell product work with app-host evidence
support. Generic UI remains a complicated-subsystem dependency consumed through
stable contracts.

## Implementation Scope For The Later Slice

The later promoted implementation may change these owners and only these owners
unless the promotion contract narrows them further:

- `domain/editor/editor_shell/src/story_lab` module `story_lab`: add product
  pattern story metadata and reusable state matrices for inspector, palette,
  diagnostics, preview, table, tree, tab, toolbar/status, split/dock,
  empty/loading/error/degraded, overflow, keyboard, and focus coverage.
- `domain/editor/editor_shell/src/composition/surface_control_polish.rs` module
  `surface_control_polish`: consolidate reusable compact control sizing,
  overflow, focus, and state styling policies where current code already owns
  them.
- `domain/editor/editor_shell/src/composition/build_inspector_panel.rs`
  function `build_inspector_panel`: prove inspector fields, target summaries,
  readonly/editable controls, long labels, empty/error targets, and focus
  order through Story Lab evidence.
- `domain/editor/editor_shell/src/composition/build_toolbar.rs` function
  `build_toolbar` and the active toolbar definition path: prove toolbar/status
  action states, disabled/selected variants, keyboard paths, overflow, and
  route metadata.
- `domain/editor/editor_shell/src/composition/build_console_panel.rs` function
  `build_console_panel`: prove diagnostics/status text, log levels, overflow,
  and focus behavior without treating console text dumps as product pattern
  proof for unrelated surfaces.
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs` module
  `build_editor_shell`: prove split/dock/tab-stack chrome and keyboard/focus
  routes only where this milestone owns reusable product pattern evidence.
- `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `tool_surface_readiness` and
  `tool_surface_readiness_for_definition_id`: keep readiness honest and do not
  promote all registered surfaces from pattern evidence alone.
- `apps/runenwerk_editor/src/shell/providers/mod.rs` module `providers`: host
  provider frames through existing registry and route contracts, without
  moving reusable pattern truth into the app.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab` module
  `editor_ux_story_lab`: extend app-owned manifests and artifacts for product
  pattern reports, focus/accessibility/diagnostics/timing evidence, and native
  screenshot or platform-impossible reports.

## Non-Goals

- Do not implement `PM-EDITOR-UX-007` all-registered-surface certification.
- Do not implement `PM-EDITOR-UX-008` game UI readiness seams.
- Do not claim final local-native no-gap certification from pattern evidence.
- Do not move editor product pattern source truth into app provider files.
- Do not make generic UI crates depend on editor shell, provider, or runtime
  vocabulary.
- Do not reopen graph canvas productization unless a focused regression appears
  in the shared Story Lab manifest.
- Do not use WR-116 as a replacement for WR-024 Interaction V2 shell-chrome
  policy.

## Acceptance Criteria For Promotion Planning

- `PM-EDITOR-UX-006` references this active design contract as a design gate and
  is `ready_next`.
- `WR-116` is in the active roadmap source, not the deferred source, with
  dependencies on completed `WR-113`, `WR-114`, and `WR-115`.
- `WR-116` is `ready_next`, not `current_candidate`, after this action.
- `task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116`
  classifies the next action as promotion planning or an exact metadata repair.
- No Rust, RON fixture, asset, provider, or product implementation code changes
  are introduced by this design-first action.

## Promotion Readiness Contract

After the design-first action validated, `task ai:goal -- --track
PT-EDITOR-UX` reported `PM-EDITOR-UX-006` as `ready_next` with next legal action
`prepare_wr_promotion_contract`. The required bridge command:

```text
task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116
```

reported:

```text
Production milestone state: ready_next
Roadmap planning_state: ready_next
Roadmap blocker: B2
Roadmap dependencies: WR-113:completed, WR-114:completed, WR-115:completed
Next action: write_promotion_contract
Promotion preflight: promotable
```

WR-116 can be promoted because:

- its required production design gates are active;
- `WR-113`, `WR-114`, and `WR-115` are completed prerequisites;
- the WR row is B2 and therefore at the implementation promotion gate;
- the implementation decision gate is this active contract;
- write scopes exist and stay within the editor/product-pattern ownership
  boundary;
- no current-candidate row blocks the WR-116 write scope;
- no product code change is needed to promote the row.

Accepted promotion command:

```text
task roadmap:promote -- --id WR-116 --state current_candidate --evidence "Accepted PM-EDITOR-UX-006 shell and product pattern polish design and promotion contract at docs-site/src/content/docs/reports/implementation-plans/wr-116-shell-and-product-pattern-polish/plan.md; completed WR-113 layered design-system, WR-114 standalone UI Designer workbench, and WR-115 graph canvas productization closeouts; production plan preflight status promotable."
```

Promotion must be followed by:

```text
task roadmap:render
task production:render
task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116
task planning:validate
task puml:validate
git diff --check
task ai:goal -- --track PT-EDITOR-UX
```

The expected next action after promotion is an implementation contract, not
direct product code.

Promotion-contract action closeout on 2026-05-25:

- `task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116`
  passed and reported `Next action: write_promotion_contract` with promotion
  preflight status `promotable`;
- `task docs:validate` passed;
- `task planning:validate` passed, including roadmap validate/check,
  production validate/check, and docs validation;
- `task puml:validate` passed;
- `git diff --check` passed;
- no product code changed in the promotion-contract action.

Promotion execution evidence on 2026-05-25:

- `task roadmap:promote -- --id WR-116 --state current_candidate --evidence
  "Accepted PM-EDITOR-UX-006 shell and product pattern polish design and
  promotion contract at
  docs-site/src/content/docs/reports/implementation-plans/wr-116-shell-and-product-pattern-polish/plan.md;
  completed WR-113 layered design-system, WR-114 standalone UI Designer
  workbench, and WR-115 graph canvas productization closeouts; production plan
  preflight status promotable."` passed and promoted `WR-116` to
  `current_candidate`;
- `task roadmap:render` and `task production:render` refreshed generated
  roadmap and production views after promotion;
- `task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116`
  passed after promotion and reported `Next action:
  write_implementation_contract`;
- `task planning:validate`, `task puml:validate`, and `git diff --check`
  passed after promotion.

## Implementation Contract

`task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116`
reported `WR-116` as `current_candidate` with next action
`write_implementation_contract`. Product code remains blocked until this
implementation contract validates and `task ai:goal -- --track PT-EDITOR-UX`
is rerun.

Bounded implementation goal:

- add typed editor-shell product-pattern evidence for the pattern families
  listed by `PM-EDITOR-UX-006`;
- register Story Lab stories or host scenarios that exercise those patterns
  through visible retained UI, scenario states, interactions, focus,
  accessibility, diagnostics, timing, and native capture evidence;
- extend the app-owned Story Lab manifest so pattern evidence cannot be claimed
  without the required artifact families;
- keep all-surface certification, game UI readiness, and final no-gap
  certification out of scope.

Implementation steps:

1. Add editor-owned product-pattern evidence contracts in
   `domain/editor/editor_shell/src/story_lab` module `story_lab`, with exact
   pattern kinds, required state coverage, route evidence, focus/keyboard
   evidence, diagnostics/overflow evidence, and native evidence checks.
2. Add a retained product-pattern story that covers inspector, palette,
   diagnostics, preview, table, tree, tab, toolbar/status, split/dock,
   empty/loading/error/degraded, long-text, disabled/readonly, dense, and
   overflow states. The story must be built from typed retained UI nodes and
   existing editor-shell composition helpers, not from app-provider text dumps.
3. Extend `domain/editor/editor_shell/src/story_lab/story.rs` module `story`
   so `EditorUxStory` can carry product-pattern evidence in the same optional,
   typed style used for design-system, workbench, and graph evidence.
4. Extend `domain/editor/editor_shell/src/story_lab/catalog.rs` module
   `catalog` so the default catalog includes the product-pattern story and
   rejects missing interactions.
5. Extend `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs`
   module `manifest` so `EditorUxEvidenceManifest::validate` requires typed
   pattern evidence and the artifact families for product-pattern claims.
6. Extend `apps/runenwerk_editor/src/shell/editor_ux_story_lab/runner.rs`
   module `runner` so the app-owned runner emits pattern report, focus,
   accessibility, diagnostics, timing, and native screenshot or
   platform-impossible artifacts.
7. Add focused tests in `editor_shell` for story registration and evidence
   coverage, and in `runenwerk_editor` for manifest rejection and PM006
   evidence generation.
8. Generate PM006 artifacts under
   `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-006-shell-and-product-pattern-polish/artifacts/`,
   write the closeout, and update production/roadmap metadata only after
   focused validation passes.

Public API and data-flow impact:

- Public Story Lab model changes stay under `domain/editor/editor_shell` and are
  exported only as editor UX evidence contracts.
- App-owned manifest model changes stay under
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab`.
- Generic `domain/ui` crates are not changed by this WR slice unless a focused
  test exposes a generic widget bug. Any such bug must be fixed as a narrowly
  scoped dependency repair, not by moving editor pattern semantics into
  `domain/ui`.
- No persistence migration is expected.
- Diagnostics must fail closed: a story with product-pattern evidence but
  missing pattern artifacts, interactions, required states, or native proof must
  fail manifest validation.

## Critical Review Gate

Source truth:

- `domain/editor/editor_shell/src/story_lab` owns the reusable product-pattern
  evidence contract.
- `domain/editor/editor_shell/src/composition` owns retained composition
  adapters for inspector, toolbar/status, console/diagnostics, split/dock, and
  product surfaces.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab` owns evidence execution
  and artifact emission.
- Generated closeout artifacts are runtime proof, not source truth.

Required source-to-runtime chain:

```text
Editor product pattern contract
  -> retained editor-shell story fixture
  -> visible-widget scan and scenario matrix
  -> app-owned Story Lab run
  -> product-pattern evidence manifest
  -> native screenshot or platform-impossible report
  -> focus, accessibility, diagnostics, timing, and runtime-proof artifacts
  -> closeout evidence and roadmap/production completion metadata
```

The implementation is insufficient if it stops at names, labels, provider
status rows, screenshots without typed evidence, or a single happy-path retained
tree. The manifest must reject incomplete pattern claims.

Architecture guards:

- `editor_shell` tests must prove the product-pattern story includes all PM006
  pattern families and required states.
- `runenwerk_editor` tests must reject product-pattern evidence that lacks a
  pattern report, focus report, accessibility report, diagnostics snapshot,
  timing report, or native/platform proof.
- PM006 closeout must preserve known gaps for PM007, PM008, and PM009.

## Perfectionist Closeout Audit

Expected completion quality for `PM-EDITOR-UX-006`: `runtime_proven`.

`perfectionist_verified` is not allowed for PM006 because all-surface
certification, game UI readiness, and final local-native no-gap certification
remain incomplete. The closeout must list those as known quality gaps in both
production and roadmap evidence.

Perfectionist verification is reserved for `PM-EDITOR-UX-009` after local
native screenshots, accessibility, interactions, visual/performance reports,
docs, roadmap, and production state agree with empty known gaps.

Implementation-contract action closeout on 2026-05-25:

- `task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116`
  passed and reported `WR-116` as `current_candidate` with `Next action:
  write_implementation_contract`;
- this section defines owners, exact modules, implementation steps, public API
  and data-flow impact, stop conditions, validation, closeout, and completion
  quality before product code changes;
- no product code changed in the implementation-contract action.

## Stop Conditions

Stop before implementation if:

- `task ai:goal -- --track PT-EDITOR-UX` does not select `PM-EDITOR-UX-006`
  after this action;
- `task production:plan -- --milestone PM-EDITOR-UX-006 --roadmap WR-116`
  reports a hard blocker or unmet gate;
- promotion preflight reports metadata that is not directly owned by `WR-116`;
- implementation would require changing dependency direction or command/surface
  source-truth authority without an ADR or accepted design update;
- source files changed enough that the coordinator command must be rerun.

## Closeout Requirements For The Later Implementation

The later implementation closeout must include:

- completed closeout path under
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-006-shell-and-product-pattern-polish/closeout.md`;
- evidence artifacts proving product pattern reports, focus traversal,
  accessibility, diagnostics, timing, and native screenshot or
  platform-impossible reports;
- focused `editor_shell` and `runenwerk_editor` tests for the patterns and app
  manifest;
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`;
- `task production:render`, `task production:validate`, and
  `task production:check`;
- `task planning:validate`, `task docs:validate`, `task puml:validate`, and
  `git diff --check`;
- an explicit known-gap handoff stating that `PM-EDITOR-UX-007`,
  `PM-EDITOR-UX-008`, and `PM-EDITOR-UX-009` remain incomplete.
