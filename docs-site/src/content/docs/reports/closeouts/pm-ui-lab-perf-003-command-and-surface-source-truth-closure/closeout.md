---
title: PM-UI-LAB-PERF-003 Command And Surface Source Truth Closure Closeout
description: Runtime-proven closeout evidence for WR-107 command catalog, toolbar/menu, binding, and surface registry source-truth closure.
status: completed
owner: editor
layer: app/domain
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/accepted/ui-lab-command-surface-source-truth-closure-design.md
  - ../../../design/accepted/ui-lab-perfectionist-audit-design.md
  - ../../../design/accepted/ui-lab-command-catalog-and-surface-registry-design.md
related_reports:
  - ../../implementation-plans/wr-107-ui-lab-command-and-surface-source-truth-closure/plan.md
  - ../pm-ui-lab-perf-002-runtime-evidence-platform-closure/closeout.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# PM-UI-LAB-PERF-003 Command And Surface Source Truth Closure Closeout

## Scope

`WR-107` completes the bounded `PM-UI-LAB-PERF-003` implementation slice:
normal Editor Lab command and surface behavior now has enforced source-truth
audits at the owning catalog and registry boundaries.

This closeout does not claim direct-manipulation UX closure, persistence/API
ergonomics closure, or final no-gap certification. Those remain owned by
`PM-UI-LAB-PERF-004` through `PM-UI-LAB-PERF-006`.

## Implementation Summary

- `apps/runenwerk_editor/src/shell/command_catalog/mod.rs` module
  `command_catalog` extends `EditorCommandCatalog::validate` so the app-owned
  catalog rejects duplicate route targets, missing `KnownEditorCommand`
  descriptors, empty labels, empty static-disabled diagnostics, non-round-trip
  route targets, and toolbar-command reverse lookup drift.
- `apps/runenwerk_editor/src/shell/command_catalog/mod.rs` test
  `checked_in_toolbar_binding_routes_resolve_to_catalog_descriptors` now proves
  checked-in workspace and menu binding labels plus static disabled reasons
  match catalog descriptors.
- `apps/runenwerk_editor/src/shell/toolbar_adapter.rs` functions
  `build_toolbar_observation_frame` and `active_menu_label` now resolve default
  File/Edit/Window menu labels from the catalog route descriptors rather than
  independent literal fallbacks.
- `assets/editor/ui/editor_bindings.ron` updates the
  `editor.toolbar.window.load_custom_workspace` authored menu label to match
  the catalog-owned `Load Custom Workspace` descriptor while preserving the
  disabled reason.
- `apps/runenwerk_editor/src/shell/tests.rs` tests
  `pm_ui_lab_perf_003_command_source_truth_closure` and
  `pm_ui_lab_perf_003_surface_source_truth_closure` prove runtime route action
  projection, checked-in binding metadata, mounted surface request metadata,
  provider-family assignment, and stable-key provider resolution derive from
  the owning catalog or registry.

## Source-Truth Evidence

Command audit results:

- every catalog descriptor route target projects to the exact
  `RoutedShellAction` returned by that descriptor;
- the runtime `EditorShellFrameModel::route_actions_by_route_target` map equals
  `active_route_actions_by_target` output;
- checked-in workspace catalog entries and toolbar menu items resolve to one
  catalog descriptor and use the descriptor label;
- checked-in static disabled reasons match the descriptor availability reason;
- unavailable toolbar diagnostics continue to use
  `EditorCommandCatalog::descriptor_for_toolbar_command`.

Surface audit results:

- Scene, Editor Design, Materials, and Runtime Debug workspace profiles mount
  registry-backed surfaces;
- each mounted `SurfaceProviderRequest` gets provider family, route kind, and
  capabilities from the matching `ToolSurfaceDefinition`;
- every audited provider family has assigned app providers through
  `ProviderFamilyProviderMap`;
- provider resolution observes at least one `StableKey` support path and does
  not require legacy-only or unsupported resolution for normal audited
  surfaces;
- the audit covers `runenwerk.scene.viewport`,
  `runenwerk.editor_design.ui_canvas`,
  `runenwerk.material_lab.graph_canvas`, and
  `runenwerk.diagnostics.tool_suite_registry_inspector`.

Legacy adapter boundaries that remain:

- `KnownEditorCommand`, `ToolbarCommandKind`, and `ToolbarMenuKind` remain
  compatibility enums for dispatch and shell routing, but normal route
  metadata is audited through `EditorCommandCatalog`.
- `ToolSurfaceKind` and `PanelKind` remain compatibility and persistence
  adapter edges in `domain/editor/editor_shell/src/workspace/surface_contract.rs`
  and tool-suite declarations, while normal mounted surface provider metadata
  is audited through `ToolSurfaceDefinition`.
- `domain/ui/ui_definition` remains behavior-free; it stores authored route
  slots, labels, templates, and availability bindings but does not execute
  editor commands or provider behavior.

## Validation

Focused validation completed for this slice:

```text
cargo fmt
cargo test -p runenwerk_editor command_catalog
cargo test -p runenwerk_editor command_source_truth
cargo test -p runenwerk_editor surface_source_truth
cargo test -p editor_shell tool_suite
cargo test -p editor_shell surface_contract
git diff --check
```

Final metadata validation must pass with this closeout, completed roadmap
archive state, and completed production milestone state:

```text
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
task ai:goal -- --track PT-UI-LAB-PERFECTION
```

## Remaining Quality Gaps

- `PM-UI-LAB-PERF-004` still owns direct-manipulation Editor Lab UX closure.
- `PM-UI-LAB-PERF-005` still owns persistence, diff/apply, rollback review,
  public API, prelude, guide, and example ergonomics closure.
- `PM-UI-LAB-PERF-006` still owns final no-gap certification and may claim
  `perfectionist_verified` only after all remaining gaps are closed.
