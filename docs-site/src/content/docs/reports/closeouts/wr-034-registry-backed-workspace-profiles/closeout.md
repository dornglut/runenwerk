---
title: WR-034 Registry-Backed Workspace Profiles Closeout
description: Completed bounded implementation closeout for building Workbench workspace profiles from installed composition profile data.
status: completed
owner: editor
layer: domain / app
canonical: true
last_reviewed: 2026-05-20
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-034-registry-backed-workspace-profiles/plan.md
  - ../../implementation-plans/wr-033-remove-legacy-tool-surface-identity/plan.md
  - ../../implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
---

# WR-034 Registry-Backed Workspace Profiles Closeout

## Result

WR-034 is complete as a bounded platform-foundation implementation slice for
`PM-WB-CAP-001`. Full-editor and standalone Material Lab workspace profiles now
come from installed Workbench composition profile declarations, and shell
bootstrap, workspace switching, and provider document filtering consume the
active `RunenwerkWorkbenchHost` profile registry instead of app-shell global
profile lookups.

This does not complete `PM-WB-CAP-001`. WR-035 still owns clean persistence
format work and unsupported old-schema diagnostics. WR-036 still owns final
Material Lab clean migration proof across full-editor and standalone hosts.

## Changed Files

- `apps/runenwerk_editor/src/shell/workbench_host.rs`: added compiled
  full-editor and Material Lab profile specs, feeds them into
  `WorkbenchCompositionBuilder`, builds host-owned `WorkspaceProfileRegistry`
  values from registry-resolved stable surface keys, and exposes profile access
  through `RunenwerkWorkbenchHost`.
- `domain/editor/editor_shell/src/workspace/profile.rs`: added
  `WorkspaceProfile::from_tool_suite_profile_definition`, removed legacy
  fallback population from
  `WorkspaceProfile::build_default_workspace_state_with_registry`, and changed
  default profile construction away from central `ToolSurfaceKind` lists.
- `apps/runenwerk_editor/src/shell/state.rs` and
  `apps/runenwerk_editor/src/runtime/resources.rs`: bootstrap shell workspaces
  from host-selected profile registries and the active host surface registry.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`: resolves
  profile navigation, profile loading, and saved-layout compatibility against
  the active Workbench host profiles.
- `apps/runenwerk_editor/src/shell/providers/mod.rs` and
  `apps/runenwerk_editor/src/shell/providers/tool_suite_registry_inspector.rs`:
  resolve workspace document filtering and inspector mounted-surface
  observations through host profile data.
- `apps/runenwerk_editor/src/shell/workbench_host.rs`,
  `apps/runenwerk_editor/src/shell/tests.rs`, and
  `apps/runenwerk_editor/src/shell/providers/tests.rs`: updated coverage for
  host-backed profile mounting and provider resolution.
- `domain/editor/editor_shell/src/tests.rs`: added source guards that reject
  central legacy default-profile helpers returning as profile authority.

## Validation

Passed:

```text
cargo fmt
cargo test -p editor_shell workspace::profile
cargo test -p editor_shell stable_key_authority_is_end_to_end_guard
cargo test -p runenwerk_editor workbench_host
cargo check -p runenwerk_editor
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
```

## Completion Quality

Completion quality: `bounded_contract`.

The slice removes central legacy default-profile authority from the Workbench
profile path and proves full-editor and standalone Material Lab host-backed
profile mounting. It is not `perfectionist_verified` because known downstream
gaps remain by design:

- WR-035 still owns stable-key-only persistence cleanup, V1-V4 loader removal,
  V5 legacy fallback removal, and unsupported-schema diagnostics.
- WR-036 still owns final Material Lab clean migration proof across full-editor
  and standalone hosts.
- Host capability policy, product/service declarations, multi-host presets,
  and external component readiness remain outside WR-034.

## Closeout Decision

Archive WR-034 as completed with this closeout evidence. Continue
`PM-WB-CAP-001` with the next dependency-order WR only after roadmap and
production gates pass and the scoped production goal command is rerun.
