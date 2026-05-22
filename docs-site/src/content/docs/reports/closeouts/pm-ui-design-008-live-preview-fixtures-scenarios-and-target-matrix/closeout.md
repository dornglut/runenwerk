---
title: PM-UI-DESIGN-008 Live Preview Fixtures Scenarios And Target Matrix Closeout
description: Closeout evidence for the bounded WR-052 generic UI preview fixture, scenario, target matrix, and evidence descriptor contract implementation.
status: completed
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md
related_reports:
  - ../../implementation-plans/wr-052-pm-ui-design-008-preview-fixture-scenario-and-target-matrix-contracts/plan.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# PM-UI-DESIGN-008 Live Preview Fixtures Scenarios And Target Matrix Closeout

## Scope

`WR-052` completed the first bounded `PM-UI-DESIGN-008` implementation slice:
generic preview fixture, scenario, target matrix, and preview evidence
descriptor contracts in `domain/ui/ui_definition`.

The implementation remains runtime-neutral and domain-level. It does not add
app-hosted Preview Lab UI, screenshot capture, renderer golden comparison,
provider session orchestration, game-runtime replay loading, persistence
activation, accessibility/performance reporting, or production-readiness
hardening.

## Implementation Summary

- `domain/ui/ui_definition/src/preview_fixture/mod.rs` module
  `preview_fixture` defines stable fixture ids, scenario ids, matrix ids,
  evidence ids, data package ids, capability ids, target-profile ids, source
  package ids, diagnostic refs, data-state kinds, matrix axis kinds, validation
  modes, activation impact, scenario steps, declarations, libraries,
  validation requests, reports, and typed diagnostics.
- `domain/ui/ui_definition/src/preview_fixture/mod.rs` function
  `validate_preview_fixtures` validates fixture, scenario, matrix, target
  profile, capability, data package, data-state coverage, expected diagnostic,
  scenario step, matrix axis, and preview-only activation constraints.
- `domain/ui/ui_definition/src/preview_fixture/mod.rs` type
  `UiPreviewDiagnostic` records severity, code, fixture/scenario/matrix id,
  axis, source location, target profile, owning domain, source package,
  expected and actual diagnostics, denied capabilities, activation impact, and
  suggested fix.
- `domain/ui/ui_definition/src/preview_fixture/mod.rs` function
  `validate_preview_fixtures` rejects duplicate fixture and scenario ids,
  missing required data-state coverage, unsupported target profiles, missing
  data packages, unknown or denied capabilities, invalid scenario steps,
  duplicate or conflicting matrix axes, missing matrix fixture/scenario refs,
  expected diagnostic mismatches, and preview-only activation.
- `domain/ui/ui_definition/src/lib.rs` module exports make the new preview
  fixture contracts discoverable through the crate public surface.

## Validation

Focused validation before closeout metadata updates:

- `cargo test -p ui_definition preview_fixture` passed with 9 preview-fixture
  tests.
- `cargo test -p ui_definition` passed with 44 unit tests and doc-tests.

Workflow validation completed before closeout metadata updates:

- `task ai:closeout -- --task "WR-052 PM-UI-DESIGN-008 Preview Fixture Scenario And Target Matrix Contracts" --scope "domain/ui/ui_definition; docs-site/src/content/docs/reports/implementation-plans/wr-052-pm-ui-design-008-preview-fixture-scenario-and-target-matrix-contracts/plan.md; docs-site/src/content/docs/workspace/roadmap-items.yaml; docs-site/src/content/docs/workspace/production-tracks.yaml" --roadmap docs-site/src/content/docs/workspace/roadmap-items.yaml` produced the required phase-completion drift-check prompt.

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

- Editor/workbench and game-runtime examples without shared runtime, provider,
  screenshot, or renderer ownership are covered by
  `preview_fixture_validates_editor_and_runtime_examples_without_runtime_ownership`.
- Required fixture data-state coverage is covered by
  `preview_fixture_rejects_missing_data_state_coverage`.
- Missing data package diagnostics are covered by
  `preview_fixture_rejects_missing_data_package`.
- Denied capability diagnostics are covered by
  `preview_fixture_rejects_denied_capability`.
- Target-profile diagnostics are covered by
  `preview_fixture_rejects_unsupported_target_profile`.
- Scenario step diagnostics are covered by
  `preview_fixture_rejects_invalid_scenario_steps`.
- Matrix axis diagnostics are covered by
  `preview_fixture_rejects_matrix_axis_conflicts`.
- Expected diagnostic mismatch diagnostics are covered by
  `preview_fixture_rejects_expected_diagnostic_mismatches`.
- Preview-only activation rejection is covered by
  `preview_fixture_rejects_preview_only_activation`.

## Completion Quality

Completion quality is `bounded_contract`.

Known quality gaps:

- App-hosted Preview Lab UI is not implemented in this slice.
- Screenshot capture, visual diffing, and renderer golden comparison are not
  implemented in this slice.
- Provider session orchestration and game-runtime replay loading remain future
  work.
- Persistence migration, diff, and activation gates remain
  `PM-UI-DESIGN-009` work.
- Accessibility/performance reporting, visual captures, production examples,
  and production-readiness evidence remain `PM-UI-DESIGN-010` work.
- This slice proves generic preview fixture, scenario, matrix, and evidence
  descriptor declaration contracts and diagnostics, not a user-visible
  Interface Lab Preview UI or runtime-proven replay bridge.
- PM-008 did not claim `runtime_proven` or `perfectionist_verified` evidence.

## Drift Check

The phase completion drift check found the implementation aligned with the
accepted PM-008 design and `WR-052` contract:

- Generic preview fixture, scenario, matrix, and evidence descriptor truth stays
  in `domain/ui/ui_definition`.
- Target-profile compatibility, denied capabilities, missing data packages,
  invalid scenario steps, expected diagnostic mismatches, and preview-only
  activation fail closed with typed diagnostics.
- Editor/workbench and game-runtime examples share generic contracts without
  moving runtime, provider, renderer, screenshot, app window, project, or
  gameplay state into generic UI definitions.
- Persistence activation and production-readiness evidence remain later
  milestones and must not be inferred complete from this bounded preview
  contract slice.
