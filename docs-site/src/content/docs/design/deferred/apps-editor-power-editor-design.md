---
title: apps/editor/power_editor Design
description: Deferred design for editor authoring of gameplay powers, techniques, constraints, consequences, and validation feedback.
status: deferred
owner: editor
layer: app
canonical: true
last_reviewed: 2026-04-27
---

# `apps/editor/power_editor` Design

## Purpose

`apps/editor/power_editor` provides editor-facing authoring tools for Runenwerk's power system.

The crate owns UI workflows for creating, editing, validating, inspecting, previewing, and saving power profile and technique documents.

It does not own power semantics. It uses `domain/gameplay/powers` drafts, ratifiers, validators, diagnostics, definitions, and schema metadata.

## Doctrine Alignment

This crate belongs to editor/tooling and session reality.

It owns editor-facing documents, draft editing state, UI view state, graph authoring state, diagnostics display, and preview orchestration. It does not own power meaning, runtime mutation, authoritative ratification, asset database internals, or retention backends.

Doctrine stance:

- documents/drafts are authored reality;
- graph view/layout/selection are session reality;
- validation reports and lowering reports are observed reality;
- save/export is a migration path;
- runtime preview is session reality unless explicitly promoted;
- no preview or draft edit is ratified by default.

## Scope

In scope:

- power profile editor panels;
- technique editor panels;
- constraint and oath editors;
- cost/effect/risk/consequence editors;
- validation panels;
- graph or structured authoring views;
- editor document state;
- draft editing state;
- dirty state;
- undo/redo command state;
- conversion between UI state and domain drafts;
- graph-to-draft lowering;
- graph lowering diagnostics;
- visualization of diagnostics;
- preview requests;
- asset binding metadata;
- editor commands for creating/updating/deleting drafts;
- doctrine-facing metadata for session, authored, observed, and migration boundaries.

## Non-scope

This crate must not own:

- power validation rules;
- technique semantics;
- oath semantics;
- ECS runtime state;
- runtime execution;
- world mutation;
- renderer effects;
- gameplay input bindings;
- save file format;
- asset database core logic;
- asset migration engine;
- authority ratification;
- retention backend;
- collaboration merge engine.

It may integrate with asset/editor infrastructure but should not define underlying domain rules.

## Architectural Position

Dependency direction:

```text
domain/gameplay/powers
        ↓
apps/editor/power_editor

editor document/asset infrastructure
        ↓
apps/editor/power_editor
```

Allowed dependencies:

- `domain/gameplay/powers`;
- `foundation/diagnostics`;
- editor shell/UI crates;
- editor asset/document crates if they exist;
- doctrine policy value types if provided by foundation/editor infrastructure.

Forbidden dependencies:

- game app crates;
- runtime systems except behind explicit preview adapters;
- renderer-specific runtime logic.

## Reality Classification

| Artifact | Reality | Owner | Notes |
|---|---|---|---|
| `TechniqueDocument` | Authored reality | Power editor/document infrastructure | Open document for technique authoring. |
| `ProfileDocument` | Authored reality | Power editor/document infrastructure | Open document for profile authoring. |
| `TechniqueDraft` | Authored reality | Domain draft contract + editor document | May be invalid. |
| `TechniqueGraph` | Authored/session reality | Editor | Editing representation. |
| `DocumentViewState` | Session reality | Editor | Pan/zoom/selection/collapsed UI state. |
| `DocumentDirtyState` | Session/workflow reality | Editor | Save/publish workflow state. |
| `DocumentAssetBinding` | Workflow/authored boundary | Editor + asset infra | Links document to asset identity. |
| `GraphLoweringReport` | Observed reality | Editor | Derived report, rebuildable. |
| `TechniqueValidationReport` | Observed reality | Power domain | Displayed by editor. |
| `StaticPreview` | Observed/session reality | Editor | No runtime. |
| `DomainPreview` | Observed/session reality | Editor + domain | Uses fake/selected context snapshot. |
| `RuntimeSandboxPreview` | Session/simulated sandbox reality | Preview adapter/runtime sandbox | Non-ratifying by default. |

## Ratification Policy

Editor edits do not automatically ratify content or world change.

Ratification-like boundaries:

