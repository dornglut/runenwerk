---
title: Surface Workflow Contract Redesign Closeout
description: Completion and drift-check record for the provider-backed editor surface workflow redesign.
status: completed
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-08
related:
  - ../../../design/active/surface-workflow-contract-redesign.md
  - ../../../domain/ui/roadmap.md
  - ../../../apps/runenwerk-editor/roadmap.md
  - ../../../workspace/repo-execution-priority-checklist.md
---

# Surface Workflow Contract Redesign Closeout

## Status

Complete as of 2026-05-08.

The completed slice is the long-term provider-backed surface workflow migration for outliner, entity-table, inspector, viewport, and editor-definition/self-authoring actions. It does not start the next product phase.

## Completion Evidence

- `domain/editor/editor_shell/src/surfaces/` owns surface-specific app-neutral contracts for outliner, entity-table, inspector, viewport, and editor definition workflows.
- `SurfaceLocalAction`, `SurfaceSessionMutation`, and `EditorDomainMutation` use typed wrappers instead of flat per-surface variants.
- `ShellCommand` uses `ApplySurfaceSessionMutation`, `ApplyEditorDomainMutation`, and `DispatchSurfaceLocalAction` for provider-backed surfaces.
- `apps/runenwerk_editor/src/shell/dispatch/` owns surface-specific command handling; `dispatch_shell_command.rs` remains the global facade.
- Entity-table sessions store `EntityTableQuery` and support search, selected-only, roots-only, component filtering, and sorting.
- Inspector fields carry `InspectorFieldControlKind`; bool, numeric, text, enum-disabled-select, read-only, group, and unsupported controls form through retained UI alternatives.
- `ui_definition` remains provider-behavior free. `UiAvailability::Unavailable` omits inactive alternatives, while `UiAvailability::Disabled` forms non-interactive nodes.

## Drift Corrections

- The app and workspace priority checklists previously described non-viewport surface maturity as active/partially implemented. They now mark the slice complete and move remaining UI work to broader reusable-control adoption.
- The editor roadmap previously described richer inspector controls and entity-table/query workflows as remaining product maturity outside M3.7. It now records those workflows as complete and names M4 asset pipeline foundation as the next primary product track.
- The viewport implementation roadmap previously grouped richer non-viewport surfaces into remaining product maturity. It now records that the planned surface workflow follow-up has landed.

## Deferred Work

The closeout intentionally leaves these items for later phases:

- enum inspector mutation contracts in `editor_inspector`;
- broader reusable-control adoption across editor surfaces;
- exhaustive self-authoring field polish, packaging/export hardening, and live activation for non-theme definition families;
- more viewport producer types;
- broader history/workflow coverage;
- M4 SDF/field-first asset pipeline and field-product foundation.

## Validation

Completed validation:

- `cargo test -p editor_shell surface`
- `cargo test -p runenwerk_editor entity_table`
- `cargo test -p runenwerk_editor inspector`
- `cargo test -p runenwerk_editor viewport`
- `cargo test -p runenwerk_editor outliner`
- `cargo test -p runenwerk_editor surface`
- `cargo test -p runenwerk_editor --test viewport_architecture_guards`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`

The originally proposed combined runenwerk editor filter command was not valid Cargo syntax because Cargo accepts a single test filter per invocation; equivalent filters were run separately.
