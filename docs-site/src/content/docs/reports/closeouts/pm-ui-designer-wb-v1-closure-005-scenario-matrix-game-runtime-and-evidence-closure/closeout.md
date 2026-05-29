---
title: PM-UI-DESIGNER-WB-V1-CLOSURE-005 Scenario Matrix Game Runtime And Evidence Closure
description: Runtime-proven closeout evidence for WR-131 scenario matrix, game.runtime compatibility descriptors, source-versioned evidence packets, and measured product-path performance baselines.
status: completed
owner: editor
layer: domain/ui / domain/editor / app
canonical: false
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../../../design/accepted/ui-designer-production-readiness-and-evidence-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
  - ../../../design/accepted/ui-designer-workbench-product-design.md
related_reports:
  - ../../implementation-plans/wr-131-ui-designer-workbench-v1-closure-scenario-matrix-game-runtime-evidence-and-performance-baselines/plan.md
  - ../pm-ui-designer-wb-v1-closure-004-operation-diff-apply-rollback-parity/closeout.md
  - ../../roadmap-intake/2026-05-26-ui-designer-workbench-v1-closure-scenari/proposal.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# PM-UI-DESIGNER-WB-V1-CLOSURE-005 Scenario Matrix Game Runtime And Evidence Closure

## Summary

`PM-UI-DESIGNER-WB-V1-CLOSURE-005` / `WR-131` closes the bounded scenario,
game.runtime compatibility, evidence packet, and performance baseline slice for
the UI Designer Workbench V1 closure track.

The workbench now captures source-versioned scenario evidence for both
`editor.workbench` and `game.runtime` target profiles through an explicit
product command. Evidence packets record package, document, source version,
target profile, diagnostics, artifact or unsupported-check references,
freshness, read-only fixture/binding descriptors, validated intent descriptors,
and measured frame/canvas/catalog/diagnostics/resize baselines.

The game.runtime proof remains descriptor-only. It records fixture, binding,
intent, compatibility, and unsupported-check evidence without executing
game-runtime commands or implementing concrete HUD runtime behavior.

This slice does not claim final V1 product closeout, packaged release
readiness, perfectionist no-gap certification, or concrete game HUD behavior.

## Implementation Evidence

- `domain/ui/ui_definition/src/production_readiness/mod.rs` module
  `production_readiness` already owns generic readiness packets, target
  profiles, compatibility axes, freshness, external artifact ownership, and
  fail-closed game.runtime axis validation.
- `domain/ui/ui_definition/src/preview_fixture/mod.rs` module
  `preview_fixture` already owns generic fixture, scenario, matrix, evidence,
  target-profile, data-state, capability, expected-diagnostic, and
  game.runtime compatibility-axis validation.
- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs` module
  `editor_lab_evidence` now models source-versioned scenario evidence packets
  with `EditorLabReadOnlyFixtureBindingDescriptor`,
  `EditorLabValidatedIntentDescriptor`, and
  `EditorLabScenarioEvidencePacket::validate_scenario_evidence`.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` method
  `SelfAuthoringWorkspaceState::capture_pm005_scenario_evidence_packets`
  captures explicit `editor.workbench` and `game.runtime` packets, measures
  baselines through selected workbench product paths, and clears stale packets
  on project reload.
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition` adds the app-neutral
  `EditorDefinitionSurfaceAction::CaptureScenarioEvidence` route action.
- `domain/editor/editor_shell/src/commands/shell_command.rs` module
  `shell_command` adds `ShellCommand::CaptureUiDesignerScenarioEvidence`.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `providers::self_authoring` projects PM-005 readiness rows, packet summaries,
  descriptor summaries, baseline summaries, and a capture action without making
  provider projection source truth.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` function
  `dispatch_shell_command_with_viewport_commands` routes the explicit capture
  command to app-owned self-authoring evidence capture.
- `apps/runenwerk_editor/src/shell/providers/tests.rs` test
  `ui_designer_workbench_exposes_pm005_scenario_evidence_and_performance_baselines`
  proves both target profiles, fresh source-version matching, descriptor
  validation, unsupported game-runtime boundary evidence, and provider
  projection.
- `apps/runenwerk_editor/src/shell/tests.rs` test
  `dispatch_shell_command_captures_ui_designer_scenario_evidence` proves the
  evidence capture is reachable through shell command dispatch.

## Validation Results

Focused validation run on 2026-05-26:

```text
cargo test -p ui_definition production_readiness
cargo test -p ui_definition preview_fixture
cargo test -p editor_shell ui_designer
cargo test -p runenwerk_editor ui_designer
cargo test -p runenwerk_editor self_authoring
cargo test -p runenwerk_editor scenario_evidence
```

Results:

- `ui_definition production_readiness`: 10 matching tests passed.
- `ui_definition preview_fixture`: 13 matching tests passed.
- `editor_shell ui_designer`: 2 matching tests passed.
- `runenwerk_editor ui_designer`: 11 matching tests passed.
- `runenwerk_editor self_authoring`: 11 matching unit tests plus 2 viewport
  architecture guard tests passed.
- `runenwerk_editor scenario_evidence`: 5 matching tests passed.

Planning validation for this closeout is:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-005 --roadmap WR-131
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

`./quiet_full_gate.sh` is intentionally not part of this bounded closeout
because PM-005 is proven by focused domain, editor-shell, app workflow, and
evidence-packet tests plus roadmap/production validation.

## Completion Quality

Completion quality is `runtime_proven`.

Known quality gaps remain by design:

- `PM-UI-DESIGNER-WB-V1-CLOSURE-006` still owns final runtime-proven product
  closeout, honest known-gap classification, and downstream handoff.
- Concrete game HUD runtime behavior remains downstream of
  `PT-GAME-RUNTIME-UI`.
- Native window screenshot artifacts remain typed unsupported evidence in
  headless validation; retained workbench evidence is sufficient for this
  bounded PM-005 product-path proof but not for no-gap certification.
- Packaged release readiness and perfectionist no-gap certification are not
  claimed by this slice.

## Closeout Decision

Close `PM-UI-DESIGNER-WB-V1-CLOSURE-005`, archive `WR-131` as completed
runtime-proven scenario/evidence/performance baseline evidence, and rerun
`task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` before selecting the
next legal closure action.