| Operation | Ratification Class |
|---|---|
| edit document field | Non-ratifying session/authored change |
| graph lowering | Non-ratifying authoring transformation |
| domain validation | Non-ratifying observed product |
| ratify draft into definition | Content ratification via domain + asset/document policy |
| save document | Document persistence, not necessarily publication |
| publish/export definition asset | Content migration/ratification through asset pipeline |
| runtime sandbox preview | Non-ratifying session simulation |
| promote preview result | Forbidden unless explicit migration/ratification path exists |

The editor must not treat preview success as content or runtime ratification.

## Reconciliation Policy

Default policies:

| Artifact | Reconciliation |
|---|---|
| `TechniqueDocument` | Structure-merged only if editor document infrastructure supports it. |
| `ProfileDocument` | Structure-merged only if editor document infrastructure supports it. |
| `TechniqueGraph` | Structure-merged only if graph merge policy exists. |
| `DocumentViewState` | Session-local only. |
| `DocumentDirtyState` | Session-local/workflow-owned. |
| `DocumentAssetBinding` | Reject-on-conflict or workflow-owned. |
| `GraphLoweringReport` | Rebuildable; not mergeable. |
| `ValidationReport` | Rebuildable; not mergeable. |
| `RuntimeSandboxPreview` | Session-local only. |

No editor document is mergeable by default merely because it is text or graph-like. Mergeability must be earned by document structure and policy.

## Stability and Retention

| Artifact | Stability Class | Retention Strategy |
|---|---|---|
| `TechniqueDocument` | Observationally stable under document revision | State-retained by document infrastructure |
| `ProfileDocument` | Observationally stable under document revision | State-retained by document infrastructure |
| `TechniqueGraph` | Observationally stable under document revision | State-retained if graph is document source |
| `DocumentViewState` | Ephemeral or presentation-stable | Session-only |
| `DocumentDirtyState` | Presentation-stable/workflow-local | Session-only or workflow-retained |
| `GraphLoweringReport` | Observationally stable for same graph revision | Rebuildable |
| `TechniqueValidationReport` | Observationally stable for same draft/context | Rebuildable |
| `StaticPreview` | Presentation-stable | Ephemeral |
| `DomainPreview` | Observationally stable for same context | Ephemeral/rebuildable |
| `RuntimeSandboxPreview` | Ephemeral or partition-stable within sandbox | Ephemeral unless explicitly retained |

## Migration Paths

Important migration paths:

```text
TechniqueDocument
  -> TechniqueDraft
  -> TechniqueDefinition
  -> TechniqueDefinition asset
```

Authoring to ratified content asset.

```text
ProfileDocument
  -> PowerProfileDraft
  -> PowerProfileDefinition
  -> PowerProfileDefinition asset
```

Profile authoring to ratified content asset.

```text
TechniqueGraph
  -> TechniqueDraft
```

Graph-lowering migration inside authored reality.

```text
TechniqueDefinition asset
  -> runtime registry
```

Content publication/formation path owned by asset/runtime infrastructure.

```text
TechniqueDocument
  -> RuntimeSandboxPreview
```

Session preview path. Non-ratifying by default.

Each migration path should declare:

- source scope;
- target scope;
- preconditions;
- side effects;
- compensation behavior;
- terminal failure behavior;
- visibility policy.

## Capability Requirements

Suggested capabilities:

- `OpenPowerDocumentCapability`;
- `EditPowerDocumentCapability`;
- `LowerPowerGraphCapability`;
- `ValidatePowerDraftCapability`;
- `RatifyPowerDraftCapability`;
- `SavePowerDocumentCapability`;
- `PublishPowerAssetCapability`;
- `RunPowerStaticPreviewCapability`;
- `RunPowerDomainPreviewCapability`;
- `RunPowerRuntimeSandboxPreviewCapability`;
- `ObservePowerEditorDiagnosticsCapability`.

Editor APIs should avoid ambient write authority where scoped capabilities are available.

## Core Workflow

```text
User edits TechniqueDocument
  ↓
Document updates TechniqueDraft or graph state
  ↓
Graph/structured authoring lowers to domain draft
  ↓
domain/gameplay/powers ratifies/validates
  ↓
TechniqueValidationReport
  ↓
Power editor displays diagnostics
  ↓
Valid draft can be saved/exported as TechniqueDefinition asset
```

