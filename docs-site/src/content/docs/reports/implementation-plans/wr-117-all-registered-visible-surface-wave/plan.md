---
title: WR-117 All Registered Visible Surface Wave Design Contract
description: Design-first contract for PM-EDITOR-UX-007 all registered visible editor surface readiness, Story Lab evidence, and native manifest proof.
status: active
owner: editor
layer: domain/editor / app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-story-lab-and-game-ui-ready-foundations-design.md
related_reports:
  - ../wr-114-standalone-ui-designer-workbench/plan.md
  - ../wr-115-graph-canvas-and-node-editor-productization/plan.md
  - ../wr-116-shell-and-product-pattern-polish/plan.md
  - ../../closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md
  - ../../closeouts/pm-editor-ux-005-graph-canvas-and-node-editor-productization/closeout.md
  - ../../closeouts/pm-editor-ux-006-shell-and-product-pattern-polish/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# WR-117 All Registered Visible Surface Wave Design Contract

## Goal

Clear the design-first blocker for `PM-EDITOR-UX-007` and prepare `WR-117` for
promotion planning. This action is planning and metadata only. It does not
change editor product code.

This contract is produced from:

```text
task production:plan -- --milestone PM-EDITOR-UX-007 --roadmap WR-117
```

Expected production outcome for the later implementation slice:

- every registered user-facing editor surface and explicit diagnostic/fallback
  surface has a typed readiness decision;
- product surfaces have Story Lab scenarios, visible-widget scans, state
  coverage, interaction routes, provider evidence, focus/accessibility reports,
  diagnostics snapshots, timing proof, and native screenshot or typed
  platform-impossible evidence;
- diagnostic, fallback-only, and hidden-until-productized surfaces carry
  explicit reasons and cannot be mistaken for certified product workflows;
- normal product workflows expose no generic text/action panels, placeholder
  product surfaces, unsupported provider dumps, or misleading visible fallbacks;
- the slice does not claim game-runtime UI readiness or final no-gap
  certification. Those remain `PM-EDITOR-UX-008` and `PM-EDITOR-UX-009`.

## Source Of Truth

- Production milestone: `PM-EDITOR-UX-007` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-117` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml` after this design
  action is accepted.
- Active editor UX doctrine:
  `docs-site/src/content/docs/design/active/editor-product-ux-story-lab-and-game-ui-ready-foundations-design.md`.
- Completed standalone UI Designer prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md`.
- Completed graph canvas prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-005-graph-canvas-and-node-editor-productization/closeout.md`.
- Completed shell and product pattern prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-006-shell-and-product-pattern-polish/closeout.md`.
- Registered surface source truth:
  `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `editor_surface_definitions`, `tool_surface_definition_id`,
  `tool_surface_readiness`, and `tool_surface_readiness_for_definition_id`.
- Registered-surface Story Lab source:
  `domain/editor/editor_shell/src/story_lab/catalog.rs` function
  `surface_story` and module `domain/editor/editor_shell/src/story_lab`.
- Story metadata source:
  `domain/editor/editor_shell/src/story_lab/story.rs` struct
  `EditorUxStory` and enum `EditorUxStoryKind`.
- App-owned Story Lab evidence source:
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs` struct
  `EditorUxEvidenceManifest` and function `EditorUxEvidenceManifest::validate`.
- App-owned visible-widget scan adapter:
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/visible_widget_scan.rs`
  function `scan_editor_ux_story`.
- App-owned provider execution:
  `apps/runenwerk_editor/src/shell/providers/mod.rs` module `providers` and
  provider-specific modules under `apps/runenwerk_editor/src/shell/providers`.

## Readiness

`task production:plan -- --milestone PM-EDITOR-UX-007 --roadmap WR-117`
reported:

```text
Production milestone state: designing
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-114:completed, WR-115:completed, WR-116:completed
Next action: design_first
```

