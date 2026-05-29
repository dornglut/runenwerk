---
title: WR-125 Scenario Evidence And Performance Baselines Contract
description: Ready-next implementation contract for PM-UI-DESIGNER-WB-006 source-versioned scenario evidence, diagnostics snapshots, explicit capture controls, and UI Designer performance baselines.
status: active
owner: editor
layer: domain/ui-definition / domain/editor / app
canonical: false
last_reviewed: 2026-05-26
related:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../../../design/accepted/ui-designer-production-readiness-and-evidence-design.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../reports/closeouts/pm-ui-designer-wb-005-operation-diff-apply-and-rollback/closeout.md
---

# WR-125 Scenario Evidence And Performance Baselines Contract

## Goal

Implement `PM-UI-DESIGNER-WB-006` by adding source-versioned UI Designer
scenario evidence packets, diagnostics snapshots, explicit capture controls,
and measured resize, canvas, catalog, diagnostics, and frame-build baselines.

This contract covers evidence and baseline hardening only:

- evidence packets include package id, document id, source version, target
  profile, scenario id, diagnostics, artifact references or typed unsupported
  reasons, performance counters, and freshness status;
- scenario and target-matrix descriptors reuse the generic
  `domain/ui/ui_definition` preview fixture and production readiness contracts;
- editor/workbench scenario vocabulary stays in `domain/editor`;
- app code owns concrete capture orchestration, timings, snapshots, and
  artifact records;
- evidence capture is explicit and must not run accidentally every frame.

It must not implement game-runtime HUD behavior, final usage docs/examples, or
a track-level handoff closeout. Those remain owned by
`PM-UI-DESIGNER-WB-007` and `PM-UI-DESIGNER-WB-008`.

## Source Of Truth

- Production milestone: `PM-UI-DESIGNER-WB-006` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-125` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Accepted preview fixture, scenario, target matrix, and evidence design:
  `docs-site/src/content/docs/design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md`.
- Accepted production readiness and evidence design:
  `docs-site/src/content/docs/design/accepted/ui-designer-production-readiness-and-evidence-design.md`.
- Completed operation/apply/rollback closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-005-operation-diff-apply-and-rollback/closeout.md`.
- Generic preview fixtures and scenario matrices:
  `domain/ui/ui_definition/src/preview_fixture` module
  `preview_fixture`.
- Generic readiness packets and freshness decisions:
  `domain/ui/ui_definition/src/production_readiness` module
  `production_readiness`.
- Editor/workbench scenario contracts:
  `domain/editor/editor_shell/src/ux_lab` module `ux_lab`.
- App evidence artifact records:
  `apps/runenwerk_editor/src/shell/editor_lab_evidence` module
  `editor_lab_evidence`.
- UI Designer provider projection:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`.

## Readiness

`PM-UI-DESIGNER-WB-006` starts from `designing` after
`PM-UI-DESIGNER-WB-005` completed. Existing accepted UI Designer designs cover
generic preview fixture, scenario, target matrix, production readiness,
diagnostic, artifact freshness, and performance report doctrine, but this
production milestone had no linked WR row or decision-complete implementation
contract.

Architecture governance kickoff was run for this scope on 2026-05-26. The
bounded owner split remains:

- `domain/ui/ui_definition` owns generic preview fixture, scenario, target
  matrix, evidence descriptor, production readiness, diagnostic, freshness, and
  readiness decision contracts.
- `domain/editor/editor_shell` owns editor/workbench scenario vocabulary,
  app-neutral surface evidence view models, and retained composition contracts.
- `apps/runenwerk_editor` owns concrete UI Designer capture orchestration,
  source-versioned evidence runs, diagnostics snapshots, performance sampling,
  artifact paths, and unsupported-platform reports.

No ADR is required while WR-125 preserves these ownership and dependency
boundaries. Require an ADR or accepted design update before moving screenshot
bytes, renderer handles, provider sessions, app windows, project IO, or
performance runner implementation into `domain/ui`.

After this planning action, expected next workflow is:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-006 --roadmap WR-125
```

The command must report the next promotion or implementation action before any
product code changes start.

## Promotion Readiness

