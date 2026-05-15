---
title: Batch 2026-05-15-next-current-candidate-roadmap-batch-wr-025-wr-018
description: Parallel roadmap batch closeout report.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-15
---

# Batch 2026-05-15-next-current-candidate-roadmap-batch-wr-025-wr-018

Goal: Next current-candidate roadmap batch: WR-025, WR-018
Approval state: approved
Integration status: merged
Closeout status: completed
Integrated into: main@b1c525bbe24bc50646b100bdb0bcc6b5abf679dd

## Validation Results

- 2026-05-15T18:03:00+00:00 batch validate passed: host batch validation; task roadmap:render, task roadmap:validate, task roadmap:check, task docs:validate, cargo check -p runenwerk_editor, cargo test -p runenwerk_editor viewport

## Roadmap Evidence Updates

- WR-025 documented the named Interaction V2 retained UI migration slice catalog and linked WR-024 shell polish to those contracts before implementation.
- WR-018 routed CPU entity picking through the viewport render-state scene packet and documented fail-closed behavior when no packet exists.

## Tooling Hardening

- Fixed batch finalization path matching to compare Git-cleaned blob hashes, preventing CRLF-normalized integrated worker files from being reported as unmerged.

## Items

### WR-025 UI Runtime V2 and interaction formation

- Branch: `codex/2026-05-15-next-current-candidate-roadmap-batch-wr-025-wr-018-wr-025`
- Worktree: `cleaned after integration`
- Status: `integrated`
- Roadmap outcome: `slice_landed_item_still_current`
- Write scopes: `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`, `docs-site/src/content/docs/design/active/editor-shell-menu-and-tab-chrome-polish-design.md`, `docs-site/src/content/docs/design/active/editor-ui-popup-adornment-drop-preview-contract.md`, `docs-site/src/content/docs/design/deferred/ui-model-multiple-execution-strategies-design.md`, `docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md`, `docs-site/src/content/docs/domain/ui/README.md`, `docs-site/src/content/docs/domain/ui/architecture.md`, `docs-site/src/content/docs/domain/ui/roadmap.md`, `docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md`, `docs-site/src/content/docs/workspace/roadmap-items.yaml`

### WR-018 Rendered World V1

- Branch: `codex/2026-05-15-next-current-candidate-roadmap-batch-wr-025-wr-018-wr-018`
- Worktree: `cleaned after integration`
- Status: `integrated`
- Roadmap outcome: `slice_landed_item_still_current`
- Write scopes: `apps/runenwerk_editor/src/runtime`, `assets/shaders`, `docs-site/src/content/docs/design/active/editor-rendered-world-and-multi-entity-viewport-design.md`
