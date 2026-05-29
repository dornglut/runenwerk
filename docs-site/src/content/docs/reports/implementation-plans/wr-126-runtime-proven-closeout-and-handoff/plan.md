---
title: WR-126 Runtime Proven Closeout And Handoff Contract
description: Ready-next implementation contract for PM-UI-DESIGNER-WB-008 final runtime-proven UI Designer Workbench closeout, usage docs, examples, and downstream handoff.
status: active
owner: editor
layer: docs / workspace
canonical: false
last_reviewed: 2026-05-26
related:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../reports/closeouts/pm-ui-designer-wb-001-governance-code-truth-and-track-activation/closeout.md
  - ../../../reports/closeouts/pm-ui-designer-wb-002-v1-package-document-session-and-evidence-model/closeout.md
  - ../../../reports/closeouts/pm-ui-designer-wb-003-standalone-app-shell-and-embedded-host-parity/closeout.md
  - ../../../reports/closeouts/pm-ui-designer-wb-004-catalog-hierarchy-canvas-inspector-v1/closeout.md
  - ../../../reports/closeouts/pm-ui-designer-wb-005-operation-diff-apply-and-rollback/closeout.md
  - ../../../reports/closeouts/pm-ui-designer-wb-006-scenario-evidence-and-performance-baselines/closeout.md
  - ../../../reports/closeouts/pm-ui-designer-wb-007-game-runtime-compatibility-seam/closeout.md
---

# WR-126 Runtime Proven Closeout And Handoff Contract

## Goal

Complete `PM-UI-DESIGNER-WB-008` by closing the UI Designer Workbench
Productization track at `runtime_proven` quality. The slice must link the
completed PM001-PM007 evidence, add practical usage docs and examples, update
production/roadmap metadata, and hand off downstream game-runtime UI and
perfectionist no-gap work without claiming that those separate tracks are
complete.

This contract is a final closeout and documentation slice only. It must not
change product runtime code, implement game HUD runtime behavior, add
game-runtime UI projection execution, claim native screenshot evidence, or make
a perfectionist no-gap claim.

## Source Of Truth

- Production milestone: `PM-UI-DESIGNER-WB-008` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-126` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Active downstream game-runtime UI boundary design:
  `docs-site/src/content/docs/design/active/game-runtime-ui-projection-and-hud-platform-design.md`.
- Completed UI Designer Workbench closeouts:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-001-governance-code-truth-and-track-activation/closeout.md`
  through
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-007-game-runtime-compatibility-seam/closeout.md`.
- Archived game-runtime seam evidence:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-008-game-ui-readiness-seam/closeout.md`
  and `WR-118` in
  `docs-site/src/content/docs/workspace/roadmap-archive.yaml`.

## Readiness

`PM-UI-DESIGNER-WB-008` starts from `designing` after
`PM-UI-DESIGNER-WB-007` completed. The accepted UI Designer Workbench product
design covers the product boundary, and PM001-PM007 provide completed
evidence gates for governance, package/session model, host parity, product
surfaces, operation/apply/rollback, scenario evidence/performance baselines,
and game-runtime compatibility seam proof.

Architecture governance kickoff was run for PM008 on 2026-05-26. The bounded
owner split remains:

- `domain/ui` owns generic UI definition, target-profile, preview, readiness,
  binding, recipe, persistence, and visual-layout truth.
- `domain/editor` owns UI Designer app-neutral workbench contracts and UX Lab
  evidence adapters.
- `apps/runenwerk_editor` owns concrete standalone and embedded host wiring,
  session state, provider projection, evidence capture, and runtime proof
  artifacts or typed unsupported reasons.
- `docs-site/src/content/docs/workspace` and `docs-site/src/content/docs/reports`
  own final planning, closeout, generated registers, and handoff metadata.

No ADR is required while WR-126 only records final evidence, usage docs,
examples, and handoff notes. Require an ADR or accepted design update before
moving source-truth ownership, adding a game-runtime UI owner crate, changing
dependency direction, or claiming no-gap quality.