Prerequisites are complete. Implementation remains illegal until this contract
validates, `PM-EDITOR-UX-007` is moved to `ready_next`, `WR-117` is moved out
of deferred planning, and `task ai:goal -- --track PT-EDITOR-UX` is rerun. The
next legal action after that rerun is promotion-readiness planning, not product
code.

## Promotion And Implementation-Readiness Contract

`task production:plan -- --milestone PM-EDITOR-UX-007 --roadmap WR-117`
reported the ready-next state as promotable after this design contract was
accepted:

```text
Production milestone state: ready_next
Roadmap planning_state: ready_next
Roadmap blocker: B2
Roadmap dependencies: WR-114:completed, WR-115:completed, WR-116:completed
Next action: write_promotion_contract
Promotion preflight status: promotable
```

Promotion is allowed only with this evidence:

- `PM-EDITOR-UX-007` is `ready_next`.
- `WR-117` is `ready_next`.
- Dependencies `WR-114`, `WR-115`, and `WR-116` are completed with closeout
  evidence.
- This contract is active and names source truth, ownership, non-goals,
  implementation scope, validation, stop conditions, and closeout requirements.
- Product code remains unchanged by the design and promotion actions.

Promotion evidence string:

```text
Accepted PM-EDITOR-UX-007 all registered visible surface wave design and promotion contract at docs-site/src/content/docs/reports/implementation-plans/wr-117-all-registered-visible-surface-wave/plan.md; completed WR-114 standalone UI Designer workbench, WR-115 graph canvas productization, and WR-116 shell product pattern polish closeouts; production plan preflight status promotable.
```

After promotion, the next legal action is to write a narrowed implementation
contract before product code changes. That implementation contract must name:

- the exact registered-surface evidence type and owner;
- the exact provider families that need repair, certification, fallback proof,
  or hidden-surface proof;
- the exact manifest validation changes and artifact kinds;
- the exact generated PM007 artifact paths;
- focused tests for `editor_shell` surface/story coverage and
  `runenwerk_editor` surface/provider evidence.

Do not start implementation from this design/promote action.

## Implementation Contract

`task production:plan -- --milestone PM-EDITOR-UX-007 --roadmap WR-117`
reported the promoted current-candidate state as:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-114:completed, WR-115:completed, WR-116:completed
Next action: write_implementation_contract
```

The implementation may start only after this contract validates and
`task ai:goal -- --track PT-EDITOR-UX` is rerun.

### Bound Slice

The coding pass must certify the registered-surface evidence chain, not polish
unrelated UI:

1. Add typed registered-surface evidence in
   `domain/editor/editor_shell/src/story_lab/surface_wave.rs`.
2. Attach that evidence to every registered-surface story from
   `domain/editor/editor_shell/src/story_lab/catalog.rs` function
   `surface_story`.
3. Extend `domain/editor/editor_shell/src/story_lab/story.rs` so
   `EditorUxStory` can carry registered-surface evidence.
4. Extend `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs`
   so `EditorUxEvidenceManifest::validate` rejects registered-surface stories
   without the typed evidence and required artifacts.
5. Extend `apps/runenwerk_editor/src/shell/editor_ux_story_lab/runner.rs`
   function `run_story` so PM007 evidence artifacts can be generated from the
   app-owned Story Lab harness.
6. Add runtime PM007 proof under
   `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-007-all-registered-visible-surface-wave/artifacts/`
   only from the app-owned test harness.

Provider fixes are allowed only when a test proves that a currently visible
registered surface cannot honestly satisfy its readiness category. Any provider
fix must stay under `apps/runenwerk_editor/src/shell/providers` and preserve the
editor-shell readiness source truth.

### Exact Evidence Contract

Registered-surface evidence must include:

- surface definition ID;
- semantic key;
- display name;
- readiness category;
- whether the surface is visible in product workflows;
- required artifact kinds;
- required state kinds;
- required route kinds;
- required provider/native evidence checks;
- an explicit reason for diagnostic, fallback-only, or hidden surfaces.

Product surfaces must require retained UI, native screenshot or
platform-impossible evidence, visible-widget scan, focus traversal,
accessibility, diagnostics, timing, and provider/runtime proof.

Diagnostic and fallback-only surfaces must require retained UI, diagnostics or
fallback reason proof, and must not satisfy product requirements.

Hidden-until-productized surfaces must prove they remain registered but are not
certified product workflows.

### Focused Tests To Add Or Extend

- `domain/editor/editor_shell/src/story_lab/surface_wave.rs` module
  `surface_wave`: evidence covers every `editor_surface_definitions` entry and
  records readiness-specific requirements.
- `domain/editor/editor_shell/src/story_lab/catalog.rs` tests: default catalog
  has registered-surface evidence for every registered surface and does not let
  hidden surfaces claim product interactions.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs` tests:
  manifest validation rejects missing registered-surface evidence, missing
  required product artifacts, and unexpected surface evidence.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/runner.rs` tests:
  PM007 evidence generation emits a manifest and runtime proof summary.
- `apps/runenwerk_editor/src/shell/providers/tests.rs` tests: provider
  stable-key coverage remains intact for every provider family named by the
  surface-wave evidence.

### Validation For Implementation Closeout

Run at minimum:

```text
cargo fmt --all
cargo test -p editor_shell surface -- --nocapture
cargo test -p editor_shell story_lab -- --nocapture
cargo test -p runenwerk_editor surface -- --nocapture
cargo test -p runenwerk_editor editor_ux -- --nocapture
cargo test -p runenwerk_editor providers -- --nocapture
RUNENWERK_WRITE_PM_EDITOR_UX_007_EVIDENCE=1 cargo test -p runenwerk_editor pm_editor_ux_007 -- --nocapture
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

