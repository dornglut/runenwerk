---
title: WR-118 Game UI Readiness Seam Design Contract
description: Design-first contract for PM-EDITOR-UX-008 target-profile compatibility evidence without editor-shell coupling or game HUD implementation.
status: active
owner: editor
layer: domain/ui / domain/editor
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
related_reports:
  - ../wr-117-all-registered-visible-surface-wave/plan.md
  - ../../closeouts/pm-editor-ux-007-all-registered-visible-surface-wave/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# WR-118 Game UI Readiness Seam Design Contract

## Goal

Clear the design-first blocker for `PM-EDITOR-UX-008` and prepare `WR-118` for
promotion planning. This action is planning and metadata only. It does not
change product code and does not implement game-runtime HUD behavior.

This contract is produced from:

```text
task production:plan -- --milestone PM-EDITOR-UX-008 --roadmap WR-118
```

Expected production outcome for the later implementation slice:

- generic UI target-profile, recipe, binding, fixture, persistence, and layout
  validators can prove `game.runtime` compatibility axes;
- editor Story Lab evidence can describe future game-runtime compatibility
  without importing editor shell vocabulary into game-runtime contracts;
- safe-area/layout axes, input modality axes, platform prompts,
  localization/text expansion, accessibility modes, split-screen-like sizing,
  performance/readability budgets, and stale/missing view-model diagnostics are
  represented as compatibility evidence;
- no editor shell, Workbench host policy, editor provider vocabulary, or editor
  command route enters game-runtime UI contracts;
- no game HUD runtime behavior is implemented in this milestone.

## Source Of Truth

- Production milestone: `PM-EDITOR-UX-008` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-118` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml` after this design
  action is accepted.
- Active editor UX doctrine:
  `docs-site/src/content/docs/design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md`.
- Active game-runtime UI doctrine:
  `docs-site/src/content/docs/design/active/game-runtime-ui-projection-and-hud-platform-design.md`.
