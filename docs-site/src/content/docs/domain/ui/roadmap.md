---
title: UI Substrate and Surface Roadmap
description: Current implementation roadmap for Runenwerk UI substrate and surface semantics.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-04-29
related:
  - ./architecture.md
  - ../../design/active/editor-ui-workspace-tool-surface-architecture.md
  - ../../design/active/workspace-identity-contract-and-migration-map.md
---

# Runenwerk UI Substrate and Surface Roadmap

## Purpose

Track implementation sequencing for UI substrate and surface work from current code truth.

This roadmap is intentionally execution-oriented. Target architecture belongs in active design docs.

## Current Code Truth

Implemented and in use:

- substrate crates are present and active: `ui_math`, `ui_input`, `ui_layout`, `ui_text`, `ui_theme`, `ui_render_data`, `ui_tree`, `ui_runtime`, `ui_widgets`;
- `ui_surface` exists with definition/mount/observation/session/presentation/intent/ratification contracts;
- shell/runtime integration routes core editor flows through prepared `SurfacePresentationModel`, typed `SurfaceIntent`, and host-side ratification adapters;
- runtime viewport routing is structural-first with one explicit bootstrap-only single-viewport seam;
- architecture guard tests enforce no `first_frame()` routing fallback and no `ViewportId(0)` synthesis;
- viewport semantic slot taxonomy remains in `editor_viewport`, with opaque renderer-facing payload slots in `ui_render_data`.

Evidence in code:

- `domain/ui/ui_surface/src/*`
- `domain/ui/ui_runtime/src/*`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/runtime/viewport/routing.rs`
- `apps/runenwerk_editor/tests/viewport_architecture_guards.rs`

## Phases and Status

### Phase 1 - Substrate Ownership Hardening

Status: complete for current scope.

Notes:

- retained tree/runtime/widget ownership is in `domain/ui/*`;
- ad hoc viewport routing fallbacks are removed and guarded.

### Phase 2 - Establish `ui_surface` as Semantic Kernel

Status: complete for baseline scope.

Notes:

- `ui_surface` crate is present and used by production editor flows;
- mounted-surface and capability/session contracts are active.

### Phase 3 - Formal Observation/Session/Presentation/Intent/Ratification Boundaries

Status: partially complete and active.

Completed:

- core outliner/inspector/viewport command paths route through surface presentation and intent contracts.

Remaining:

- extend coverage across additional surface families and non-core interactions;
- keep contract usage consistent in new command paths.

### Phase 4 - Viewport/Embed/Render-Data Seam Consolidation

Status: largely complete, guard-hardened, and still active.

Completed:

- semantic slot ownership is in `editor_viewport`;
- renderer-facing payload ownership is in `ui_render_data`;
- structural binding adapters are active in `runenwerk_editor` runtime seams.

Remaining:

- preserve this boundary while expanding multi-surface coverage and docking/tab behavior.

### Phase 5 - Control Semantics Hardening

Status: active.

Focus:

- broaden use of reusable controls across editor surfaces where ad hoc composition remains;
- keep interaction flows surface-centered and capability-aware.

### Phase 6 - Verification and Docs Hardening

Status: active.

Focus:

- keep guard suites authoritative as behavior evolves;
- keep architecture and roadmap pages synchronized with implemented seams.

## Current Next Steps

1. Finish docking/tab behavior on top of existing structural identity and binding contracts.
2. Expand non-viewport surface maturity (entity-table/query, richer inspector controls) using existing surface contracts.
3. Preserve and extend guard coverage for structural routing, capability gating, and seam ownership.
4. Keep cross-doc sequencing aligned so workspace index docs do not restate stale phase history.

## Non-Goals for This Track

- redesigning renderer architecture;
- introducing full authored editor-definition workflows now;
- collapsing surface semantics into shell or runtime substrate layers;
- moving privileged ratification ownership into generic UI substrate code.
