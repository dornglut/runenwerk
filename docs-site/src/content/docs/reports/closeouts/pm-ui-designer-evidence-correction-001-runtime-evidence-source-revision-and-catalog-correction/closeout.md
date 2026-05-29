---
title: PM-UI-DESIGNER-EVIDENCE-CORRECTION-001 Runtime Evidence Source Revision And Catalog Correction
description: Corrective closeout for UI Designer Workbench runtime evidence truth, source revision freshness, catalog routing, and game-runtime descriptor boundaries.
status: completed
owner: editor
layer: app / editor
canonical: true
last_reviewed: 2026-05-29
related_designs:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
related_reports:
  - ../pm-ui-designer-wb-v1-closure-006-runtime-proven-product-closeout-and-handoff/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# PM-UI-DESIGNER-EVIDENCE-CORRECTION-001 Runtime Evidence Source Revision And Catalog Correction

## Summary

`PM-UI-DESIGNER-EVIDENCE-CORRECTION-001` supersedes the earlier UI Designer
Workbench runtime-evidence claim shape without rewriting historical closeouts.
The correction makes descriptor compatibility and runtime product evidence
separate contracts, records source revisions with content hashes and session
epochs, invalidates evidence on source mutations, and prevents `game.runtime`
descriptor compatibility from passing editor-workbench runtime readiness.

## Implementation Evidence

- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs` splits
  descriptor compatibility packets from runtime product packets and requires
  product-path artifact provenance, `blake3` digests, nonzero bytes, and
  product-path measurement provenance for runtime readiness.
- `apps/runenwerk_editor/src/shell/self_authoring/mod.rs` records
  `EditorLabSourceRevision` values, bumps the session epoch on source-affecting
  mutations, clears stale evidence centrally, and captures editor-workbench
  runtime evidence separately from game-runtime descriptor compatibility.
- `apps/runenwerk_editor/src/shell/editor_lab_evidence/game_runtime.rs` owns
  game-runtime descriptor fixtures and validated intent descriptors in the app
  evidence layer.
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` and
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  expose structured, filterable catalog rows with disabled reasons and routes
  only for compatible insertions.
- `domain/editor/editor_shell/src/ux_lab/design_system.rs` keeps the editor
  recipe library editor-owned and no longer exports game HUD safe-area recipes.

## Supersession

The completed `PT-UI-DESIGNER-WORKBENCH`,
`PT-UI-DESIGNER-WB-V1-CLOSURE`, and archived `WR-124` through `WR-132` records
remain historical evidence. Current readiness claims must use this corrected
evidence standard. In particular, old synthetic status summaries,
descriptor-only packets, `memory://` placeholders, and string-length byte
counts are not sufficient runtime proof.

## Validation

- `cargo test -p runenwerk_editor editor_lab_evidence --quiet`
- `cargo test -p runenwerk_editor ui_designer --quiet`
- `cargo test -p runenwerk_editor source_revision --quiet`
- `cargo test -p editor_shell ui_designer --quiet`

Full closeout validation also requires docs, roadmap, production, PUML,
`git diff --check`, and `./quiet_full_gate.sh`.

## Known Gaps

- Concrete game HUD runtime behavior remains owned by `PT-GAME-RUNTIME-UI`.
- Native runtime-window screenshots, packaged release readiness, and
  perfectionist no-gap certification remain out of scope for this correction.
