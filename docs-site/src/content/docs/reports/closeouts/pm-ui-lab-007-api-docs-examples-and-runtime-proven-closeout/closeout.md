---
title: PM-UI-LAB-007 API Docs Examples And Runtime-Proven Closeout
description: Final runtime-proven closeout for Editor Lab V1 public APIs, usage docs, examples, evidence aggregation, and perfectionist-audit handoff.
status: completed
owner: editor
layer: domain/app/docs
canonical: true
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/accepted/ui-lab-api-docs-examples-runtime-closeout-design.md
  - ../../../design/active/ui-lab-productization-design.md
related_reports:
  - ../../implementation-plans/wr-088-ui-lab-api-docs-examples-and-runtime-proven-closeout/plan.md
  - ./api-ergonomics-review.md
  - ../pm-ui-lab-001-productization-governance/closeout.md
  - ../pm-ui-lab-002-registry-and-command-source-of-truth/closeout.md
  - ../pm-ui-lab-003-app-hosted-editor-lab-surface-shell/closeout.md
  - ../pm-ui-lab-004-operation-driven-visual-authoring/closeout.md
  - ../pm-ui-lab-005-persistence-project-io-diff-apply-and-rollback/closeout.md
  - ../pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../roadmap-intake/2026-05-24-pt-ui-lab-perfection-no-gap-audit/proposal.yaml
---

# PM-UI-LAB-007 API Docs Examples And Runtime-Proven Closeout

## Scope

`WR-088` completes the bounded `PM-UI-LAB-007` release slice: normal
`ui_definition` and `editor_definition` workflows now have focused public
entry points, usage docs, compile-backed examples, an API ergonomics review,
and a final PT-UI-LAB closeout that aggregates PM001-PM006 runtime evidence.

This closeout does not claim `perfectionist_verified`. Native screenshot/GPU
visual-diff platform work, deeper accessibility and performance platform
coverage, game-runtime UI projection execution, module-structure perfection
review, and zero-known-gap certification remain explicit later audit scope.

## Implementation Summary

- `domain/ui/ui_definition/src/prelude.rs` module `prelude` exposes the normal
  behavior-free UI definition imports.
- `domain/ui/ui_definition/src/workflow.rs` module `workflow` exposes
  `validate_ui_template`, `normalize_ui_template`,
  `apply_ui_layout_operation`, `preview_ui_layout_operation`,
  `validate_ui_preview_library`, `validate_ui_persistence_flow`, and
  `validate_ui_readiness`.
- `domain/ui/ui_definition/examples/ui_definition_workflow.rs` compiles and
  runs through `ui_definition::prelude::*` to validate/normalize an authored UI
  template, apply a generic visual layout operation, and validate preview,
  persistence, and readiness descriptor libraries.
- `domain/editor/editor_definition/src/prelude.rs` module `prelude` exposes
  the normal runtime-neutral editor definition imports.
- `domain/editor/editor_definition/src/workflow.rs` module `workflow` exposes
  `new_editor_definition_document`, `validate_editor_document`,
  `editor_document_has_blocking_diagnostics`, `apply_editor_lab_edit`, and
  `form_editor_theme_tokens`.
- `domain/editor/editor_definition/examples/editor_definition_workflow.rs`
  compiles and runs through `editor_definition::prelude::*` to create a
  document, validate it, apply an `EditorLabOperation`, and inspect the
  accepted operation report and deterministic diff.
- `docs-site/src/content/docs/domain/ui/ui-definition-usage.md` and
  `docs-site/src/content/docs/domain/editor/editor-definition-usage.md` teach
  normal public workflows before architecture-only details.
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/api-ergonomics-review.md`
  records the required public API ergonomics review.

## Runtime Evidence Aggregation

PT-UI-LAB is runtime-proven through the completed milestone chain:

- PM-UI-LAB-001:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-001-productization-governance/closeout.md`
  records governance, architecture findings, code-truth reconciliation, and WR
  candidate scopes.
- PM-UI-LAB-002:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-002-registry-and-command-source-of-truth/closeout.md`
  records command catalog and surface registry runtime proof.
- PM-UI-LAB-003:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-003-app-hosted-editor-lab-surface-shell/closeout.md`
  records app-hosted Editor Lab surface runtime proof and retained visual
  artifacts.
- PM-UI-LAB-004:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md`
  records typed operation, diff, diagnostics, history, undo, and redo runtime
  proof.
- PM-UI-LAB-005:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-005-persistence-project-io-diff-apply-and-rollback/closeout.md`
  records project IO, apply review, activation reports, failed activation
  preservation, reload, and rollback runtime proof.
- PM-UI-LAB-006:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md`
  records Preview Lab scenario evidence, retained visual artifacts,
  diagnostics snapshots, accessibility snapshots, performance snapshots,
  unsupported-check diagnostics, and degraded-provider proof.
- PM-UI-LAB-007:
  this closeout records public API ergonomics, usage docs, examples, and the
  final runtime-proven evidence aggregation.

## Acceptance Evidence

- Normal UI definition users can start from `ui_definition::prelude::*` and
  use focused workflow helpers instead of discovering the API through broad
  crate-level glob exports.
- Normal editor definition users can start from
  `editor_definition::prelude::*` and use focused workflow helpers for
  document creation, validation, operation application, diagnostics, diffs, and
  theme formation.
- Public examples compile and run:

```text
cargo run -p ui_definition --example ui_definition_workflow
cargo run -p editor_definition --example editor_definition_workflow
```

- Usage docs link from the owning domain index pages and teach app handoff
  boundaries explicitly.
- The public API ergonomics review is completed and links the code, docs,
  examples, and remaining gaps.
- `ui_definition` remains behavior-free.
- `editor_definition` remains runtime-neutral.
- Runtime evidence execution, project IO, activation, rollback, provider
  sessions, and artifact writing remain app-owned.

## Validation

Focused validation for PM007:

```text
cargo fmt
cargo test -p ui_definition
cargo test -p editor_definition
cargo run -p ui_definition --example ui_definition_workflow
cargo run -p editor_definition --example editor_definition_workflow
```

Final track validation:

```text
cargo test -p runenwerk_editor editor_lab
cargo test -p runenwerk_editor pm_ui_lab_006
task docs:validate
task puml:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
git diff --check
task ai:goal -- --track PT-UI-LAB --scope non-deferred
```

## Completion Quality

Completion quality is `runtime_proven`.

Known quality gaps:

- Native window screenshots, GPU visual diffing, native focus traversal, pixel
  contrast sampling, native screenshot timing, and GPU visual-diff timing
  remain explicit unsupported-check diagnostics from PM006.
- Broad crate-level glob exports remain for compatibility; a later no-gap
  audit may choose deeper API reshaping.
- Game-runtime UI projection execution remains out of Editor Lab V1 scope.
- Module-structure perfectionist review, deeper UI ergonomics audit, native
  screenshot and visual diff platform decisions, accessibility and performance
  depth beyond retained evidence, and zero-known-gap certification remain
  separate audit scope.
- `perfectionist_verified` is intentionally not claimed.

## Perfectionist Audit Handoff

PM007 creates a separate roadmap intake for the no-gap audit:

```text
docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-pt-ui-lab-perfection-no-gap-audit/proposal.yaml
```

That later audit owns no-gap certification and must not be inferred from this
runtime-proven closeout.

## Closeout Decision

Close PM-UI-LAB-007 and PT-UI-LAB as runtime-proven after WR088 is archived,
production metadata is updated, final validation passes, and
`task ai:goal -- --track PT-UI-LAB --scope non-deferred` reports no remaining
non-deferred incomplete milestones.