- Completed all-surface prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-007-all-registered-visible-surface-wave/closeout.md`.
- Generic UI recipe target-profile validation:
  `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe`.
- Generic UI preview fixture and evidence descriptor validation:
  `domain/ui/ui_definition/src/preview_fixture/mod.rs` module
  `preview_fixture`.
- Generic UI production readiness evidence validation:
  `domain/ui/ui_definition/src/production_readiness/mod.rs` module
  `production_readiness`, especially `UiReadinessEvidencePacket`,
  `UiReadinessInspectionReport`, `UiReadinessRequest`, and
  `validate_production_readiness`.
- Generic UI view binding target-profile validation:
  `domain/ui/ui_definition/src/view_binding/mod.rs` module `view_binding`.
- Generic UI persistence activation target-profile validation:
  `domain/ui/ui_definition/src/persistence_activation/mod.rs` module
  `persistence_activation`.
- Generic UI visual-layout target-profile validation:
  `domain/ui/ui_definition/src/visual_layout` modules `apply` and
  `operation`.
- Editor Story Lab scenario axes:
  `domain/editor/editor_shell/src/story_lab/scenario.rs` module `scenario`.

## Readiness

`task production:plan -- --milestone PM-EDITOR-UX-008 --roadmap WR-118`
reported:

```text
Production milestone state: designing
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-117:completed
Next action: design_first
```

The all-surface prerequisite is complete. Implementation remains illegal until
this contract validates, `PM-EDITOR-UX-008` is moved to `ready_next`, `WR-118`
is moved out of deferred planning, and `task ai:goal -- --track PT-EDITOR-UX`
is rerun. The next legal action after that rerun should be promotion-readiness
planning, not product code.

## Promotion And Implementation-Readiness Contract

`task production:plan -- --milestone PM-EDITOR-UX-008 --roadmap WR-118`
reported the ready-next state as promotable after this design contract was
accepted:

```text
Production milestone state: ready_next
Roadmap planning_state: ready_next
Roadmap blocker: B2
Roadmap dependencies: WR-117:completed
Next action: write_promotion_contract
Promotion preflight status: promotable
```

Promotion is allowed only with this evidence:

- `PM-EDITOR-UX-008` is `ready_next`.
- `WR-118` is `ready_next`.
- Dependency `WR-117` is completed with closeout evidence.
- This contract is active and names source truth, ownership, non-goals,
  implementation scope, validation, stop conditions, and closeout requirements.
- Product code remains unchanged by the design and promotion actions.

Promotion evidence string:

```text
Accepted PM-EDITOR-UX-008 game UI readiness seam design and promotion contract at docs-site/src/content/docs/reports/implementation-plans/wr-118-game-ui-readiness-seam/plan.md; completed WR-117 all registered visible surface wave closeout; production plan preflight status promotable.
```

After promotion, the next legal action is to write a narrowed implementation
contract before product code changes. That implementation contract must name:

- the exact generic UI evidence descriptor type and owner;
- the exact target-profile compatibility axes and fail-closed checks;
- the exact editor Story Lab compatibility report owner, if one is needed;
- the exact tests proving no editor-shell vocabulary enters `game.runtime`
  contracts;
- the exact generated PM008 artifact paths, if runtime evidence artifacts are
  required by the implementation slice.

Do not start implementation from this design/promote action.

## Architecture Governance Review

Recommendation: clear the design blocker and then run promotion planning before
implementation. Do not implement product code in this action.

DDD owner:

- `domain/ui/ui_definition` owns generic target-profile IDs, recipe
  compatibility, view binding compatibility, preview fixture/evidence
  descriptors, persistence activation compatibility, and visual-layout target
  profile validation.
- `domain/editor/editor_shell` owns editor Story Lab scenarios and may express
  compatibility evidence descriptors for future game-runtime use, but it must
  not become a game-runtime UI source-truth owner.
- `PT-GAME-RUNTIME-UI` owns future game-runtime UI projection and HUD behavior.

Vocabulary and invariants:

- `game.runtime` is a target profile, not an editor shell profile.
- Compatibility evidence proves generic UI contracts can represent future
  game-runtime UI axes; it does not implement a HUD.
- Editor Story Lab evidence may reference target-profile compatibility axes,
  but game-runtime contracts must not import editor workbench, provider,
  surface, command, or shell vocabulary.
- Generic UI validators must fail closed for unsupported target profiles,
  missing view-model packages, stale descriptors, and unsupported evidence
  descriptors.

Dependency direction:

```text
domain/ui -> domain/editor/editor_shell -> apps/runenwerk_editor
```

PM008 may add generic `domain/ui` contracts and editor-side Story Lab
compatibility descriptors. It must not create a reverse dependency from
`domain/ui` to editor shell and must not add a game-runtime UI owner crate.

ADR need: no ADR is required for a bounded compatibility seam that preserves
existing target-profile ownership and does not create runtime HUD behavior.
Require an ADR or accepted design update before adding a game-runtime UI owner
crate, moving game-runtime UI source truth into editor shell, or making editor
shell vocabulary a dependency of `game.runtime` contracts.

ATAM-lite:

- Quality attributes in tension: future game UI compatibility, generic UI
  purity, editor evidence reuse, fail-closed validation, and avoiding premature
  HUD implementation.
- Chosen option: prove compatibility through target-profile evidence
  descriptors and validators, then leave runtime HUD implementation to
  `PT-GAME-RUNTIME-UI`.
- Sensitivity points: accepting editor-specific vocabulary as generic,
  treating examples as runtime HUD behavior, and allowing compatibility to pass
  without stale/missing view-model diagnostics.

Fitness functions before implementation:

- `cargo test -p ui_definition game -- --nocapture`
- `cargo test -p editor_shell game -- --nocapture`
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

Ownership mode: enabling UI platform seam for future stream-aligned game UI
work.

## Implementation Contract

`task production:plan -- --milestone PM-EDITOR-UX-008 --roadmap WR-118`
reported the promoted current-candidate state as:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-117:completed
Next action: write_implementation_contract
```

The implementation may start only after this contract validates and
`task ai:goal -- --track PT-EDITOR-UX` is rerun.

### Bound Slice

The coding pass must prove `game.runtime` compatibility through generic UI
contracts and an editor-side compatibility report. It must not implement game
HUD behavior.

1. In `domain/ui/ui_definition/src/preview_fixture/mod.rs` module
   `preview_fixture`, extend the preview matrix contract so a
   `game.runtime` compatibility matrix can explicitly cover safe area, input
   modality, platform prompt, localization/text expansion, accessibility, size,
   performance/readability, and view-model freshness axes. Existing
   `UiPreviewMatrixAxisKind`, `UiPreviewEvidenceDescriptor`, and
   `validate_preview_fixtures` remain the owning API surface.
2. In `domain/ui/ui_definition/src/production_readiness/mod.rs` module
   `production_readiness`, add typed axis coverage to generic readiness
   evidence. The exact new public type is `UiReadinessCompatibilityAxis`,
   owned by `ui_definition`, and `validate_production_readiness` must reject
   `game.runtime` readiness packets that omit required compatibility axes,
   required evidence kinds, fresh external artifact references, or diagnostic
   inspection.
