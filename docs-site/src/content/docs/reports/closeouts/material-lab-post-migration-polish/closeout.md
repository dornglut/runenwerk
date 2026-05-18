---
title: Material Lab Post-Migration Polish Closeout
description: Completion and drift-check record for ML-A through ML-E Material Lab polish after the stable-key Workbench migration.
status: completed
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui
canonical: true
last_reviewed: 2026-05-18
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
related_reports:
  - ../../../reports/closeouts/tool-suite-registry-inspector/closeout.md
  - ../../../reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md
  - ../../../reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/closeout.md
---

# Material Lab Post-Migration Polish Closeout

## Status

Complete as of 2026-05-18.

This closeout records the Material Lab polish track that followed the Tool Suite
Registry, WorkbenchHost, stable-key authority, and Tool Suite Registry Inspector
closeouts. The track used Material Lab as the first product-facing suite to
exercise the completed platform without changing its runtime product behavior.

## Summary

Material Lab now has clearer provider diagnostics, preview/product status,
texture/material reference diagnostics, provider-owned graph interaction proof,
and stable-key workspace/persistence proof. The work is intentionally polish and
evidence, not a product-semantics rewrite.

The accepted final state is:

- `ToolSurfaceStableKey` is the Material Lab suite surface identity.
- Material Lab suite surfaces do not require legacy `ToolSurfaceKind` metadata.
- Existing historical Material Lab `ToolSurfaceKind` variants remain
  compatibility metadata only, not normal authority.
- Material Graph routing remains provider-owned.
- Material providers remain stable-key-first.
- V5 workspace persistence writes Material Lab stable keys as primary identity.
- Preview workflow, product publication, last-good preservation, and texture
  resource resolution behavior remain unchanged.

## Scope

Closed scope:

- Structured provider-facing diagnostics and preview-status presentation.
- Provider-owned Material Graph interaction parity hardening.
- Preview product and last-good current-state surfacing.
- Texture/material resource-binding diagnostic presentation.
- Stable-key workspace and V5 persistence proof for Material Lab surfaces.
- Documentation closeout for the post-migration polish track.

Out of scope:

- Runtime preview behavior changes.
- Product publication behavior changes.
- Texture resource resolution behavior changes.
- Texture viewer handoff or texture graph tooling.
- Product lineage behavior changes beyond current-state surfacing.
- V5 persistence format changes.
- Dynamic plugin behavior.
- New `ToolSurfaceKind` variants or enum-backed menu integration.

## Completed Slices

### ML-A: Diagnostic DTO And Preview-Status Cleanup

`ML-A` added app-neutral Material Lab presentation DTOs under
`domain/editor/editor_shell/src/surfaces/material.rs` and populated them from
existing app runtime state in `apps/runenwerk_editor/src/material_lab/state.rs`.
The Material Graph Canvas, Material Inspector, and Material Preview providers now
render structured diagnostic/status rows while retaining legacy string lines for
compatibility.

No Material Lab workflow behavior, preview publication semantics, provider-owned
graph routing, provider matching, V5 persistence, renderer/product ownership, or
stable-key surface identity changed.

### ML-B: Provider-Owned Graph Interaction Parity

`ML-B` hardened the provider-owned graph route:

```text
GraphCanvasAction
  -> SurfaceInteraction::GraphCanvasAction
  -> MaterialGraphCanvasProvider::map_interaction(...)
  -> Material Lab shell command
```

Supported graph actions are covered by tests, provisional canvas-local gestures
are intentionally ignored, stale projection epochs fail closed, and missing
active material source state fails closed without creating source state.

`editor_shell` continues to emit generic graph-canvas interactions only. It does
not construct Material Lab actions or import Material Graph semantics for graph
interaction mapping.

### ML-C: Preview Product And Last-Good Surfacing

`ML-C` expanded preview-status presentation using existing observable runtime
state: active preview, last workflow status, publication records, artifact
labels, shader labels, viewport product labels, diagnostic counts, and
last-good preservation labels where already available.

This slice surfaced current state only. It did not change preview rebuild timing,
publication acceptance, prior-valid preservation, renderer handoff, or product
publication flow.

### ML-D: Texture/Material Reference Diagnostics

`ML-D` added app-neutral resource-binding diagnostic view models and populated
them from app-owned Material Lab resource observation. The providers now render a
simple Texture / Resource Bindings diagnostic section for observable missing,
ambiguous, incompatible, unsupported, unresolved, generated-available, and
generated-unavailable states.

`resolve_material_resources` remains the resolver path. Texture resolution,
texture domain contracts, preview workflow behavior, and publication behavior did
not change.

### ML-E: Workspace And Persistence Proof

`ML-E` proves Material Lab on the completed stable-key platform:

- `runenwerk.material_lab.graph_canvas` is stable-key-native and uses
  `ToolSurfaceRoute::ProviderOwnedGraphCanvas`.
- `runenwerk.material_lab.inspector` is stable-key-native and uses
  `ToolSurfaceRoute::ProviderOwnedLocal`.