If focused validation reveals broader app shell breakage, add
`cargo test -p runenwerk_editor shell -- --nocapture` before closeout.

### Implementation Stop Conditions

Stop and report instead of coding around the issue if:

- any surface cannot be assigned a readiness category from editor-shell source
  truth;
- app provider behavior would need to become the readiness source;
- satisfying PM007 would require changing persistence compatibility or deleting
  registered surfaces without an accepted migration path;
- the manifest can only pass by accepting descriptor-only, screenshot-only,
  fallback-only, provider-line-dump, or status-panel-only proof;
- the work starts to implement game-runtime UI, safe-area/game target profiles,
  or final no-gap certification.

## Architecture Governance Review

Recommendation: clear the design blocker and then run promotion planning before
implementation. Do not implement product code in this action.

DDD owner:

- `domain/editor/editor_shell` owns editor surface registry truth, readiness
  vocabulary, registered-surface Story Lab contracts, surface-specific product
  semantics, and hide-or-certify policy.
- `apps/runenwerk_editor` owns provider execution, native Story Lab runs,
  provider fixtures, screenshots or platform-impossible artifacts,
  focus/accessibility/timing reports, diagnostics snapshots, and evidence
  manifests.
- `domain/ui` owns generic retained UI primitives, visible-widget scanning,
  layout/input contracts, target profiles, and graph substrate. It must not
  grow editor surface vocabulary or provider-specific semantics.

Vocabulary and invariants:

- Registered surface means an entry returned by
  `editor_surface_definitions`.
- Product surface means a surface that may appear in normal product workflows
  only after Story Lab evidence and app-owned native proof validate.
- Diagnostic surface means a visible surface whose purpose is diagnosis and
  whose evidence proves diagnostic honesty rather than product workflow
  completion.
- Fallback-only surface means a visible fallback with an explicit non-product
  reason. It must not look like a completed product surface.
- Hidden-until-productized means the surface may remain registered and available
  to source-truth code, but it must not appear as a normal visible product
  workflow until a later WR certifies it.
- Descriptor-only, retained-preview-only, status-panel-only, provider-line-dump,
  screenshot-only, or fallback-only evidence cannot certify a product surface.

