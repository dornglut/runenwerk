---
title: Tool Suite Registry Inspector Closeout
description: Completion and drift-check record for the stable-key-native Tool Suite Registry Inspector.
status: completed
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-18
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
---

# Tool Suite Registry Inspector Closeout

## Status

Complete as of 2026-05-18.

The Tool Suite Registry Inspector is the first post-migration platform tool
surface built on the completed Tool Suite Registry, WorkbenchHost, V5
persistence, provider-family filtering, and stable-key authority stack.

## Purpose

The Inspector gives the editor a read-only diagnostic surface for the new
platform. It proves that a new tool can be added as a normal suite surface
without introducing a `ToolSurfaceKind` variant or relying on the old
enum-backed switch menu.

## Completion Evidence

- Phase A registers
  `runenwerk.diagnostics.tool_suite_registry_inspector` in the
  `runenwerk.diagnostics` suite with `ToolSurfaceRoute::ProviderOwnedLocal`,
  no legacy `ToolSurfaceKind`, and a stable-key-native
  `ToolSuiteRegistryInspectorProvider`.
- Phase A renders read-only registry and provider rows for installed suites,
  registered surfaces, provider families, provider-family assignments,
  concrete providers, conservative metadata status, and simple diagnostics.
- Phase B adds mounted workspace resolution rows showing stable key, structural
  `PanelKind`, provider family, route, candidate providers, support modes, and
  selected provider or diagnostic status.
- Phase C adds an in-memory V5 persistence preview derived from
  `WorkspaceState::to_persisted_v5()` without calling app workspace-layout save
  functions or writing files.
- Phase D adds structured diagnostics and fail-closed drilldown rows for
  registry, provider-family, provider-assignment, provider-resolution, mounted
  surface, and persistence-preview failures.
- Phase E makes the surface reachable through the Runtime Debug workspace
  profile as a stable-key-only default surface with `PanelKind::Diagnostics` as
  structural layout grouping and `legacy_tool_surface_kind == None`.

## Final Reachability Path

The accepted reachability path is:

```text
Runtime Debug profile
  -> stable-key-only default surface
  -> runenwerk.diagnostics.tool_suite_registry_inspector
  -> PanelKind::Diagnostics structural grouping
  -> diagnostics provider family
  -> ToolSuiteRegistryInspectorProvider
```

This path is stable-key-native. It does not use the old enum-backed switch menu
and does not add `ToolSurfaceKind::ToolSuiteRegistryInspector`.

## Read-Only Guarantees

- The Inspector provider does not mutate `WorkspaceState`.
- The Inspector provider does not mutate the `ToolSuiteRegistry`,
  `ToolSurfaceRegistry`, `ProviderFamilyProviderMap`, or provider registry.
- The Inspector provider does not write V5 workspace layout files.
- Provider matching, provider-family filtering, and graph routing behavior are
  unchanged.
- Dynamic external plugins remain deferred.

## Stable-Key Proof

- `ToolSurfaceStableKey` is the Inspector surface identity.
- `ToolSurfaceKind` is absent for the Inspector and remains a legacy boundary
  enum elsewhere.
- `PanelKind::Diagnostics` is structural shell/layout grouping only.
- V5 persistence round-trips the Inspector surface by stable key with no legacy
  kind metadata.
- Provider resolution selects the Inspector provider through provider-family
  filtering and stable-key-first matching.

## Deferred Work

- Stable-key-native menu and self-authoring integration beyond the Runtime
  Debug profile reachability path.
- Provider ID allocation cleanup: centralize or replace numeric
  `SurfaceProviderId` allocation with symbolic provider IDs.
- Existing standalone `rustfmt` toolchain drift for files that use let-chain
  syntax remains separate from Inspector behavior.
- Product work should now consume the platform, with Material Lab polish as the
  recommended next flagship suite.

## Validation

Closeout validation rerun on 2026-05-18:

- `cargo test -p runenwerk_editor tool_suite_registry_inspector`
- `cargo test -p runenwerk_editor workbench_host`
- `cargo test -p runenwerk_editor workspace_layout`
- `cargo test -p runenwerk_editor providers`
- `cargo test -p editor_shell profile`
- `cargo test -p editor_shell workspace`
- `cargo test -p editor_shell persisted`
- `cargo test -p editor_shell surface_provider`
- `cargo test -p editor_shell tool_suite`
- `cargo test -p editor_shell tool_surface_kind_usage_is_boundary_only_guard`
- `cargo test -p editor_shell shell_graph_routing_has_no_new_domain_specific_graph_dispatch_actions`
- `cargo check -p editor_shell -p runenwerk_editor`
- `task docs:validate`
- `task puml:validate`