- `runenwerk.material_lab.preview` is stable-key-native and uses
  `ToolSurfaceRoute::ProviderOwnedLocal`.
- The Material Lab suite registers these surfaces with
  `ToolSurfacePersistence::StableKey`.
- The stable-key-only V5 Material Lab fixture writes no Material Lab legacy kind
  metadata.
- V5 round trip preserves graph canvas, inspector, and preview stable keys.
- Loaded Material Lab surfaces resolve providers through stable-key-only requests
  after compatibility metadata is cleared.

Historical V5 load compatibility may still recover legacy metadata for old
Material Lab keys. That metadata remains compatibility metadata only and is not
surface authority.

## Final Authority And Boundary State

- `domain/material_graph` owns material graph source truth, commands,
  validation, ratification, lowering, product contracts, and material semantics.
- `domain/texture` owns texture contracts and formed texture products.
- `domain/editor/editor_shell` owns generic shell/provider/workbench contracts
  and app-neutral Material Lab presentation DTOs only.
- `apps/runenwerk_editor` owns concrete Material Lab app workflow, provider
  state, preview orchestration, asset/catalog integration, texture resource
  resolution, and product publication/handoff.
- `engine/render` consumes formed products and prepared renderer data only.
- No Material Graph semantic mapping exists in `editor_shell` graph routing.

## Final Surface State

- Graph canvas: `runenwerk.material_lab.graph_canvas`,
  `ToolSurfaceRoute::ProviderOwnedGraphCanvas`.
- Inspector: `runenwerk.material_lab.inspector`,
  `ToolSurfaceRoute::ProviderOwnedLocal`.
- Preview: `runenwerk.material_lab.preview`,
  `ToolSurfaceRoute::ProviderOwnedLocal`.

All three surfaces are stable-key-native suite surfaces. The stable-key-only
workspace fixture does not persist legacy Material Lab kind metadata. Historical
legacy enum compatibility remains quarantined to compatibility paths.

Provider-owned graph routing is hardened, preview/product/last-good state is
surfaced, and texture/material binding diagnostics are surfaced.

## Non-Goals And Unchanged Behavior

- No preview workflow behavior change.
- No publication or last-good behavior change.
- No texture resource resolution behavior change.
- No provider matching change.
- No graph routing architecture change.
- No V5 persistence format change.
- No new `ToolSurfaceKind`.
- No dynamic plugins.
- No texture viewer handoff.
- No texture graph tooling.
- No product lineage behavior change beyond current-state surfacing.
- No renderer/product ownership change.
- No Material Lab special cases added back to `editor_shell`.

## Validation Evidence Summary

The polish track was validated phase by phase with focused checks for Material
Lab runtime state, providers, graph canvas interactions, preview status,
resource-binding diagnostics, stable-key surface registration, V5 round trip,
provider resolution, and architecture guards.

Final ML-E closeout validation passed on 2026-05-18:

- `cargo test -p runenwerk_editor material_lab`
- `cargo test -p runenwerk_editor material_graph_canvas`
- `cargo test -p runenwerk_editor material_preview`
- `cargo test -p runenwerk_editor material_inspector`
- `cargo test -p runenwerk_editor workspace_layout`
- `cargo test -p runenwerk_editor providers`
- `cargo test -p runenwerk_editor workbench_host`
- `cargo test -p editor_shell persisted`
- `cargo test -p editor_shell profile`
- `cargo test -p editor_shell workspace`
- `cargo test -p editor_shell surface_provider`
- `cargo test -p editor_shell tool_suite`
- `cargo test -p editor_shell tool_surface_kind_usage_is_boundary_only_guard`
- `cargo test -p editor_shell shell_graph_routing_has_no_new_domain_specific_graph_dispatch_actions`
- `cargo test -p material_graph`
- `cargo test -p texture`
- `cargo check -p editor_shell -p runenwerk_editor`
- `task docs:validate`
- `task puml:validate`

ML-F documentation validation:

- `task docs:validate`
- `task puml:validate`

## Remaining Future Work

- Behavior-changing preview/product polish if focused tests show current
  semantics are insufficient.
- Richer product lineage once stable neutral product identifiers are available.
- Texture2D/Texture3D viewer handoff through stable-key-native routes.
- Deeper material/texture binding workflows.
- Material asset save/load polish.
- Generated shader and product diagnostics beyond current-state surfacing.
- Product publication proof closeout if later behavior work changes publication
  semantics.
- Material Lab UX layout polish.

## Risks And Watchpoints

- Historical Material Lab `ToolSurfaceKind` variants still exist for legacy
  compatibility. Future work must not treat them as normal surface identity.
- Preview publication and last-good behavior are surfaced, not redesigned. Any
  behavior-changing follow-up should be scoped as a separate ML-C2-style slice
  with focused tests.
- Texture/resource diagnostics are observational. Future viewer handoff or
  texture graph tooling must not move app asset-catalog or IO concerns into
  `editor_shell` or `domain/texture`.
- Material Lab remains a product-facing stress case for the stable-key
  Workbench platform; future polish should consume the platform rather than add
  shell special cases.
