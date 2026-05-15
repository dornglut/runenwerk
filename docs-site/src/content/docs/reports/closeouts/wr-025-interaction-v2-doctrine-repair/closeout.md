---
title: WR-025 Interaction V2 Doctrine Repair Closeout
description: Completion and drift-check record for the WR-025 invalid dock/drop target and viewport/status behavior repair.
status: completed
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-15
related_designs:
  - ../../../design/active/editor-ui-runtime-v2-and-interaction-formation-design.md
related_adrs:
  - ../../../adr/accepted/0009-ui-interaction-formation-v2.md
related_roadmaps:
  - ../../../domain/ui/roadmap.md
  - ../../../apps/runenwerk-editor/roadmap.md
  - ../../../workspace/roadmap-index.md
related_reports:
  - ../../batches/2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025-4/batch.md
---

# WR-025 Interaction V2 Doctrine Repair Closeout

## Status

Complete as of 2026-05-15.

This repair closes the WR-025 doctrine gap where evidence said invalid dock/drop targets and viewport/status arbitration were complete before behavior-level proof existed. It does not start WR-024 shell polish, WR-018, menu sizing expansion, chrome-slot expansion, dock/drop-zone expansion, compiled-reactive UI, ECS-driven UI, or any alternate UI execution target.

## Owning Scope

- Architecture: `docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md`.
- Active design: `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`.
- UI contracts/runtime: `domain/ui/ui_tree`, `domain/ui/ui_runtime`, and existing `domain/ui/ui_definition` Interaction V2 records.
- Editor shell: `domain/editor/editor_shell/src/composition/build_editor_shell.rs`, `domain/editor/editor_shell/src/composition/build_viewport_panel.rs`, and `apps/runenwerk_editor/src/shell`.
- App bridge: `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs`.

## Completion Evidence

- `domain/editor/editor_shell/src/composition/build_editor_shell.rs` now uses `DockDropCandidateState` instead of a boolean active flag. Invalid editor-owned dock/drop rules are explicit through `DockDropInvalidTargetReason`.
- `apps/runenwerk_editor/src/shell/controller.rs` forms source-only same-area and same-host split candidates as invalid, keeps them visible in preview evidence, and excludes them from commit target resolution.
- `apps/runenwerk_editor/src/shell/state.rs` cycles only selectable dock/drop candidates and preserves invalid candidates as evidence.
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::dock_drop_zone_interaction_model` maps editor-owned invalid candidates to the generic `UiDockDropZoneStateDefinition::Invalid` contract without moving workspace rules into `domain/ui`.
- `domain/ui/ui_tree/src/tree/node.rs` adds explicit popup dismiss policy, and `domain/editor/editor_shell/src/composition/build_viewport_panel.rs` marks persistent viewport chrome/status overlays as non-dismissible anchored popups. Menu popups remain dismissible stack members.
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs` now has behavior-level coverage proving viewport status dispatch targets cannot become scene fallback routes, while existing wheel ownership guards still prove viewport zoom happens only after UI declines wheel ownership.

## Drift Findings

- The active Interaction V2 design claimed invalid dock/drop target and viewport/status completeness with evidence that was too broad. Corrected the design to cite the repaired candidate state, controller/state behavior tests, popup dismiss policy, and viewport status dispatch guard.
- UI/editor roadmap evidence marked WR-025 complete before the doctrine repair existed. Corrected the roadmap source and rendered roadmaps to make completion depend on this closeout.
- Finalized WR-025 batch manifests had completed/merged items without `worktree_cleanup`. Corrected the finalized manifests and marked the final WR-025 outcome as `roadmap_completed`.
- A stale mixed WR-025/WR-018 continuation proposal was removed before regenerating the WR-018-only continuation proposal from the repaired head.

## Validation

Repair validation completed:

- `cargo test -p editor_shell dock`
- `cargo test -p runenwerk_editor shell`
- `cargo test -p runenwerk_editor input_bridge`
- `cargo test -p ui_runtime popup`
- `cargo check -p runenwerk_editor`

Workflow validation completed from this repaired state:

- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task docs:validate`
- `task batch:scope-check` for the touched WR-025 batch manifests
- `task repo:hygiene` failed only because the WR-025 repair working tree was intentionally dirty before commit.
- `./quiet_full_gate.sh` reached clippy and failed only on the known separate Draw hygiene blocker: `apps/runenwerk_draw/src/runtime/systems.rs::submit_draw_frame_system` has 10 arguments where clippy allows 7.

## Deferred Work

- WR-024 remains downstream retained shell polish and may only consume the repaired WR-025 contracts.
- WR-018 remains the next roadmap proposal after this repair is committed and the continuation batch is regenerated from the repaired head.
- The known Draw clippy full-gate blocker remains a separate hygiene slice; this closeout does not hide it with an allow.

Post-closeout update, 2026-05-16: the Draw clippy gate blocker was resolved in the separate gate-hygiene slice by making grouped ECS system params first-class enough for Draw frame submission to avoid a local lint allow.