Dependency direction:

```text
domain/ui -> domain/editor/editor_shell -> apps/runenwerk_editor
```

`domain/ui` remains generic. `domain/editor/editor_shell` may consume generic UI
primitives and own editor surface contracts. `apps/runenwerk_editor` may host,
execute, and prove registered-surface stories, but it must not become the source
of surface readiness truth.

Translation boundaries:

- `surface_contract.rs` translates editor-owned surface definitions into
  readiness decisions.
- `story_lab/catalog.rs` translates each surface definition into a registered
  surface story with scenario, scan, and interaction expectations.
- App providers translate runtime state into retained UI frames and provider
  diagnostics.
- `editor_ux_story_lab` translates stories and provider frames into manifest
  artifacts and validation proof.

ADR need: no ADR is required for a bounded all-surface certification wave that
preserves current source-truth ownership and dependency direction. Require an
ADR or accepted design update before changing surface source-truth ownership,
making app providers the readiness source, moving native evidence capture into
`domain/ui` or `domain/editor`, changing persistence compatibility for
registered surfaces, or coupling future game-runtime UI to editor-shell
vocabulary.

ATAM-lite:

- Quality attributes in tension: completeness across all registered surfaces,
  product polish, diagnostic honesty, future migration space, provider
  reliability, local-native evidence, and avoiding scope creep into game UI.
- Chosen option: certify or hide every registered visible surface using typed
  readiness and evidence tiers rather than promoting all surfaces to product at
  once.
- Sensitivity points: treating hidden surfaces as complete, certifying provider
  text output as product UI, accepting manifest runs without native/platform
  evidence, and changing readiness decisions without provider/story proof.
- Risk: broad surface work can become a grab bag. The later implementation must
  stay registry-driven and reject unclassified or evidence-free surfaces rather
  than polishing isolated panels.

Fitness functions before implementation:

- `cargo test -p editor_shell surface -- --nocapture`
- `cargo test -p editor_shell story_lab -- --nocapture`
- `cargo test -p runenwerk_editor surface -- --nocapture`
- `cargo test -p runenwerk_editor editor_ux -- --nocapture`
- `cargo test -p runenwerk_editor providers -- --nocapture`
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

Ownership mode: stream-aligned editor product certification with editor-shell
registry support and app-hosted evidence support.

## Implementation Scope For The Later Slice

The later promoted implementation may change these owners and only these owners
unless the promotion contract narrows them further:

- `domain/editor/editor_shell/src/story_lab/surface_wave.rs` module
  `surface_wave`: add typed registered-surface evidence contracts for readiness
  category, surface definition ID, stable key, provider family, required
  scenarios, route kinds, state coverage, and native evidence checks.
- `domain/editor/editor_shell/src/story_lab/mod.rs` module `story_lab`: export
  the new surface-wave evidence module.
- `domain/editor/editor_shell/src/story_lab/story.rs` struct `EditorUxStory`:
  add optional registered-surface evidence and a builder method analogous to
  existing design-system, graph-canvas, workbench, and product-pattern
  evidence.
- `domain/editor/editor_shell/src/story_lab/catalog.rs` function
  `surface_story`: attach typed surface evidence to every
  `EditorUxStoryKind::RegisteredSurface` story and keep hidden surfaces from
  claiming product interactions.
- `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `editor_surface_definitions`, `tool_surface_readiness`, and
  `tool_surface_readiness_for_definition_id`: keep readiness decisions explicit
  and reject unclassified new visible surfaces through tests.
- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs` enum
  `EditorLabEvidenceArtifactKind`: add a surface-readiness artifact only if the
  manifest needs a distinct artifact family beyond existing retained UI,
  provider, diagnostics, accessibility, timing, and native evidence artifacts.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs` struct
  `EditorUxEvidenceRun` and function `EditorUxEvidenceManifest::validate`:
  validate typed surface evidence for every registered surface story, with
  stricter artifact requirements for product surfaces and explicit reason
  checks for fallback, diagnostic, and hidden surfaces.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/runner.rs` function
  `run_story`: emit surface-wave artifacts and PM007 runtime proof.
