---
title: Batch 2026-05-15-next-current-candidate-roadmap-batch-wr-
description: Parallel roadmap batch closeout report.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-15
---

# Batch 2026-05-15-next-current-candidate-roadmap-batch-wr-

Goal: Next current-candidate roadmap batch: WR-025, WR-018, WR-007
Approval state: approved
Integration status: merged
Closeout status: completed
Integrated into: main@e7beb2d35523f769d94a79ee5c6f0ae65f4d8a79

## Validation Results

- 2026-05-15T16:32:48+00:00 batch validate passed: host batch validation; task roadmap:render, task roadmap:validate, task roadmap:check, task docs:validate, cargo check -p runenwerk_editor, cargo test -p runenwerk_editor viewport, cargo test -p ecs, cargo test -p scheduler
- 2026-05-15T16:38:00+00:00 post-integration validation passed: cargo fmt --all -- --check; uv run pytest tools/workflow/test_workflow.py -q; task roadmap:validate; task roadmap:check; task docs:validate; cargo test -p engine_net -p engine_sim; cargo check -p runenwerk_editor; cargo test -p runenwerk_editor viewport; cargo test -p ecs -p scheduler; task puml:validate

## Roadmap Evidence Updates

- 2026-05-15 WR-025 established the Interaction V2 migration spine and guardrail contract while keeping retained UI first, WR-024 downstream, and alternate UI execution targets deferred behind a separate ADR/design.
- 2026-05-15 WR-018 aligned rendered-world V1 scene and GPU-picking packet decode for the editor SDF primitive set and added shader-slot contract tests.
- 2026-05-15 WR-007 hardened net ACK/baseline validation and delta lifecycle normalization in engine_net; the follow-up WR-007 bridge convergence closeout completed Phase 3 after this batch and is recorded in reports/closeouts/wr-007-multiplayer-replication-phase-1-3/closeout.md.

## Tooling Hardening

- Scope enforcement now ignores status-only modified entries when git diff reports no content change, preventing CRLF/stat-only generated-doc noise from failing a batch.
- Scope enforcement now permits roadmap render outputs declared by roadmap-items.yaml when a worker owns the roadmap source.
- Default batch ids now preserve WR item suffixes after slug truncation so future current-candidate batches do not collide in one report directory.

## Items

### WR-025 UI Runtime V2 and interaction formation

- Branch: `codex/2026-05-15-next-current-candidate-roadmap-batch-wr--wr-025`
- Worktree: `cleaned after integration`
- Status: `integrated`
- Roadmap outcome: `slice_landed_item_still_current`
- Write scopes: `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`, `docs-site/src/content/docs/design/active/editor-shell-menu-and-tab-chrome-polish-design.md`, `docs-site/src/content/docs/design/active/editor-ui-popup-adornment-drop-preview-contract.md`, `docs-site/src/content/docs/design/deferred/ui-model-multiple-execution-strategies-design.md`, `docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md`, `docs-site/src/content/docs/domain/ui/README.md`, `docs-site/src/content/docs/domain/ui/architecture.md`, `docs-site/src/content/docs/domain/ui/roadmap.md`, `docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md`, `docs-site/src/content/docs/workspace/roadmap-items.yaml`

### WR-018 Rendered World V1

- Branch: `codex/2026-05-15-next-current-candidate-roadmap-batch-wr--wr-018`
- Worktree: `cleaned after integration`
- Status: `integrated`
- Roadmap outcome: `slice_landed_item_still_current`
- Write scopes: `apps/runenwerk_editor/src/runtime`, `assets/shaders`, `docs-site/src/content/docs/design/active/editor-rendered-world-and-multi-entity-viewport-design.md`

### WR-007 Multiplayer replication Phase 1 to Phase 3

- Branch: `codex/2026-05-15-next-current-candidate-roadmap-batch-wr--wr-007`
- Worktree: `cleaned after integration`
- Status: `roadmap_closed`
- Roadmap outcome: `roadmap_completed`
- Write scopes: `net`, `domain/ecs`
