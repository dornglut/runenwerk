---
title: WR-122 Standalone App Shell And Embedded Host Parity Contract
description: Ready-next implementation contract for PM-UI-DESIGNER-WB-003 standalone UI Designer launch and embedded Editor Design host parity.
status: active
owner: editor
layer: app / domain/editor
canonical: false
last_reviewed: 2026-05-26
related:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../reports/closeouts/pm-ui-designer-wb-001-governance-code-truth-and-track-activation/closeout.md
  - ../../../reports/closeouts/pm-ui-designer-wb-002-v1-package-document-session-and-evidence-model/closeout.md
---

# WR-122 Standalone App Shell And Embedded Host Parity Contract

## Goal

Implement `PM-UI-DESIGNER-WB-003` by adding a real standalone UI Designer
launch path and proving the embedded `Editor Design` workspace uses the same
app-neutral workbench contract.

This contract covers only host parity:

- standalone launch selects a UI Designer workbench, not the full editor or
  Material Lab;
- the embedded `Editor Design` workspace consumes the same package/session and
  evidence-facing view-model boundary;
- app state owns executable session state and window/runtime wiring;
- `domain/editor` owns editor workbench profiles, composition contracts, and
  app-neutral view-models;
- generic package, document, Canonical UI IR, token, recipe, and evidence
  truth remain outside app code.

It must not implement the V1 catalog, hierarchy/canvas/inspector authoring,
operation diff/apply/rollback, scenario matrix capture, performance baselines,
or game-runtime HUD behavior.

## Source Of Truth

- Production milestone: `PM-UI-DESIGNER-WB-003` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-122` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Governance closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-001-governance-code-truth-and-track-activation/closeout.md`.
- V1 model acceptance closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-002-v1-package-document-session-and-evidence-model/closeout.md`.
- Existing standalone pattern:
  `apps/runenwerk_editor/src/runtime/app.rs` module `app`,
  especially `RunenwerkRuntimeWorkbench`, `configure_app_for_workbench`,
  `build_headless_app_for_workbench`, and `run_material_lab_workbench`.
- Existing binary pattern:
  `apps/runenwerk_editor/src/bin/runenwerk_material_lab.rs`.
- Existing app host state:
  `apps/runenwerk_editor/src/editor_app/state.rs` module `state`,
  especially `EditorApp`, `EditorApp::material_lab_workbench`,
  `EditorApp::with_workbench_host`, and `EditorApp::workbench_host`.
- Existing app-owned workbench host:
  `apps/runenwerk_editor/src/shell/workbench_host.rs` module
  `workbench_host`, especially `RunenwerkWorkbenchComposition` and
  `RunenwerkWorkbenchHost`.
- Existing embedded/editor-domain contracts:
  `domain/editor/editor_shell/src/workspace/state.rs` module `state`,
  `domain/editor/editor_shell/src/workspace/profile.rs` module `profile`,
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`, and
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`.
- Existing app-facing UI Designer provider:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` function
  `ui_designer_workbench_view_model`.

## Readiness

`PM-UI-DESIGNER-WB-003` starts from `designing` because the accepted product
design was present, but no linked WR row or implementation contract existed.
This contract is the design-before-implementation action that makes the
milestone `ready_next`.

Architecture governance review was run for this scope on 2026-05-26. The
review confirms the expected owner split:

- `domain/ui` owns generic package/document, Canonical UI IR, tokens, recipes,
  target profiles, diagnostics, and evidence descriptors.
- `domain/editor` owns editor workbench profiles, app-neutral view-models, and
  composition contracts.
- `apps/runenwerk_editor` owns standalone binary/runtime selection, executable
  session state, provider mounting, native evidence capture, and platform
  impossible evidence.

No ADR is required for the host-parity slice because it uses existing app and
editor boundaries. Stop and require an ADR or accepted design update if the
implementation needs to move source truth into app code, introduce a new owner
crate, or change dependency direction.

After this planning action, expected next workflow is:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-003 --roadmap WR-122
```

The command must report the next promotion or implementation action before any
product code changes start.

## Promotion Readiness

After the ready-next intake row and this contract were applied,
`task production:plan -- --milestone PM-UI-DESIGNER-WB-003 --roadmap WR-122`
reported:

- production milestone state: `ready_next`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-004:support_only`, `WR-046:support_only`,
  `WR-114:completed`, and `WR-120:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-122 --state current_candidate --evidence "<accepted evidence>"
```

Accepted promotion evidence:

- accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`;
- completed PM001 governance closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-001-governance-code-truth-and-track-activation/closeout.md`;
- completed PM002 V1 model closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-002-v1-package-document-session-and-evidence-model/closeout.md`;
- completed `WR-120` governance row in the roadmap archive;
- this active WR-122 host-parity contract.

Promotion may proceed only while this evidence remains true and the production
goal still selects `PM-UI-DESIGNER-WB-003`.

## Implementation Scope

Allowed source scopes for the future coding slice:

- `apps/runenwerk_editor/src/runtime/app.rs` module `app`:
  add a `RunenwerkRuntimeWorkbench::UiDesigner` variant, title/resource
  selection, headless builder coverage, and a launch function matching the
  Material Lab pattern.
- `apps/runenwerk_editor/src/runtime/mod.rs` module `runtime`:
  re-export the UI Designer builder/runner only if needed by the standalone
  binary.
- `apps/runenwerk_editor/src/bin/runenwerk_ui_designer.rs` module
  `runenwerk_ui_designer`:
  add the standalone entry point.
- `apps/runenwerk_editor/src/editor_app/state.rs` module `state`:
  add app-owned construction for the UI Designer workbench host without making
  app state the canonical package or document owner.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` module
  `workbench_host`:
  add a UI Designer composition/profile if needed and keep provider registry,
  tool-surface registry, and profile registry invariants explicit.
- `apps/runenwerk_editor/src/shell/providers/mod.rs` module `providers`:
  add a narrow UI Designer provider registry constructor if the standalone host
  needs a dedicated provider set instead of the full editor registry.
- `domain/editor/editor_shell/src/workspace/profile.rs` module `profile` and
  `domain/editor/editor_shell/src/workspace/state.rs` module `state`:
  preserve embedded `Editor Design` profile/layout parity and add only
  app-neutral contracts needed by both hosts.
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition` and
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`:
  preserve the app-neutral UI Designer workbench view model and composition
  boundary.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` function
  `ui_designer_workbench_view_model`:
  keep provider output on the same app-neutral product contract used by the
  standalone host.

Explicit non-goals:

- no recipe catalog search/filter implementation;
- no hierarchy, canvas, inspector, diff, or review surface implementation;
- no Canonical UI IR operation authoring or textual patch pipeline;
- no source-versioned evidence capture beyond host-parity smoke evidence;
- no performance baseline or resize instrumentation;
- no game-runtime HUD implementation.

## Acceptance Criteria

- Standalone launch opens a UI Designer workbench title/window and not the full
  editor or Material Lab.
- Headless construction can prove the selected workbench composition without a
  native window.
- Embedded `Editor Design` keeps consuming the same app-neutral workbench
  view-model family as the standalone host.
- App code owns only runtime/session wiring; generic UI package/document truth
  stays outside `apps/runenwerk_editor`.
- Existing Material Lab and full editor launch behavior remains unchanged.
- Focused tests cover runtime workbench selection, app-state construction,
  workbench-host registry/profile invariants, and embedded/standalone contract
  parity.

## Validation

Future implementation validation must include at minimum:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-003 --roadmap WR-122
cargo test -p runenwerk_editor runtime
cargo test -p runenwerk_editor workbench_host
cargo test -p runenwerk_editor ui_designer
cargo test -p editor_shell ui_designer
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
```

Use narrower cargo filters only if the exact test names are known after code
changes. Run `./quiet_full_gate.sh` before runtime-proven closeout or broader
integration, not for this docs/planning-only readiness action.

## Stop Conditions

Stop before product code changes if:

- `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` no longer selects
  `PM-UI-DESIGNER-WB-003` or the linked `WR-122` action;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-003 --roadmap WR-122`
  reports a design, promotion, metadata, dependency, or current-candidate
  blocker;
- the implementation would make `apps/runenwerk_editor` own generic UI package,
  document, operation, token, recipe, or evidence truth;
- the standalone host cannot reuse the same app-neutral workbench contract as
  embedded `Editor Design`;
- a missing product requirement belongs to `PM-UI-DESIGNER-WB-004` or later;
- validation or generated roadmap/production checks fail.

## Closeout Requirements

The later implementation closeout must include:

- this contract path;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-003 --roadmap WR-122`
  output classification used before coding;
- focused test evidence for standalone runtime selection and embedded host
  parity;
- confirmation that full editor and Material Lab behavior did not regress;
- roadmap evidence update for `WR-122`;
- production milestone evidence update only when `PM-UI-DESIGNER-WB-003`
  acceptance criteria are actually met;
- rerun `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH`.

Expected completion quality for `WR-122` is `runtime_proven` if launched or
headless runtime evidence proves the real host path. If native launch evidence
is unavailable, close as `bounded_contract` with explicit known gaps.

## Perfectionist Closeout Audit

No perfectionist audit is intended for `WR-122`. The row must keep later gaps
visible:

- catalog, hierarchy, canvas, and inspector V1 remain in
  `PM-UI-DESIGNER-WB-004`;
- operation diff/apply/rollback remains in `PM-UI-DESIGNER-WB-005`;
- scenario evidence and performance baselines remain in
  `PM-UI-DESIGNER-WB-006`;
- game-runtime compatibility seam proof remains in `PM-UI-DESIGNER-WB-007`;
- runtime-proven final closeout and handoff remain in
  `PM-UI-DESIGNER-WB-008`.
