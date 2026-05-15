---
title: UI Substrate and Surface Roadmap
description: Current implementation roadmap for Runenwerk UI substrate and surface semantics.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-15
related:
  - ./architecture.md
  - ../../design/active/ui-definition-formation-foundation-design.md
  - ../../reports/audits/editor-ui-priority-code-audit-2026-05-05.md
  - ../../design/active/editor-self-authoring-and-final-ui-design.md
  - ../../design/active/editor-ui-runtime-v2-and-interaction-formation-design.md
  - ../../design/active/editor-ui-popup-adornment-drop-preview-contract.md
  - ../../design/active/editor-shell-menu-and-tab-chrome-polish-design.md
  - ../../design/active/editor-ui-workspace-tool-surface-architecture.md
  - ../../design/deferred/ui-model-multiple-execution-strategies-design.md
  - ../../adr/accepted/0009-ui-interaction-formation-v2.md
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
- runtime viewport routing is structural-first. The normal viewport migration path is closed; one compatibility-only bootstrap seam remains in `apps/runenwerk_editor/src/runtime/viewport/routing.rs::resolve_structural_viewport_products`, through private helpers `select_viewport_id_with_bootstrap_policy` and `bootstrap_single_viewport_id`, before first projection artifacts exist. New viewport/product work must use structural viewport binding, not bootstrap selection;
- architecture guard tests enforce no `first_frame()` routing fallback and no `ViewportId(0)` synthesis;
- viewport semantic slot taxonomy remains in `editor_viewport`, with opaque renderer-facing payload slots in `ui_render_data`.
- `ui_definition` exists with authored templates, validation, normalization, retained formation, route/embed products, and checked-in editor UI fixtures; broader workspace profile catalogs/default layouts, unavailable feature representation, and custom workspace catalog behavior still need follow-up hardening.
- ADR 0009 makes Interaction V2 the accepted architecture for popup stack, scroll ownership, focus, menu sizing, chrome slot, docking-zone, and status overflow contracts. Current runtime execution is still retained UI.

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

Status: complete for normal viewport migration, guard-hardened, and active only
for compatibility preservation.

Completed:

- semantic slot ownership is in `editor_viewport`;
- renderer-facing payload ownership is in `ui_render_data`;
- structural binding adapters are active in `runenwerk_editor` runtime seams;
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::populate_viewport_layout_map_from_shell_tree` scans structural `ViewportSurfaceEmbed` nodes;
- `apps/runenwerk_editor/src/runtime/systems/viewport_lifecycle.rs::sync_viewport_instances_system` syncs explicit viewport instances from workspace state.

Remaining:

- preserve the one compatibility-only bootstrap seam in `apps/runenwerk_editor/src/runtime/viewport/routing.rs::resolve_structural_viewport_products`;
- do not use bootstrap selection for new viewport/product work.

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
- visual editor self-authoring implementation and user-authored document lifecycle inside M3.5; those moved to the promoted M3.6 UI self-authoring workspace after this framework became active;
- baking retained `UiNodeKind`, runtime `WidgetId`, concrete shell commands, or ECS component/entity layout into authored UI templates;
- ECS entities as authored UI/editor identities.

Milestone placement:

- the app roadmap inserts this as M3.5 before the promoted M3.6 UI self-authoring workspace and before the integrated M4 UI/editor/asset foundation so new asset/procedural/editor-design workspace work does not add more hard-coded toolbar, menu, workspace, shell chrome, and provider surface structure.

### Phase 8 - UI Runtime V2 Interaction Formation

Status: accepted architecture as of ADR 0009 on 2026-05-15. Runtime
implementation remains retained UI first and migrates by Strangler slices.

Owning design:

- `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`
- `docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md`

Decision:

- keep retained UI as the first execution target;
- insert `FormedInteractionModel` between normalized UI definitions and retained UI products;
- solve popup stack, scroll ownership, focus scope, menu sizing, dock/drop-zone, chrome slot, and status overflow behavior as shared contracts instead of surface-local patches;
- keep compiled-reactive and ECS-driven UI deferred until a separate accepted ADR or active design promotes a concrete target;
- keep renderer output as derived product data, not UI authority.

Required Interaction V2 validation coverage:

- popup/menu layer order;
- scroll clipping;
- wheel ownership and boundary propagation;
- outside-dismiss behavior;
- focus return;
- split-border precedence over tab and chrome hits;
- dock-drag/drop-zone precedence;
- radial/menu hit testing where applicable;
- viewport input receives wheel/pointer input only after UI explicitly declines ownership;
- screenshot or primitive-order harness coverage for visual/frame-order regressions where normal unit tests are insufficient.

Migration spine for each slice:

```text
definition vocabulary
  -> validation rule
  -> FormedInteractionModel record
  -> retained UI formation adapter
  -> ui_runtime enforcement
  -> editor/app guard
