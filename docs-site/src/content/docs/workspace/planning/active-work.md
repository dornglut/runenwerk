---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../reports/closeouts/phase-16-surface2d-closeout.md
  - ../../reports/investigations/phase-16-surface2d-source-investigation.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

No active implementation focus is selected after the Phase 16 Surface2D closeout.

ID: none

Title: none

State: Phase 16 completed; next work is planning intake selection, not implementation.

Lifecycle state: none

Owner: workspace planning owns the next intake selection. No product crate owns active implementation work from this record.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, `docs-site/src/content/docs/workspace/planning/completed-work.md`, `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`, `docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md`, and `docs-site/src/content/docs/reports/closeouts/phase-16-surface2d-closeout.md`.

Evidence classes: `E3` local source/history inspection, `E5` post-merge local validation evidence from `main`, `E8` accepted workspace/planning/design authority, and `E9` combined closeout alignment across source, validation, and planning truth.

Complete investigation gate: no active implementation focus. Phase 16 investigation is completed as historical evidence. The next non-trivial phase must pass the complete investigation gate before implementation authorization.

Complete design gate: no active implementation focus. Phase 16 design is completed as historical evidence. The next non-trivial phase must pass the complete design gate, including principle compliance and module decomposition evidence, before implementation authorization.

Implementation contract: none authorized.

Allowed files/crates: docs and planning files required for closeout truth recording only.

Non-owned files/crates: all product code, all Phase 17 implementation files, renderer backends, editor/game/product command owners, graph/timeline implementation owners, new crates, plugin framework work, and `foundation/meta`.

Principle compliance matrix: not active for implementation because no implementation focus is selected. Phase 16 closeout records that KISS, DRY, YAGNI, SOLID, Separation of Concerns, Avoid Premature Optimization, and Law of Demeter passed for the delivered Surface2D scope. Any next phase must record its own principle matrix before implementation.

Module decomposition map: not active for implementation because no implementation focus is selected. Phase 16 closeout records the delivered Surface2D decomposition. Any compound next phase must record a module decomposition map before implementation.

Maintainability review status: Phase 16 completed. See `docs-site/src/content/docs/reports/closeouts/phase-16-surface2d-closeout.md`.

Feature support matrix: Phase 16 completed; see `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`.

Future-use-case pressure matrix: Phase 16 completed; see `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`.

Hierarchy/composition matrix: Phase 16 completed; see `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`.

Ergonomics/usability: Phase 16 completed; see `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`.

Validation expectation: closeout docs must pass `python tools/docs/validate_docs.py` and `git diff --check`. No cargo validation is expected because this closeout must not touch product code.

Known blockers: no Phase 16 product blocker remains. The remote branch `surface2d-phase-16` is intentionally kept for manual review because it contains three non-equivalent commits with a large Surface2D design-document diff that mixes potentially useful future-use-case pressure with stale pre-merge implementation assumptions. That branch is not a Phase 16 product blocker and must not be mixed into closeout.

Next action: perform the next planning intake from the production track. The next named future milestone is `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas, but no Phase 17 implementation is authorized by this closeout.

Evidence: PR #62 merged docs-only principle/decomposition/merge-readiness workflow hardening at merge commit `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`. PR #61 squash-merged Phase 16 Surface2D at merge commit `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. Post-merge validation from `main` passed with `cargo test -p ui_controls surface2d`, `cargo test -p ui_controls control_package`, `cargo test -p ui_runtime surface2d`, `cargo test -p ui_static_mount surface2d`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`. Branch inspection showed `surface2d-phase-16` has three non-equivalent commits and is kept for manual review.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, stop conditions, principle compliance status, and module decomposition status are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Evidence classes:
Complete investigation gate:
Complete design gate:
Implementation contract:
Allowed files/crates:
Non-owned files/crates:
Principle compliance matrix:
Module decomposition map:
Maintainability review status:
Feature support matrix:
Future-use-case pressure matrix:
Hierarchy/composition matrix:
Ergonomics/usability:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
