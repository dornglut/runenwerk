---
title: PM-UI-DESIGN-009 Persistence Migration Diff And Activation Closeout
description: Closeout evidence for the bounded WR-053 generic UI definition persistence, migration dry-run, deterministic diff, and activation gate contract implementation.
status: completed
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
related_reports:
  - ../../implementation-plans/wr-053-pm-ui-design-009-persistence-migration-diff-and-activation-contracts/plan.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# PM-UI-DESIGN-009 Persistence Migration Diff And Activation Closeout

## Scope

`WR-053` completed the first bounded `PM-UI-DESIGN-009` implementation slice:
generic UI definition persistence, migration dry-run, deterministic diff, and
activation gate contracts in `domain/ui/ui_definition`.

The implementation remains runtime-neutral and domain-level. It does not add
app-hosted persistence UI, project save/load UI, diff review UI, concrete
project file IO, provider session orchestration, runtime activation plumbing,
renderer resources, screenshot capture, gameplay mutation,
accessibility/performance reporting, or production-readiness hardening.

## Implementation Summary

- `domain/ui/ui_definition/src/persistence_activation/mod.rs` module
  `persistence_activation` defines stable document ids, migration report ids,
  diff ids, activation request ids, target-profile ids, source package ids,
  diagnostic refs, field paths, validation modes, unknown-field preservation
  policies, diff change kinds, descriptors, libraries, validation requests,
  reports, and typed diagnostics.
- `domain/ui/ui_definition/src/persistence_activation/mod.rs` function
  `validate_persistence_activation` validates persistence documents, migration
  reports, deterministic diffs, and activation requests against a target
  profile, unknown-field policy, expected diagnostics, schema versions, and
  activation mode.
- `domain/ui/ui_definition/src/persistence_activation/mod.rs` type
  `UiPersistenceDiagnostic` records severity, code, document/report/diff/request
  ids, field path, source location, target profile, source/target schema
  versions, owning domain, source package, expected and actual diagnostics,
  activation impact, and suggested fix.
- `domain/ui/ui_definition/src/persistence_activation/mod.rs` function
  `validate_persistence_activation` rejects duplicate ids, unsupported schema
  versions, unknown required fields, unpreservable unknown fields, unknown-field
  policy violations, incompatible migrations, schema mismatches, unsupported
  target schema versions, missing preserved unknown fields, dropped unknown
  fields under preserve-compatible policy, non-deterministic diff output,
  missing or unknown migration reports, missing or unknown diff descriptors,
  document mismatches, target-profile incompatibility, expected diagnostic
  mismatches, and preview-only activation.
- `domain/ui/ui_definition/src/lib.rs` module exports make the new persistence
  activation contracts discoverable through the crate public surface.

## Validation

Focused validation before closeout metadata updates:

- `cargo test -p ui_definition persistence_activation` passed with 10
  persistence-activation tests.
- `cargo test -p ui_definition` passed with 54 unit tests and doc-tests.

Workflow validation completed before closeout metadata updates:

- `task ai:closeout -- --task "WR-053 PM-UI-DESIGN-009 Persistence Migration Diff And Activation Contracts" --scope "domain/ui/ui_definition; docs-site/src/content/docs/reports/implementation-plans/wr-053-pm-ui-design-009-persistence-migration-diff-and-activation-contracts/plan.md; docs-site/src/content/docs/workspace/roadmap-items.yaml; docs-site/src/content/docs/workspace/production-tracks.yaml" --roadmap docs-site/src/content/docs/workspace/roadmap-items.yaml` produced the required phase-completion drift-check prompt.

Workflow validation completed after this closeout was linked from roadmap
archive and production-track evidence:

- `task roadmap:render` passed.
- `task roadmap:validate` passed.
- `task roadmap:check` passed.
- `task production:render` passed.
- `task production:validate` passed.
- `task production:check` passed.
- `task docs:validate` passed.
- `task planning:validate` passed.

`task ai:goal -- --track PT-UI-DESIGN` must be rerun after this closeout update
and before the next milestone starts.

## Acceptance Evidence

- Editor/workbench and game-runtime examples without shared project IO,
  provider, runtime, renderer, screenshot, or gameplay ownership are covered by
  `persistence_activation_validates_editor_and_runtime_examples_without_shared_ownership`.
- Unsupported schema version diagnostics are covered by
  `persistence_activation_rejects_unsupported_schema_version`.
- Incompatible migration diagnostics are covered by
  `persistence_activation_rejects_incompatible_migration`.
- Compatible unknown-field preservation is covered by
  `persistence_activation_preserves_compatible_unknown_fields`.
- Unpreservable unknown-field diagnostics are covered by
  `persistence_activation_rejects_unpreservable_unknown_fields`.
- Non-deterministic diff diagnostics are covered by
  `persistence_activation_rejects_non_deterministic_diff`.
- Missing migration report and diff descriptor diagnostics are covered by
  `persistence_activation_requires_migration_report_and_diff`.
- Target-profile diagnostics are covered by
  `persistence_activation_rejects_unsupported_target_profile`.
- Expected diagnostic mismatch diagnostics are covered by
  `persistence_activation_rejects_expected_diagnostic_mismatches`.
- Preview-only activation rejection is covered by
  `persistence_activation_rejects_preview_only_activation`.

## Completion Quality

Completion quality is `bounded_contract`.

Known quality gaps:

- App-hosted persistence UI, project save/load UI, and user-facing diff review
  UI are not implemented in this slice.
- Concrete project file IO in `apps/runenwerk_editor` is not implemented in
  this slice.
- Runtime activation plumbing and runtime consumption of activation reports
  remain future work.
- Provider session orchestration, screenshot capture, visual regression, and
  renderer golden evidence remain future work.
- Accessibility/performance reporting, production examples, and
  production-readiness evidence remain `PM-UI-DESIGN-010` work.
- This slice proves generic persistence, migration dry-run, deterministic diff,
  activation request, activation decision, and diagnostic contracts, not a
  user-visible Interface Lab persistence workflow or runtime-proven activation
  bridge.
- PM-009 did not claim `runtime_proven` or `perfectionist_verified` evidence.

## Drift Check

The phase completion drift check found the implementation aligned with the
accepted PM-009 design and `WR-053` contract:

- Generic persistence, migration dry-run, deterministic diff, activation, and
  diagnostic truth stays in `domain/ui/ui_definition`.
- Schema/version gates, incompatible migrations, unknown-field preservation,
  non-deterministic diffs, missing migration reports, missing diff descriptors,
  target-profile incompatibility, expected diagnostic mismatches, and
  preview-only activation fail closed with typed diagnostics.
- Editor/workbench and game-runtime examples share generic contracts without
  moving project IO, provider sessions, runtime state, renderer handles,
  screenshots, app windows, gameplay state, concrete editor commands, or direct
  mutation into generic UI definitions.
- Production-readiness evidence remains `PM-UI-DESIGN-010` and must not be
  inferred complete from this bounded persistence activation contract slice.