```

Retained UI slice catalog:

| Slice | Readiness gate | Downstream consumer |
|---|---|---|
| `IV2-menu-stack` | popup stack/menu scope vocabulary, parent/child anchor validation, retained menu formation, runtime layer/dismiss/focus enforcement, and shell/app guards | toolbar, viewport, tab action, and Switch Type menu polish |
| `IV2-scroll-ownership` | scroll owner/axis/boundary policy, retained scroll formation, runtime input ownership reporting, and viewport fallback guards | scrollable popups, status bars, and panel overflow polish |
| `IV2-menu-sizing` | intrinsic menu sizing, item fill, clamp, and scroll fallback contracts with runtime layout tests | popup width/item stretch fixes |
| `IV2-chrome-slots` | close, active indicator, label, command, and drag slots with shell projection and primitive-order guards | tab/workspace chrome polish |
| `IV2-dock-drop-zones` | tab reorder, split insertion, floating-host, invalid-target, and preview-only drop-zone policy with shell/controller guards | docking preview and tab-strip precedence polish |
| `IV2-status-and-viewport-arbitration` | status overflow and viewport pointer/wheel arbitration with shell projection and app guard tests | FPS/frame-time status and compact viewport metadata polish |

Phase 8 leads WR-024. Shell polish may proceed only as retained UI
implementation after the relevant contract slice is defined, or as bounded
compatibility evidence that names the old path, target formed contract, and
regression guard. It must not define durable popup, scroll, focus, docking,
chrome, status, or viewport-input policy in app or shell code.

## Current Now Tasks

- [x] Finish docking/tab behavior on top of existing structural identity and binding contracts. Status: implemented and test-covered; automated coverage exists for tab reorder, rehome, floating host creation, split resizing, area split/duplicate/reset/close, dynamic split-area composition, and structural identity preservation.
- [x] Expose editor-area/type switching with a retained select/dropdown route. Status: implemented and test-covered; tab chrome renders an editor type selector and maps `SelectChanged` to `SwitchPanelToolSurfaceKind`.
- [x] Add plus/new-tab affordance for tab stacks. Status: implemented and test-covered; tab chrome exposes a plus/new-tab control that allocates panel and tool-surface identities after structural ratification.
- [x] Expand non-viewport surface maturity (entity-table/query, richer inspector controls) using existing surface contracts. Status: implemented as of 2026-05-08; provider-backed surfaces now use typed surface action/session/domain wrappers, entity-table query workflows cover search/selected-only/hierarchy/component filters plus sorting, and inspector controls render bool/numeric/text/enum/read-only alternatives without moving provider behavior into `ui_definition`.
- [x] Complete active UI/editor definition consumption before adding more hard-coded editor surfaces. Status: implemented as of 2026-05-09 for M4A-M4D. Active templates/editor bindings feed the shell frame, command bindings resolve authored route ids to app-owned known commands, shortcuts and menus dispatch through that command spine, active toolbar/menu definitions can replace checked-in fixture menu items, and panel/tool-surface registries drive future shell creation/switch choices without mutating existing workspace state.
- [x] Broaden reusable control adoption in editor surfaces. Status: implemented as of 2026-05-09 for the M4E cleanup. `editor_shell` now owns shared retained surface fixture helpers, shared compact reusable-control polish, and self-authoring control-panel composition; app providers supply DTOs/actions/routes instead of direct reusable-control construction.
- [x] Preserve and extend guard coverage for structural routing, capability gating, and seam ownership. Status: updated as of 2026-05-09; guard coverage rejects direct app-provider reusable-control construction while preserving existing `ui_definition` behavior-isolation and surface-routing checks.
- [ ] Keep future UI execution strategies design-gated. Status: active/open; compiled-reactive UI and ECS-driven UI remain future formation targets only after a new active design or accepted ADR identifies the concrete surface, formation product, invalidation/debug model, and command/ratification boundaries. Any future target must consume normalized definitions plus formed interaction contracts.
- [x] Migrate retained UI slices to Interaction V2 contracts. Status: complete for the named WR-025 retained-slice catalog with code-bearing slices landed for `IV2-menu-stack`, `IV2-scroll-ownership`, `IV2-menu-sizing`, `IV2-chrome-slots`, `IV2-dock-drop-zones`, and `IV2-status-and-viewport-arbitration`. `domain/ui/ui_definition/src/interaction.rs`, `domain/ui/ui_definition/src/form.rs::form_retained_ui`, `domain/ui/ui_definition/src/validate.rs::validate_menu`, `domain/ui/ui_runtime/src/layout/engine.rs::layout_popup`, `domain/ui/ui_runtime/src/input/hit_test.rs`, `domain/ui/ui_runtime/src/input/pointer.rs::dispatch_pointer_event`, `domain/ui/ui_runtime/src/runtime/ui_runtime.rs::dispatch_keyboard_event`, `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_editor_shell_frame_with_docking_visual_state`, `domain/editor/editor_shell/src/composition/build_editor_shell.rs::dock_drop_zone_interaction_model`, `domain/editor/editor_shell/src/composition/build_editor_shell.rs::viewport_surface_interaction_model`, `domain/editor/editor_shell/src/composition/build_viewport_panel.rs::viewport_status_overlay`, and `domain/editor/editor_shell/src/composition/toolbar_definition.rs::project_workspace_close_buttons` now carry the generic menu-stack, focus-return, popup-layer, Escape-dismiss, scroll-boundary ownership, menu sizing, chrome slot, dock/drop-zone, status overflow, and viewport input-arbitration spine.
- [x] Complete the M3.5 UI definition formation framework before M3.6 and M4. Status: implemented and validated; crates, fixtures, retained formation, app fixture validation, toolbar route-slot integration, toolbar popup binding data, normal shell chrome formation, and common provider surface fixture formation exist. Provider behavior remains outside `ui_definition`.
- [x] Implement the promoted UI self-authoring workspace before M4. Status: complete as of 2026-05-06; `editor_definition` owns durable editor schemas and validation guards, `editor_shell` exposes the Editor Design workspace/profile and self-authoring surface kinds, and `runenwerk_editor` loads checked-in UI fixtures as editable documents with validation, retained preview, command diff summaries, retained authoring control routes, UI node/theme/workspace-layout draft edits, and explicit apply/rollback.
- [x] Keep UI Designer visible as the promoted self-authoring path. Status: complete/current; UI Designer is not a missing roadmap item, it is the Editor Design/self-authoring workspace tracked by `docs-site/src/content/docs/design/active/editor-self-authoring-and-final-ui-design.md`.
- [ ] Continue popup/menu and tab chrome polish only under Interaction V2. Status: ready-next/supporting evidence; `docs-site/src/content/docs/design/active/editor-shell-menu-and-tab-chrome-polish-design.md` tracks immediate retained-UI symptoms, but WR-024 follows WR-025 and may only consume the named `IV2-*` contract slices or provide compatibility evidence. It may not own long-term popup, scroll, focus, menu sizing, chrome, docking-zone, status-overflow, or viewport-input contracts.
- [ ] Keep cross-doc sequencing aligned so workspace index docs do not restate stale phase history. Status: active; docs validation currently passes, and this page is aligned with the workspace priority checklist as of 2026-05-08.

## Non-Goals for This Track

- redesigning renderer architecture;
- introducing the visual editor or user-authored document lifecycle inside M3.5;
- collapsing surface semantics into shell or runtime substrate layers;
- moving privileged ratification ownership into generic UI substrate code.
- using ECS entities, runtime widget ids, or shell session ids as durable authored UI/editor identity.
