---
title: PM-UI-DESIGNER-WB-006 Scenario Evidence And Performance Baselines Closeout
description: Runtime-proven closeout for WR-125 source-versioned UI Designer scenario evidence, diagnostics snapshots, explicit capture controls, and performance baseline projection.
status: completed
owner: editor
layer: domain/ui-definition / domain/editor / app
canonical: false
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../../../design/accepted/ui-designer-production-readiness-and-evidence-design.md
related_reports:
  - ../../implementation-plans/wr-125-scenario-evidence-and-performance-baselines/plan.md
  - ../pm-ui-designer-wb-005-operation-diff-apply-and-rollback/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# PM-UI-DESIGNER-WB-006 Scenario Evidence And Performance Baselines Closeout

## Summary

`PM-UI-DESIGNER-WB-006` / `WR-125` completed the bounded UI Designer
scenario-evidence and performance-baseline slice. The implementation adds an
app-owned explicit PM006 evidence packet for the selected UI Designer document,
including package id, document id, source version, target profile, scenario id,
diagnostics snapshot, retained artifact reference, typed unsupported native
screenshot reason, freshness check, and resize/canvas/catalog/diagnostics/
frame-build baseline records.

The slice preserves the accepted ownership split. `domain/ui` remains the owner
of generic preview and readiness contracts, `domain/editor` remains the owner
of editor/workbench scenario vocabulary and app-neutral surface contracts, and
`apps/runenwerk_editor` owns concrete capture orchestration, timings, retained
artifacts, unsupported-platform reasons, and provider projection.

This closeout does not claim game-runtime compatibility seam proof, game HUD
runtime behavior, final usage docs/examples, or track-level handoff completion.

## Implementation Evidence

Code changes for this PM006 implementation action are limited to the WR-125
scenario evidence and baseline write scope:

- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs` module
  `editor_lab_evidence`: added `EditorLabScenarioEvidencePacket`,
  PM006 freshness/capture-mode enums, performance baseline kinds, baseline
  validation, and focused tests for explicit capture, source-version identity,
  artifact or unsupported reason coverage, and required baseline kinds.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`: added `selected_source_version_label`,
  `capture_pm006_evidence_packet`, `last_pm006_evidence_packet`, and
  app-owned measured baseline construction. Capture is explicit and stored as
  session evidence; project reload clears stale PM006 evidence.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`: projects PM006 readiness rows, source-version freshness,
  evidence packet metadata, diagnostics counts, retained artifact references,
  typed unsupported native-screenshot reasons, and performance baseline rows
  through the real UI Designer Workbench surface path.
- `apps/runenwerk_editor/src/shell/providers/tests.rs` module
  `providers::tests`: proves explicit PM006 capture and provider projection on
  the real `ToolSurfaceKind::UiCanvas` UI Designer Workbench route.

Existing support contracts remain in their owning domains:

- `domain/ui/ui_definition/src/preview_fixture` module `preview_fixture`:
  accepted generic fixture, scenario, target matrix, target-profile, and
  evidence descriptor validation.
- `domain/ui/ui_definition/src/production_readiness` module
  `production_readiness`: accepted evidence packet, freshness, artifact
  ownership, compatibility, inspection, and readiness decision validation.
- `domain/editor/editor_shell/src/ux_lab` module `ux_lab`: editor/workbench
  scenario catalog and readiness vocabulary.

## Gate Classification

`task production:plan -- --milestone PM-UI-DESIGNER-WB-006 --roadmap WR-125`
passed with `PM-UI-DESIGNER-WB-006` in `ready_next`, `WR-125` promoted to
`current_candidate`, and dependencies `WR-004:support_only`,
`WR-046:support_only`, `WR-052:completed`, `WR-054:completed`,
`WR-120:completed`, `WR-122:completed`, `WR-123:completed`, and
`WR-124:completed`.

`task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` selected
`PM-UI-DESIGNER-WB-006` with next legal action
`execute_next_wr_implementation_contract` before this implementation slice.

## Validation Results

Focused validation run on 2026-05-26:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-006 --roadmap WR-125 passed.
cargo fmt --package ui_definition --package editor_shell --package runenwerk_editor passed.
cargo test -p ui_definition preview_fixture passed.
cargo test -p ui_definition production_readiness passed.
cargo test -p editor_shell ux_lab passed.
cargo test -p runenwerk_editor editor_lab_evidence passed.
cargo test -p runenwerk_editor ui_designer passed.
task docs:validate passed.
task roadmap:render passed.
task roadmap:validate passed.
task roadmap:check passed.
task production:render passed.
task production:validate passed.
task production:check passed.
task planning:validate passed.
task puml:validate passed.
git diff --check passed.
```

`./quiet_full_gate.sh` is intentionally not part of this closeout because this
is a focused scenario evidence and performance baseline product slice. Later
milestones still own game-runtime seam proof and final handoff quality.

## Completion Quality

Completion quality is `runtime_proven`.

The runtime proof is headless provider and app-state evidence. Tests explicitly
capture a PM006 packet from `SelfAuthoringWorkspaceState`, validate the packet,
then resolve the real UI Designer Workbench canvas through
`EditorSurfaceProviderRegistry`, `SelfAuthoringProvider`, editor-shell view
models, retained `build_editor_lab_surface` composition, and typed
editor-definition routes. The resulting surface exposes PM006 readiness rows,
source-versioned evidence metadata, explicit capture mode, freshness status,
diagnostics snapshot counts, retained artifact references, typed unsupported
native-screenshot reasons, and all five required baseline families.

Known quality gaps remain by design:

- `PM-UI-DESIGNER-WB-007` still owns game-runtime compatibility seam proof.
- `PM-UI-DESIGNER-WB-008` still owns final runtime-proven track closeout,
  usage docs, examples, and handoff notes.
- PM006 records a typed unsupported native-screenshot reason in headless
  validation; it does not claim native-window screenshot artifacts.
- PM006 proves app/headless evidence capture and provider projection; it does
  not claim packaged product release readiness.

## Drift Check

The closeout satisfies PM006 source-versioned evidence packet, diagnostics
snapshot, freshness, explicit capture, artifact or unsupported reason, and
resize/canvas/catalog/diagnostics/frame-build baseline acceptance criteria. It
does not claim PM007 game-runtime seam proof or PM008 final handoff behavior.
