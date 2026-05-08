---
title: UI Substrate and Surface Roadmap
description: Current implementation roadmap for Runenwerk UI substrate and surface semantics.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-08
related:
  - ./architecture.md
  - ../../design/active/ui-definition-formation-foundation-design.md
  - ../../reports/audits/editor-ui-priority-code-audit-2026-05-05.md
  - ../../design/active/editor-self-authoring-and-final-ui-design.md
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
- retained UI node and runtime interaction support exists for reusable controls including text input, numeric input, toggle, tabs, select, table, tree, scroll, split, and viewport embed;
- runtime viewport routing is structural-first with one explicit bootstrap-only single-viewport seam;
- architecture guard tests enforce no `first_frame()` routing fallback and no `ViewportId(0)` synthesis;
- viewport semantic slot taxonomy remains in `editor_viewport`, with opaque renderer-facing payload slots in `ui_render_data`.
- `ui_definition` exists with authored templates, validation, normalization, retained formation, route/embed products, and checked-in editor UI fixtures; broader workspace profile catalogs/default layouts, unavailable feature representation, and custom workspace catalog behavior still need follow-up hardening.

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

### Phase 7 - UI Definition Formation Framework

Status: M3.5 implemented and validated as of 2026-05-06.

Owning design:

- `docs-site/src/content/docs/design/active/ui-definition-formation-foundation-design.md`

Decision:

- keep the current retained UI tree plus tool-surface/canvas hybrid as the accepted execution path;
- add a compiler-inspired definition/formation layer above the substrate instead of moving editor semantics into `domain/ui`;
- treat authored and normalized UI definitions as execution-neutral source/IR;
- keep compiled-reactive and ECS-driven UI execution deferred unless a future active design or accepted ADR promotes one. If promoted, they should become additional formation targets from the normalized model, not replacements for authored template identity.

Owning crates:

- `domain/ui/ui_definition`
- `domain/editor/editor_definition`

Scope:

- authored UI templates, layout nodes, controls, menu/popover intent, theme references, route slots, value slots, collection slots, repeaters, template refs, embed slots, and availability references;
- stable authored UI ids distinct from `WidgetId`, focus/capture ids, `PanelInstanceId`, and `ToolSurfaceInstanceId`;
- validation, normalization, diagnostics, source/path maps, and a first retained-tree formation target;
- generic availability descriptors so disabled/unavailable UI can render without routing fake commands;
- editor-specific bindings for toolbar, menus, workspace catalogs, shell chrome, and common provider surface templates in `domain/editor/editor_definition`.

Implemented first-slice evidence:

- `domain/ui/ui_definition` is an active workspace crate with the source model modules, validation, normalization, and retained formation.
- `domain/editor/editor_definition` is an active workspace crate with toolbar, menu, workspace, surface, command route, availability, binding, validation, and editor UI formation helpers.
- Checked-in fixtures under `assets/editor/ui/` parse and validate in tests.
- `editor_shell` forms the toolbar fixture, toolbar popup item bindings, and maps formed route slots to existing shell actions; route slots remain inert until shell mapping.
- `editor_shell` forms normal tab-stack shell chrome from `assets/editor/ui/shell_chrome.ron` while dynamic docking previews remain live shell behavior.
- `editor_shell` forms common provider surface structure from `assets/editor/ui/surfaces/*.ron` for console, inspector, outliner, entity-table, and viewport. Provider data, route proposals, follow-scroll state, viewport overlays, and render/embed payload ownership remain outside templates.
- `apps/runenwerk_editor` owns checked-in fixture loading and validation policy.

Remaining follow-up after M3.5 closeout:

- continue broadening reusable controls and richer surface workflows without moving provider semantics into UI definitions.

Non-goals:

- moving editor workspace profiles, panel/tab/tool-surface semantics, shell routing policy, provider execution, or app IO into `domain/ui/ui_definition`;
- app provider registries, file IO, or runtime instantiation;
- visual editor self-authoring implementation and user-authored document lifecycle inside M3.5; those move to the M3.6 UI self-authoring workspace after this framework is active;
- baking retained `UiNodeKind`, runtime `WidgetId`, concrete shell commands, or ECS component/entity layout into authored UI templates;
- ECS entities as authored UI/editor identities.