After this planning action, expected next workflow is:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-008 --roadmap WR-126
```

The command must report the next promotion or implementation action before any
final closeout edits start.

## Promotion Readiness

After the ready-next intake row and this contract were applied,
`task production:plan -- --milestone PM-UI-DESIGNER-WB-008 --roadmap WR-126`
reported:

- production milestone state: `ready_next`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-120:completed`, `WR-122:completed`,
  `WR-123:completed`, `WR-124:completed`, `WR-125:completed`, and
  `WR-118:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-126 --state current_candidate --evidence "<accepted evidence>"
```

Accepted promotion evidence:

- accepted UI Designer Workbench product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`;
- completed PM001-PM007 UI Designer Workbench closeout evidence:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-001-governance-code-truth-and-track-activation/closeout.md`
  through
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-007-game-runtime-compatibility-seam/closeout.md`;
- completed archived game-runtime seam evidence:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-008-game-ui-readiness-seam/closeout.md`;
- this active WR-126 runtime-proven closeout and handoff contract.

Promotion may proceed only while this evidence remains true and the production
goal still selects `PM-UI-DESIGNER-WB-008`.

## Implementation Scope

Allowed future source scopes:

- `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-008-runtime-proven-closeout-and-handoff/closeout.md`
  for the final production-track closeout.
- `docs-site/src/content/docs/apps/runenwerk-editor` for practical UI Designer
  Workbench usage docs and examples.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`,
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`, and
  `docs-site/src/content/docs/workspace/roadmap-archive.yaml` for bounded
  production and roadmap state transitions.
- Generated production and roadmap registers/diagrams produced by
  `task production:render` and `task roadmap:render`.

Out of scope:

- product runtime code changes;
- new app, domain, engine, renderer, or adapter code;
- game HUD runtime behavior, SDF HUD rendering, or game-runtime UI projection
  execution;
- native screenshot or packaged release claims not already supported by
  completed evidence;
- perfectionist no-gap quality or local-native no-gap certification.

## Acceptance Criteria

WR-126 is acceptable only when:

- the final closeout links completed PM001-PM007 runtime/design evidence;
- usage docs describe normal standalone and embedded UI Designer workflows,
  including catalog, hierarchy, canvas, inspector, operations, apply/reject,
  rollback, scenario evidence, performance baselines, diagnostics, and
  game-runtime seam boundaries;
- examples show the preferred public workflow through existing launch and
  evidence commands without relying on internal shortcuts;
- downstream handoff notes explicitly route game HUD runtime behavior to
  `PT-GAME-RUNTIME-UI` and any no-gap claim to a separate audit/certification
  track;
- known gaps are truthful and do not contradict runtime-proven V1 workbench
  completion;
- roadmap, production, docs, planning, PUML, and git diff hygiene gates pass.

## Validation

Required validation for the future implementation slice:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-008 --roadmap WR-126
cargo test -p runenwerk_editor ui_designer
cargo test -p runenwerk_editor editor_lab_evidence
cargo test -p ui_definition game -- --nocapture
cargo test -p editor_shell game -- --nocapture
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

- `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` no longer selects PM008
  as the next legal action;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-008 --roadmap WR-126`
  does not authorize promotion or implementation;
- any PM001-PM007 closeout evidence is missing or fails its focused
  validation;
- final docs would need product runtime code changes;
- the closeout would need to claim game HUD runtime behavior, native runtime
  screenshot evidence, packaged release readiness, or perfectionist no-gap
  quality.

## Closeout Requirement

The final closeout must include:

- all completed milestone evidence gates and linked WR rows;
- exact docs and metadata files changed by WR-126;
- representative focused validation output;
- generated roadmap and production validation results;
- downstream handoff notes for `PT-GAME-RUNTIME-UI` and no-gap/perfectionist
  quality;
- final known gaps that do not block `runtime_proven` UI Designer Workbench
  completion.