The editor must support invalid intermediate states.

The domain definition should only be produced after ratification.

## Editor Document Model

The editor must distinguish:

```text
PowerEditorDocument
  Open editor document wrapper.

TechniqueDocument
  Open document for one technique draft/graph.

ProfileDocument
  Open document for one profile draft.

DocumentId
  Editor-local document identity.

DocumentRevision
  Incrementing editor revision.

DocumentDirtyState
  Clean/dirty/save-pending/conflicted.

DocumentValidationState
  Unknown/stale/validating/valid/invalid.

DocumentAssetBinding
  Link to asset id/path/registry entry if one exists.

DocumentSelectionState
  Current selected node/field/diagnostic.

DocumentViewState
  UI-only layout, graph pan/zoom, collapsed panels.
```

Domain definitions must not contain document view state.

## Draft Editing

The editor edits drafts, not final domain definitions directly.

Drafts may have:

- missing expression;
- missing target;
- incomplete oath;
- invalid cost;
- unsupported effect combination;
- unconnected graph nodes;
- temporary UI-only metadata.

Domain ratification decides whether a draft becomes a valid definition.

## Graph Authoring

A graph authoring view may exist, but it is only an editing representation.

Graph nodes may lower to:

- requirements;
- constraints;
- costs;
- risks;
- effects;
- consequences;
- targeting rules;
- scaling rules.

Graph layout positions are UI metadata and must not leak into ratified domain types.

## Graph Lowering Diagnostics

Graph-to-draft lowering can fail before domain validation.

The editor should define:

- `GraphLoweringReport`;
- `GraphLoweringDiagnostic`;
- `GraphNodeSubject`;
- `GraphEdgeSubject`.

Example diagnostics:

- unconnected required output;
- missing required node input;
- illegal cycle;
- ambiguous target source;
- duplicate oath trigger;
- disconnected effect chain.

These are editor/authoring diagnostics. They may be converted to `foundation/diagnostics` for display.

## Observation Frames and Diagnostics Display

The editor observes power authoring through declared frames.

Suggested frames:

- `PowerDocumentObservationFrame`;
- `TechniqueGraphObservationFrame`;
- `TechniqueValidationObservationFrame`;
- `PowerDiagnosticObservationFrame`;
- `PowerPreviewObservationFrame`.

The editor should use structured diagnostics from `foundation/diagnostics`.

Display should include:

- severity;
- code;
- message;
- subject;
- location if available;
- suggested fix if available.

The editor must not infer validation rules from text messages. It should rely on stable diagnostic codes and subjects.

Diagnostic sources may include:

- graph lowering;
- draft ratification;
- technique validation;
- profile validation;
- asset binding;
- preview runtime.

## Preview Modes

The editor supports three preview levels.

### Static Preview

No runtime. Shows structure, declared effects, requirements, costs, risks, and possible diagnostics.

### Domain Preview

Uses domain validation/scaling with an editor-provided fake or selected context snapshot.

No ECS/world mutation.

### Runtime Sandbox Preview

Uses an explicit sandbox ECS/world/session.

Must not mutate real game/editor state.

Runtime preview dependencies must be behind preview adapters.

## Asset Binding

The editor does not own the asset database, but it should track asset binding metadata.

A document may be:

- unsaved;
- bound to an existing asset;
- imported from asset;
- exported to asset;
- conflicted with asset version;
- stale relative to schema version.

Asset/document infrastructure owns persistence and migration.

## Undo/Redo and Commands

Editor commands should be explicit and undo-friendly.

Commands may include:

- create profile document;
- create technique document;
- edit field;
- add requirement;
- add constraint;
- add oath;
- add risk;
- add consequence;
- add cost;
- add effect;
- connect graph nodes;
- disconnect graph nodes;
- validate draft;
- ratify draft;
- save document;
- discard changes.

Commands should modify editor document state, not directly mutate ratified domain definitions.

## Panels

Recommended panels:

- profile panel;
- technique panel;
- constraint panel;
- oath panel;
- validation panel;
- effect inspector;
- cost inspector;
- risk inspector;
- consequence inspector;
- asset binding panel;
- preview panel.

The validation panel should group diagnostics by severity, source, and subject.

## Invariants