3. In `domain/ui/ui_definition/src/component_recipe/mod.rs` module
   `component_recipe`, use `expand_ui_recipe` to prove recipe expansion works
   for `game.runtime` without editor profile assumptions and without weakening
   accessibility, layout, focus, token, or slot validation.
4. In `domain/ui/ui_definition/src/view_binding/mod.rs` module `view_binding`,
   keep view-model packets read-only and extend `validate_ui_bindings` so
   `UiIntentDescriptorRef::EditorCommand` is rejected for `game.runtime`
   declarations. Runtime declarations must use `UiIntentDescriptorRef::GameIntent`
   or another non-editor descriptor and must still fail closed for missing or
   stale view-model packages.
5. In `domain/ui/ui_definition/src/persistence_activation/mod.rs` module
   `persistence_activation`, keep `validate_persistence_activation` fail-closed
   for unsupported target profiles, missing migration reports, missing diffs,
   non-deterministic diffs, and unsafe unknown-field policy for `game.runtime`.
6. In `domain/ui/ui_definition/src/visual_layout` modules `operation` and
   `apply`, use `UiVisualLayoutEditContext::with_supported_target_profiles` and
   `apply_visual_layout_operation` to prove game-runtime layout edits are
   profile-gated and do not require editor shell context.
7. In `domain/editor/editor_shell/src/story_lab/game_ui_readiness.rs` module
   `game_ui_readiness`, add the editor-owned compatibility report type
   `EditorUxGameUiReadinessEvidence` and builder function
   `game_ui_readiness_evidence`. The report may aggregate generic UI
   validation result codes and axis coverage, but it must not become source
   truth for game-runtime UI semantics.
8. In `domain/editor/editor_shell/src/story_lab/mod.rs` module `story_lab`,
   export `game_ui_readiness` only as Story Lab compatibility evidence.

### Exact Evidence Contract

PM008 evidence must include:

- target profile `game.runtime`;
- explicit generic compatibility axes:
  - safe area;
  - input modality;
  - platform prompt;
  - localization/text expansion;
  - accessibility;
  - size or split-screen-like sizing;
  - performance/readability;
  - view-model freshness;
- required readiness evidence kinds:
  `CompatibilityReport`, `DiagnosticInspection`, `AccessibilityReport`,
  `PerformanceBudgetReport`, and `ExampleScenario`;
- fail-closed diagnostics for unsupported target profiles, missing axes,
  missing evidence, stale evidence, owned concrete artifacts in
  `domain/ui`, missing or stale view-model packets, editor command descriptors
  under `game.runtime`, missing persistence migration/diff evidence, and
  unsupported visual-layout edit contexts;
- editor-side Story Lab evidence that records the generic UI checks without
  importing editor shell, Workbench host, provider, surface, or editor command
  vocabulary into runtime target-profile contracts.

No generated PM008 native screenshot or HUD runtime artifact is required in
this slice. If the implementation cannot prove the seam without a concrete
runtime HUD artifact, stop and update the design or ADR path instead of adding
runtime HUD behavior under PT-EDITOR-UX.

### Focused Tests To Add Or Extend

- `domain/ui/ui_definition/src/preview_fixture/mod.rs` tests:
  `game_runtime_preview_matrix_requires_all_compatibility_axes` and
  `game_runtime_preview_matrix_rejects_editor_only_evidence`.
- `domain/ui/ui_definition/src/production_readiness/mod.rs` tests:
  `game_runtime_readiness_requires_axis_coverage_and_external_artifacts` and
  `game_runtime_readiness_rejects_missing_compatibility_report`.
- `domain/ui/ui_definition/src/view_binding/mod.rs` tests:
  `game_runtime_rejects_editor_command_descriptor` and
  `game_runtime_rejects_missing_or_stale_view_model_packages`.
- `domain/ui/ui_definition/src/component_recipe/mod.rs` tests:
  `game_runtime_recipe_expansion_preserves_accessibility_layout_and_focus`.
- `domain/ui/ui_definition/src/persistence_activation/mod.rs` tests:
  `game_runtime_persistence_activation_requires_migration_diff_and_determinism`.
- `domain/ui/ui_definition/src/visual_layout/apply.rs` tests:
  `game_runtime_visual_layout_edits_are_profile_gated`.
- `domain/editor/editor_shell/src/story_lab/game_ui_readiness.rs` tests:
  `game_ui_readiness_evidence_covers_required_axes` and
  `game_ui_readiness_evidence_contains_no_editor_runtime_vocabulary`.

