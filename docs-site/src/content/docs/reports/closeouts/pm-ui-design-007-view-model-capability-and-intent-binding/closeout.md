---
title: PM-UI-DESIGN-007 View-Model Capability And Intent Binding Closeout
description: Closeout evidence for the bounded WR-051 generic UI view-model binding, capability, and intent proposal contract implementation.
status: completed
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
related_reports:
  - ../../implementation-plans/wr-051-pm-ui-design-007-view-model-capability-and-intent-binding-contracts/plan.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# PM-UI-DESIGN-007 View-Model Capability And Intent Binding Closeout

## Scope

`WR-051` completed the first bounded `PM-UI-DESIGN-007` implementation slice:
generic read-only UI view-model binding, capability requirement, and intent
proposal contracts in `domain/ui/ui_definition`.

The implementation remains runtime-neutral and domain-level. It does not add
app-hosted binding or intent Designer/Lab UI, editor-specific package
persistence, game-runtime package loading, concrete command execution, concrete
game intent execution, live preview fixture matrices, persistence activation,
or production-readiness hardening.

## Implementation Summary

- `domain/ui/ui_definition/src/view_binding/mod.rs` module `view_binding`
  defines stable binding ids, intent ids, view-model package ids, field ids,
  capability ids, target-profile ids, value types, validation modes, package
  freshness status, missing-data policy, intent descriptor references, trigger
  sources, payload binding refs, libraries, validation requests, reports, and
  typed diagnostics.
- `domain/ui/ui_definition/src/view_binding/mod.rs` function
  `validate_ui_bindings` validates binding and intent declarations against a
  target profile, package status, capability policy, known formatters, known
  descriptors, and declared package fields.
- `domain/ui/ui_definition/src/view_binding/mod.rs` type
  `UiBindingDiagnostic` records severity, code, binding or intent id, node,
  trigger, source location, target profile, owning domain, source package,
  required and denied capabilities, activation impact, and suggested fix.
- `domain/ui/ui_definition/src/view_binding/mod.rs` function
  `validate_ui_bindings` rejects duplicate package/binding/intent ids, unknown
  packages, missing packages, stale packages where not allowed, unsupported
  target profiles, unknown fields, value type mismatches, unknown or denied
  capabilities, unknown formatters, unknown descriptors, direct mutation
  intents, payload binding failures, trigger conflicts, and preview-only
  activation.
- `domain/ui/ui_definition/src/lib.rs` module exports make the new binding and
  intent contracts discoverable through the crate public surface.

## Validation

Focused validation before closeout metadata updates:

- `cargo test -p ui_definition view_binding` passed with 9 view-binding tests.
- `cargo test -p ui_definition` passed with 35 unit tests and doc-tests.

Workflow validation completed before closeout metadata updates:

- `task ai:closeout -- --task "WR-051 PM-UI-DESIGN-007 View-Model Capability And Intent Binding Contracts" --scope "domain/ui/ui_definition; docs-site/src/content/docs/reports/implementation-plans/wr-051-pm-ui-design-007-view-model-capability-and-intent-binding-contracts/plan.md; docs-site/src/content/docs/workspace/roadmap-items.yaml; docs-site/src/content/docs/workspace/production-tracks.yaml" --roadmap docs-site/src/content/docs/workspace/roadmap-items.yaml` produced the required phase-completion drift-check prompt.

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

- Editor/workbench and game-runtime examples without shared semantic authority
  are covered by
  `view_binding_validates_editor_and_runtime_examples_without_shared_authority`
  in `domain/ui/ui_definition/src/view_binding/mod.rs`.
- Binding value type diagnostics are covered by
  `view_binding_rejects_value_type_mismatch`.
- Missing view-model package diagnostics are covered by
  `view_binding_rejects_missing_view_model_package`.
- Denied capability diagnostics are covered by
  `view_binding_rejects_denied_capability`.
- Target-profile diagnostics are covered by
  `view_binding_rejects_unsupported_target_profile`.
- Proposal-only intent enforcement is covered by
  `view_binding_intent_emits_proposals_not_direct_mutation`.
- Intent payload value type diagnostics are covered by
  `view_binding_rejects_intent_payload_type_mismatch`.
- Trigger conflict diagnostics are covered by
  `view_binding_rejects_trigger_conflicts`.
- Preview-only activation rejection is covered by
  `view_binding_rejects_preview_only_activation`.

## Completion Quality

Completion quality is `bounded_contract`.

Known quality gaps:

- App-hosted binding and intent Designer/Lab UI remain future work.
- Editor-specific binding package persistence and project file IO remain future
  work.
- Game-runtime view-model package loading is not implemented or runtime-proven
  in this slice.
- Concrete editor command execution and concrete game intent execution remain
  owned by their target adapters and are not implemented in this slice.
- Live preview fixture matrices, persistence activation,
  accessibility/performance evidence, and production-readiness hardening remain
  governed by PM-UI-DESIGN-008 through PM-UI-DESIGN-010.
- This slice proves generic binding and intent declaration contracts and
  diagnostics, not a user-visible Interface Lab binding editor or
  runtime-proven command bridge.
- PM-007 did not claim `runtime_proven` or `perfectionist_verified` evidence.

## Drift Check

The phase completion drift check found the implementation aligned with the
accepted PM-007 design and `WR-051` contract:

- Generic binding and intent declaration truth stays in
  `domain/ui/ui_definition`.
- Editor/workbench and game-runtime semantic authority stays outside generic UI
  definitions.
- UI intents describe proposals and cannot encode direct mutation.
- Denied capabilities and unsupported target-profile declarations fail closed
  with typed diagnostics.
- App, runtime, editor shell, provider, gameplay, project, and session state did
  not become binding source truth.
- The remaining milestones stay incomplete and must not be inferred complete
  from this bounded binding-contract slice.
