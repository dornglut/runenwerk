---
title: PM-UI-DESIGN-006 Component Surface And Widget Recipe Library Closeout
description: Closeout evidence for the bounded WR-050 generic UI component, widget, and surface recipe contract implementation.
status: completed
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
  - ../../../design/accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md
related_reports:
  - ../../implementation-plans/wr-050-pm-ui-design-006-component-surface-and-widget-recipe-contracts/plan.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# PM-UI-DESIGN-006 Component Surface And Widget Recipe Library Closeout

## Scope

`WR-050` completed the first bounded `PM-UI-DESIGN-006` implementation slice:
generic UI component, widget, and surface recipe contracts in
`domain/ui/ui_definition`.

The implementation remains runtime-neutral and domain-level. It does not add
app-hosted recipe browsers, editor-specific package persistence, game-runtime
package loading, renderer/material lowering, view-model binding, live preview
fixture matrices, persistence activation, or production-readiness hardening.

## Implementation Summary

- `domain/ui/ui_definition/src/component_recipe/mod.rs` module
  `component_recipe` defines stable recipe ids, source package ids, target
  profile ids, state variant ids, slot ids, recipe kinds, activation modes,
  accessibility metadata, layout behavior, focus/navigation behavior, token
  family requirements, declarations, libraries, child references, expansion
  requests, reports, and typed diagnostics.
- `domain/ui/ui_definition/src/component_recipe/mod.rs` function
  `expand_ui_recipe` expands recipe declarations into Canonical UI IR
  `UiNodeDefinition` trees deterministically while preserving stable authored
  ids and slot-scoped child composition.
- `domain/ui/ui_definition/src/component_recipe/mod.rs` type
  `UiRecipeDiagnostic` records severity, code, recipe id, slot path, source
  location, target profile, owning domain, source package, source provenance,
  activation impact, and suggested fix.
- `domain/ui/ui_definition/src/component_recipe/mod.rs` function
  `expand_ui_recipe` rejects duplicate recipe ids, unknown recipe references,
  recipe cycles, unsupported target profiles, invalid slot child kinds, unknown
  slots, missing required slots, invalid slot mount nodes, missing required
  token-family references, missing accessibility metadata, invalid layout
  constraints, invalid focus navigation, and preview-only activation.
- `domain/ui/ui_theme/src/token/mod.rs` module `token` now exposes serialized
  token id and token-family contracts so generic recipe declarations can safely
  persist token-family requirements without duplicating theme token ownership.
- `domain/ui/ui_definition/src/lib.rs` and `domain/ui/ui_theme/src/lib.rs`
  module exports make the new recipe and token contracts discoverable from
  their owning crate public surfaces.

## Validation

Focused validation before closeout metadata updates:

- `cargo test -p ui_definition component_recipe` passed with 8 component recipe
  tests.
- `cargo test -p ui_definition` passed with 26 unit tests and doc-tests.
- `cargo test -p ui_theme` passed with 15 unit tests and doc-tests.

Workflow validation completed before closeout metadata updates:

- `task ai:closeout -- --task "WR-050 PM-UI-DESIGN-006 Component Surface And Widget Recipe Contracts" --scope "domain/ui/ui_definition; domain/ui/ui_theme; docs-site/src/content/docs/reports/implementation-plans/wr-050-pm-ui-design-006-component-surface-and-widget-recipe-contracts/plan.md; docs-site/src/content/docs/workspace/roadmap-items.yaml; docs-site/src/content/docs/workspace/production-tracks.yaml" --roadmap docs-site/src/content/docs/workspace/roadmap-items.yaml` produced the required phase-completion drift-check prompt.

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

- Stable recipe id preservation and deterministic expansion are covered by
  `component_recipe_expands_deterministically_and_preserves_ids` in
  `domain/ui/ui_definition/src/component_recipe/mod.rs`.
- Slot child-kind compatibility diagnostics are covered by
  `component_recipe_rejects_invalid_slot_child_kind`.
- Required token-family diagnostics are covered by
  `component_recipe_rejects_missing_required_token_family`.
- Accessibility metadata diagnostics are covered by
  `component_recipe_rejects_missing_accessibility_metadata`.
- Target-profile compatibility diagnostics are covered by
  `component_recipe_rejects_unsupported_target_profile`.
- Preview-only activation rejection is covered by
  `component_recipe_rejects_preview_only_activation`.
- Duplicate recipe id diagnostics and source provenance are covered by
  `component_recipe_rejects_duplicate_recipe_ids`.
- Editor/workbench and game-runtime target-profile examples are covered by
  `component_recipe_supports_editor_and_runtime_target_profiles`.

## Completion Quality

Completion quality is `bounded_contract`.

Known quality gaps:

- App-hosted component, widget, and surface recipe browsing or editing remain
  future work.
- Editor-specific recipe package persistence and project file IO remain future
  work.
- Game-runtime recipe package loading is not implemented or runtime-proven in
  this slice.
- Renderer material lowering, shader styles, GPU policy, and visual target
  projection execution remain future work.
- View-model capability binding, live preview fixture matrices, persistence
  activation, accessibility/performance evidence, and production-readiness
  hardening remain governed by PM-UI-DESIGN-007 through PM-UI-DESIGN-010.
- This slice proves generic recipe contracts and diagnostics, not a
  user-visible Interface Lab recipe editor or runtime-proven recipe loading.
- PM-006 did not claim `runtime_proven` or `perfectionist_verified` evidence.

## Drift Check

The phase completion drift check found the implementation aligned with the
accepted PM-006 design and `WR-050` contract:

- Generic recipe truth stays in `domain/ui/ui_definition`.
- Token ids and token families stay owned by `domain/ui/ui_theme`.
- App, runtime, editor shell, provider, renderer, material, gameplay, project,
  and session state did not become recipe source truth.
- Preview-only recipes cannot activate.
- Unsupported target-profile declarations fail closed with typed diagnostics.
- The remaining milestones stay incomplete and must not be inferred complete
  from this bounded recipe-contract slice.
