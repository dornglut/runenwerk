---
title: WR-033 Remove Legacy Tool Surface Identity Closeout
description: Completed bounded implementation closeout for removing legacy ToolSurfaceKind identity from live Workbench paths.
status: completed
owner: editor
layer: domain / app
canonical: true
last_reviewed: 2026-05-20
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-033-remove-legacy-tool-surface-identity/plan.md
  - ../../implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
---

# WR-033 Remove Legacy Tool Surface Identity Closeout

## Result

WR-033 is complete as a bounded platform-foundation implementation slice for
`PM-WB-CAP-001`. Live Workbench state, provider requests, frame artifacts,
create candidates, shell chrome commands, app dispatch, Material Lab provider
routing, and viewport runtime bridge lookups now use stable surface keys and
typed registry/profile data instead of carrying `ToolSurfaceKind` as live
identity.

This does not complete `PM-WB-CAP-001`. WR-034, WR-035, and WR-036 still own
registry-backed profile authority, clean persistence format and unsupported old
schema diagnostics, and the final Material Lab clean migration proof.

## Changed Files

- `domain/editor/editor_shell/src/workspace/state.rs`: removed legacy live
  identity fields and constructors from `ToolSurfaceState`,
  `WorkspaceDefaultToolSurface`, and tab-stack lock state.
- `domain/editor/editor_shell/src/workspace/reducer.rs`: removed legacy
  mutation wrappers and legacy metadata from add, split, reset, lock, and
  replace mutations.
- `domain/editor/editor_shell/src/surface_provider.rs`: removed legacy kind
  fields and accessors from provider requests, resolved frames, create
  candidates, and frame model availability.
- `domain/editor/editor_shell/src/commands/shell_command.rs` and
  `domain/editor/editor_shell/src/commands/map_interactions.rs`: removed
  enum-backed shell command variants and route mappings in favor of stable-key
  commands.
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs`: removed
  legacy surface-switch chrome and built create, split, reset, and lock actions
  from stable keys.
- `domain/editor/editor_shell/src/workspace/profile.rs`,
  `domain/editor/editor_shell/src/workspace/definition_form.rs`,
  `domain/editor/editor_shell/src/workspace/persisted.rs`,
  `domain/editor/editor_shell/src/workspace/projection.rs`, and
  `domain/editor/editor_shell/src/workspace/surface_contract.rs`: kept
  compatibility only at authored/persisted boundaries while preventing legacy
  enum data from becoming live Workbench identity.
- `domain/editor/editor_shell/src/tests.rs`: updated source guards so removed
  live identity APIs cannot be reintroduced in workspace state, provider
  requests, commands, composition, or app dispatch.
- `apps/runenwerk_editor/src/shell/providers/mod.rs`,
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`,
  `apps/runenwerk_editor/src/shell/providers/m6_workspace.rs`,
  `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs`, and
  `apps/runenwerk_editor/src/shell/providers/tool_suite_registry_inspector.rs`:
  moved provider request and routing construction to stable keys plus registry
  metadata.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`,
  `apps/runenwerk_editor/src/shell/controller.rs`,
  `apps/runenwerk_editor/src/shell/state.rs`,
  `apps/runenwerk_editor/src/shell/dispatch/entity_table.rs`, and
  `apps/runenwerk_editor/src/shell/dispatch/sdf_operations.rs`: removed legacy
  command dispatch and mutation calls.
- `apps/runenwerk_editor/src/shell/applied_editor_definition/catalogs.rs` and
  `apps/runenwerk_editor/src/shell/applied_editor_definition/compatibility.rs`:
  expose available tool surfaces as stable keys rather than enum kinds.
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs` and
  `apps/runenwerk_editor/src/runtime/systems/picking.rs`: target viewport
  surfaces by stable key.
- `apps/runenwerk_editor/src/shell/workbench_host.rs`,
  `apps/runenwerk_editor/src/shell/tests.rs`,
  `apps/runenwerk_editor/src/persistence/workspace_layout.rs`,
  `apps/runenwerk_editor/tests/startup_render_smoke.rs`,
  `apps/runenwerk_editor/tests/viewport_architecture_guards.rs`, and
  `apps/runenwerk_editor/tests/viewport_gpu_truth_smoke.rs`: updated tests and
  architecture guards for the stable-key-only live contract.

## Validation

Passed:

```text
cargo fmt
cargo test -p editor_shell tool_suite
cargo test -p editor_shell workspace::profile
cargo test -p editor_shell tool_surface_kind_usage_is_boundary_only_guard
cargo test -p runenwerk_editor --test viewport_architecture_guards
cargo test -p runenwerk_editor workbench_host
cargo test -p runenwerk_editor material_lab_workbench
cargo check -p runenwerk_editor
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
```

The scoped production goal rerun remains the next required workflow step after
these gates.

## Completion Quality

Completion quality: `bounded_contract`.

The slice removes live legacy identity across the WR-033-owned paths and adds
source guards for the removal, but it is not `perfectionist_verified` because
known downstream gaps remain by design:

- WR-034 still owns registry-backed workspace profile authority and removal of
  central legacy default-profile construction.
- WR-035 still owns stable-key-only persistence, old schema rejection, and
  unsupported-schema diagnostics.
- WR-036 still owns the final Material Lab clean migration proof across
  full-editor and standalone hosts.
- Host capability policy, product/service declarations, multi-host presets,
  and external component readiness remain outside WR-033.

## Closeout Decision

Archive WR-033 as completed with this closeout evidence. Continue
`PM-WB-CAP-001` with WR-034 in dependency order after rerunning the scoped
production goal command.