Milestone placement:

- the app roadmap inserts this as M3.5 before the promoted M3.6 UI self-authoring workspace and before the integrated M4 UI/editor/asset foundation so new asset/procedural/editor-design workspace work does not add more hard-coded toolbar, menu, workspace, shell chrome, and provider surface structure.

## Current Now Tasks

- [x] Finish docking/tab behavior on top of existing structural identity and binding contracts. Status: implemented and test-covered; automated coverage exists for tab reorder, rehome, floating host creation, split resizing, area split/duplicate/reset/close, dynamic split-area composition, and structural identity preservation.
- [x] Expose editor-area/type switching with a retained select/dropdown route. Status: implemented and test-covered; tab chrome renders an editor type selector and maps `SelectChanged` to `SwitchPanelToolSurfaceKind`.
- [x] Add plus/new-tab affordance for tab stacks. Status: implemented and test-covered; tab chrome exposes a plus/new-tab control that allocates panel and tool-surface identities after structural ratification.
- [x] Expand non-viewport surface maturity (entity-table/query, richer inspector controls) using existing surface contracts. Status: implemented as of 2026-05-08; provider-backed surfaces now use typed surface action/session/domain wrappers, entity-table query workflows cover search/selected-only/hierarchy/component filters plus sorting, and inspector controls render bool/numeric/text/enum/read-only alternatives without moving provider behavior into `ui_definition`.
- [ ] Complete active UI/editor definition consumption before adding more hard-coded editor surfaces. Status: next/open; active templates and editor bindings already feed the shell frame, but the remaining order is command-binding resolver first, shortcut dispatch second, menu/toolbar consumption third, and panel/tool-surface registry projection fourth. Command bindings must remain route-id-to-known-command mappings, shortcuts and menus must resolve through that app-owned command spine, and panel/tool-surface registries must drive future shell creation/switch choices without mutating existing workspace state.
- [ ] Broaden reusable control adoption in editor surfaces. Status: active/open; controls exist in `domain/ui/*`, but shell surfaces still contain panel-specific composition and some ad hoc row/button patterns where retained tree/table/numeric/toggle/select controls should become the default.
- [ ] Preserve and extend guard coverage for structural routing, capability gating, and seam ownership. Status: active; current guard coverage exists in `apps/runenwerk_editor/tests/viewport_architecture_guards.rs` and related shell/provider tests.
- [ ] Keep future UI execution strategies design-gated. Status: active/open; compiled-reactive UI and ECS-driven UI remain future formation targets only after a new active design or accepted ADR identifies the concrete surface, formation product, invalidation/debug model, and command/ratification boundaries.
- [x] Complete the M3.5 UI definition formation framework before M3.6 and M4. Status: implemented and validated; crates, fixtures, retained formation, app fixture validation, toolbar route-slot integration, toolbar popup binding data, normal shell chrome formation, and common provider surface fixture formation exist. Provider behavior remains outside `ui_definition`.
- [x] Implement the promoted UI self-authoring workspace before M4. Status: complete as of 2026-05-06; `editor_definition` owns durable editor schemas and validation guards, `editor_shell` exposes the Editor Design workspace/profile and self-authoring surface kinds, and `runenwerk_editor` loads checked-in UI fixtures as editable documents with validation, retained preview, command diff summaries, retained authoring control routes, UI node/theme/workspace-layout draft edits, and explicit apply/rollback.
- [ ] Keep cross-doc sequencing aligned so workspace index docs do not restate stale phase history. Status: active; docs validation currently passes, and this page is aligned with the workspace priority checklist as of 2026-05-08.

## Non-Goals for This Track

- redesigning renderer architecture;
- introducing the visual editor or user-authored document lifecycle inside M3.5;
- collapsing surface semantics into shell or runtime substrate layers;
- moving privileged ratification ownership into generic UI substrate code.
- using ECS entities, runtime widget ids, or shell session ids as durable authored UI/editor identity.