### Validation For Implementation Closeout

Run at minimum:

```text
cargo fmt --all
cargo test -p ui_definition game -- --nocapture
cargo test -p editor_shell game -- --nocapture
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

### Anti-Drift Guard

The closeout must not claim PM008 from descriptor-only proof. The implemented
tests must execute the generic UI validators and the editor Story Lab evidence
builder, and the closeout must quote the validation commands that prove the
guard:

- axis coverage is typed, not ad hoc strings;
- `game.runtime` rejects editor command descriptors;
- stale or missing view-model packages produce blocking diagnostics;
- readiness artifacts remain external references, not concrete artifact
  ownership in `domain/ui`;
- visual-layout edits are accepted only by compatible target profiles;
- the Story Lab report remains editor-owned evidence, not game-runtime source
  truth.

## Implementation Scope For The Later Slice

The current-candidate implementation may change these owners and only these
owners unless this implementation contract is updated and revalidated:

- `domain/ui/ui_definition/src/preview_fixture/mod.rs` module
  `preview_fixture`: add or harden target-profile compatibility evidence
  descriptors for safe areas, input modality, platform prompts, localization,
  accessibility, split-screen-like sizing, performance/readability, and stale or
  missing view-model diagnostics.
- `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe`: prove recipe expansion supports `game.runtime` without
  editor profile assumptions.
- `domain/ui/ui_definition/src/production_readiness/mod.rs` module
  `production_readiness`: add typed compatibility-axis coverage to generic
  readiness evidence and fail closed for incomplete `game.runtime` readiness
  packets.
- `domain/ui/ui_definition/src/view_binding/mod.rs` module `view_binding`:
  prove game-runtime view-model packages remain read-only and do not consume
  editor command/provider vocabulary.
- `domain/ui/ui_definition/src/persistence_activation/mod.rs` module
  `persistence_activation`: prove target-profile activation rejects unsupported
  runtime descriptors fail-closed.
- `domain/ui/ui_definition/src/visual_layout` modules `apply` and `operation`:
  prove target-profile-compatible visual-layout edits do not require editor
  shell context.
- `domain/editor/editor_shell/src/story_lab/game_ui_readiness.rs` module
  `game_ui_readiness`: add editor-side compatibility evidence descriptors only
  if the implementation needs a Story Lab-owned seam report.
- `domain/editor/editor_shell/src/story_lab/scenario.rs` module `scenario`:
  extend scenario axes only if safe-area/input-modality compatibility cannot be
  represented by existing axes.

## Non-Goals

- Do not implement game HUD runtime behavior.
- Do not add `domain/game_ui` or another game-runtime UI owner crate.
- Do not make `domain/ui` depend on editor shell.
- Do not move editor Story Lab, Workbench host, surface registry, provider, or
  editor command vocabulary into `game.runtime` contracts.
- Do not claim final no-gap editor UX certification.

## Acceptance Criteria For The Later Slice

- `game.runtime` target-profile compatibility is proven through generic
  UI-definition validators and editor-side evidence descriptors only.
- Compatibility axes include safe area/layout, input modality, platform prompts,
  localization/text expansion, accessibility modes, split-screen-like sizing,
  performance/readability budgets, and stale/missing view-model diagnostics.
- Unsupported editor-only descriptors fail closed for `game.runtime`.
- No editor shell, Workbench host policy, editor command route, or provider
  vocabulary appears in runtime target-profile contracts.
- The closeout names `PM-EDITOR-UX-009` as the remaining final no-gap
  certification gap.

## Stop Conditions

Stop before product code if:

- proving compatibility requires adding a game-runtime UI owner crate;
- any generic UI contract must import editor shell types;
- the work requires game HUD runtime behavior;
- target-profile compatibility cannot fail closed for unsupported descriptors;
- validation fails or roadmap/production render checks drift after metadata
  updates.

## Closeout Requirements

The later implementation closeout must include:

- a completed closeout at
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-008-game-ui-readiness-seam/closeout.md`;
- exact generic UI and editor-shell files, modules, and functions changed;
- focused validation output for `ui_definition game` and `editor_shell game`;
- evidence proving no editor-shell vocabulary enters `game.runtime` contracts;
- roadmap archive metadata for `WR-118` and completed production metadata for
  `PM-EDITOR-UX-008`;
- known gap handoff to `PM-EDITOR-UX-009`.

## Perfectionist Closeout Audit

Expected completion quality for the later slice is `runtime_proven`.

`perfectionist_verified` is not allowed for PM008 because final local-native
no-gap certification remains open in PM009.