1. Editor can represent invalid drafts.
2. Ratified domain definitions come only from domain ratification.
3. Editor UI does not duplicate validation rules.
4. UI graph state does not leak into domain semantics.
5. Diagnostics are displayed from structured diagnostic reports.
6. Runtime preview is sandboxed or explicit.
7. Editor document state is separate from final saved domain definitions.
8. The crate does not mutate ECS runtime state directly.
9. Graph lowering diagnostics are distinct from domain validation diagnostics.
10. Asset binding metadata is editor/document state, not domain semantics.
11. Undo/redo operates on editor documents and drafts.
12. Preview is non-ratifying unless an explicit migration/ratification path exists.
13. Session reality does not become authored or ratified reality by accident.
14. Document mergeability must be explicitly declared by document infrastructure.
15. Capability-scoped editor operations are preferred over ambient mutation.

## Suggested Source Layout

```text
apps/editor/power_editor/
  README.md
  design.md
  roadmap.md
  Cargo.toml
  src/
    lib.rs

    documents/
      mod.rs
      power_editor_document.rs
      technique_document.rs
      profile_document.rs
      document_dirty_state.rs
      document_validation_state.rs
      document_asset_binding.rs
      document_revision.rs
      document_view_state.rs

    doctrine/
      mod.rs
      power_editor_reality.rs
      power_editor_reconciliation_policy.rs
      power_editor_stability_class.rs
      power_editor_retention_hint.rs
      power_editor_migration.rs
      power_editor_capability.rs

    panels/
      mod.rs
      profile_panel.rs
      technique_panel.rs
      constraint_panel.rs
      oath_panel.rs
      validation_panel.rs
      asset_binding_panel.rs
      preview_panel.rs

    graph/
      mod.rs
      technique_graph.rs
      technique_node.rs
      technique_edge.rs

    lowering/
      mod.rs
      graph_to_technique_draft.rs
      graph_lowering_report.rs
      graph_lowering_diagnostic.rs
      graph_subject.rs

    observation/
      mod.rs
      power_document_observation_frame.rs
      technique_graph_observation_frame.rs
      technique_validation_observation_frame.rs
      power_preview_observation_frame.rs

    inspectors/
      mod.rs
      profile_inspector.rs
      technique_inspector.rs
      effect_inspector.rs
      cost_inspector.rs
      risk_inspector.rs
      consequence_inspector.rs

    adapters/
      mod.rs
      draft_to_domain.rs
      validation_to_ui.rs
      diagnostic_to_ui.rs
      asset_binding_adapter.rs

    preview/
      mod.rs
      static_preview.rs
      domain_preview.rs
      runtime_sandbox_preview.rs

    commands/
      mod.rs
      create_document.rs
      edit_technique.rs
      edit_profile.rs
      validate_technique.rs
      ratify_technique.rs
      save_document.rs

    state/
      mod.rs
      power_editor_state.rs
      open_documents.rs
```

## Relationship to `domain/gameplay/powers`

The editor uses domain drafts, validators, and ratifiers.

It must not reimplement power rules in UI code.

Good:

```text
UI constructs TechniqueDraft and calls domain ratifier.
```

Bad:

```text
UI decides that oath severity is valid using duplicated hardcoded logic.
```

## Relationship to Runtime

The editor may invoke runtime only through explicit preview adapters.

Runtime integration should be optional and separated from validation-only authoring.

The editor should validate drafts without starting a full game simulation.

## Testing Strategy

Tests should cover:

- document dirty state transitions;
- draft-to-domain adapter behavior;
- validation diagnostics rendered correctly;
- graph-to-draft conversion;
- graph lowering diagnostics;
- invalid draft remains editable;
- ratified draft produces definition;
- UI command state transitions;
- undo/redo behavior for draft changes;
- diagnostics grouping and display mapping;
- preview mode boundaries;
- doctrine metadata mapping;
- non-ratifying preview invariants.

UI-heavy behavior can be tested with editor test-support crates if available.

## Implementation Readiness

This crate should be implemented after:

1. `domain/gameplay/powers` draft/ratification contracts stabilize.
2. Editor document infrastructure exists or is explicitly designed.
3. Diagnostics-to-UI mapping is stable enough.
4. Asset binding expectations are known.
5. Document migration and reconciliation policies are known.