- `apps/runenwerk_editor/src/shell/providers` modules: repair only providers
  whose registered surface readiness requires a real product, diagnostic,
  fallback, or hidden proof path. Provider repairs must be named in the
  promotion contract before code changes.

## Non-Goals

- Do not implement game-runtime UI or create a game UI owner crate.
- Do not claim `perfectionist_verified`; PM009 owns final no-gap certification.
- Do not change `domain/ui` to know editor surface names, provider families, or
  readiness vocabulary.
- Do not make `apps/runenwerk_editor` the source truth for readiness
  classification.
- Do not turn hidden surfaces into product surfaces without product scenarios,
  provider proof, and native or platform-impossible evidence.
- Do not remove registered surfaces solely to pass the wave unless the
  promotion contract explicitly accepts a migration or archival path.

## Acceptance Criteria For The Later Slice

- Every `editor_surface_definitions` entry has typed registered-surface
  evidence in the default Editor UX Story Lab catalog.
- Product surface stories require strict visible-widget scan states, interaction
  routes, focus reachability, accessibility labels, overflow policy, diagnostics
  snapshots, timing proof, and native screenshot or platform-impossible proof.
- Diagnostic and fallback-only surface stories carry explicit diagnostic or
  fallback reasons and cannot satisfy product evidence requirements.
- Hidden-until-productized surfaces remain registered but have evidence proving
  they are not visible as normal product workflows.
- App-owned evidence manifests reject missing stories, missing surface evidence,
  unexpected surface evidence, unclassified surfaces, and product stories that
  only have retained UI debug artifacts.
- Provider support-mode tests cover stable-key registration for every provider
  family touched by the wave.
- The closeout names remaining gaps for PM008 and PM009.

## Stop Conditions

Stop before product code if:

- a surface readiness decision requires changing source-truth ownership,
  provider ownership, persistence compatibility, or dependency direction without
  an accepted ADR or design update;
- a registered surface cannot be classified as product, diagnostic,
  fallback-only, or hidden-until-productized from current source truth;
- the implementation would require broad app-only styling without editor-shell
  Story Lab contracts;
- a product surface lacks a provider path or native/platform evidence path;
- validation fails, closeout evidence cannot be generated, or roadmap/production
  render checks drift after metadata updates.

## Closeout Requirements

The later implementation closeout must include:

- a completed closeout at
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-007-all-registered-visible-surface-wave/closeout.md`;
- app-owned PM007 artifacts under that closeout's `artifacts/` directory,
  including the evidence manifest and a runtime proof summary;
- exact files, modules, functions, and provider families changed;
- focused validation output for editor-shell surface/story tests and app surface
  evidence tests;
- roadmap archive metadata for `WR-117` and completed production metadata for
  `PM-EDITOR-UX-007`;
- known gap handoff to `PM-EDITOR-UX-008` and `PM-EDITOR-UX-009`.

## Perfectionist Closeout Audit

Expected completion quality for the later slice is `runtime_proven`.

`perfectionist_verified` is not allowed for PM007 because game UI readiness and
final local-native no-gap certification remain open. A PM007 closeout may only
claim all registered visible editor surfaces are certified, diagnostic,
fallback-only, or hidden until productized when the manifest and runtime proof
show every registered surface is accounted for.

Known quality gaps that must remain visible after PM007:

- `PM-EDITOR-UX-008` still owns game UI readiness seam proof.
- `PM-EDITOR-UX-009` still owns final local-native no-gap certification and any
  `perfectionist_verified` claim.

Anti-drift guards must reject descriptor-only, retained-preview-only,
status-panel-only, provider-line-dump, fallback-only, screenshot-only,
unclassified, or unconsumed-contract completion claims.