After the ready-next intake row and this contract were applied,
`task production:plan -- --milestone PM-UI-DESIGNER-WB-006 --roadmap WR-125`
reported:

- production milestone state: `ready_next`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-004:support_only`, `WR-046:support_only`,
  `WR-052:completed`, `WR-054:completed`, `WR-120:completed`,
  `WR-122:completed`, `WR-123:completed`, and `WR-124:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-125 --state current_candidate --evidence "<accepted evidence>"
```

Accepted promotion evidence:

- accepted UI Designer product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`;
- accepted preview fixture, scenario, target matrix, and evidence design:
  `docs-site/src/content/docs/design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md`;
- accepted production readiness and evidence design:
  `docs-site/src/content/docs/design/accepted/ui-designer-production-readiness-and-evidence-design.md`;
- completed PM005 operation/apply/rollback closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-005-operation-diff-apply-and-rollback/closeout.md`;
- this active WR-125 scenario evidence and performance baseline contract.

Promotion may proceed only while this evidence remains true and the production
goal still selects `PM-UI-DESIGNER-WB-006`.

## Implementation Scope

Allowed future source scopes:

- `domain/ui/ui_definition/src/preview_fixture` for missing generic fixture,
  scenario, target matrix, and evidence descriptor validation needed by PM006.
- `domain/ui/ui_definition/src/production_readiness` for missing readiness
  packet, freshness, evidence kind, and diagnostic validation needed by PM006.
- `domain/editor/editor_shell/src/ux_lab` for editor/workbench scenario matrix
  contracts and app-neutral surface evidence view models.
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` and
  `domain/editor/editor_shell/src/composition` for app-neutral evidence and
  performance projection view models.
- `apps/runenwerk_editor/src/shell/editor_lab_evidence` for app-owned evidence
  runs, artifact records, unsupported checks, and performance counter records.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`,
  `apps/runenwerk_editor/src/shell/providers/tests.rs`,
  `apps/runenwerk_editor/src/shell/self_authoring.rs`, and
  `apps/runenwerk_editor/src/shell/tests.rs` for UI Designer provider evidence,
  explicit capture commands, and focused tests.

Out of scope:

- game-runtime HUD behavior or SDF screen HUD proof;
- final usage docs, examples, and track handoff closeout;
- moving generic evidence truth into app state;
- automatic every-frame capture;
- broad screenshot/golden-image tooling beyond typed unsupported reasons and
  retained artifact records required for PM006.

## Acceptance Criteria

The WR-125 implementation is acceptable only when:

- evidence packets include package id, document id, source version, target
  profile, scenario id, diagnostics, artifact references or typed unsupported
  reasons, performance counters, and freshness status;
- resize, canvas interaction, catalog projection, diagnostics projection, and
  frame-build baselines are captured as explicit records;
- capture execution is opt-in and testable;
- unsupported native screenshot, GPU visual diff, accessibility, or timing
  checks produce typed evidence instead of silent success;
- UI Designer surfaces expose PM006 readiness without claiming PM007
  game-runtime seam proof or PM008 final handoff completion.

## Validation

Required validation for the future implementation slice:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-006 --roadmap WR-125
cargo fmt --package ui_definition --package editor_shell --package runenwerk_editor
cargo test -p ui_definition preview_fixture
cargo test -p ui_definition production_readiness
cargo test -p editor_shell ux_lab
cargo test -p runenwerk_editor editor_lab_evidence
cargo test -p runenwerk_editor ui_designer
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

## Stop Conditions

Stop before implementation if:

- `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` no longer selects PM006
  as the next legal action;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-006 --roadmap WR-125`
  does not authorize promotion or implementation;
- any required design or PM005 closeout evidence is missing;
- implementation would require concrete screenshots, renderer handles,
  provider sessions, app windows, project IO, or performance runner truth in
  `domain/ui`;
- a required behavior belongs to PM007 or PM008.

## Closeout Requirement

The implementation closeout must include:

- exact changed file paths and modules;
- the scenario matrix and target profiles exercised;
- the evidence packet schema and source-version fields proven;
- the performance counters and baseline values captured;
- explicit unsupported checks, if any;
- focused validation output;
- truthful known gaps for PM007 and PM008.
