---
title: Runenwerk Editor Final Implementation Roadmap
description: Repo-truth implementation roadmap from current editor MVP state to feature-complete UI, editor, asset pipeline, runtime preview, and self-authoring workflows.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-08
related_designs:
  - ../../design/active/workspace-field-world-and-simulation-platform-design.md
  - ../../design/active/ui-definition-formation-foundation-design.md
  - ../../design/active/editor-workspace-document-mode-panel-architecture.md
  - ../../design/active/editor-ui-workspace-tool-surface-architecture.md
  - ../../design/active/editor-native-multi-window-presentation-design.md
  - ../../design/active/workspace-viewport-expression-upgrade-design.md
  - ../../design/active/render-product-surface-foundation-bundle-design.md
  - ../../design/active/viewport-dynamic-product-target-allocation-design.md
  - ../../design/active/editor-self-authoring-and-final-ui-design.md
  - ../../design/active/editor-asset-pipeline-and-content-workflow-design.md
  - ../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ../../design/active/gameplay-graph-atr-ir-and-ecs-lowering-design.md
  - ../../design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
related_roadmaps:
  - ../../domain/ui/roadmap.md
  - ../../engine/plugins/render/docs/roadmap.md
related_reports:
  - ../../reports/audits/editor-ui-priority-code-audit-2026-05-05.md
related:
  - ./execution-priority-checklist.md
  - ./current-architecture.md
  - ./viewport-expression-implementation-roadmap.md
  - ./mvp/first-3d-editor-mvp.md
  - ./mvp/acceptance-criteria.md
  - ./mvp/implementation-sequence.md
  - ../../domain/sdf/README.md
  - ../../domain/world-sdf/README.md
  - ../../domain/world-ops/README.md
  - ../../domain/spatial/README.md
  - ../../domain/chunking/README.md
---

# Runenwerk Editor Final Implementation Roadmap

## Goal

Make the Runenwerk editor feature-complete end to end:

- project creation and project loading;
- document tabs for scenes, prefabs, SDF and field-world documents, UI, graphs, scripts, materials, textures, particles, physics, animation, procedural generation, gameplay graphs, gameplay rules/triggers/abilities/quests, imports, runtime/debug documents, and editor definitions;
- task workspaces with scoped modes and mode-aware tools;
- reusable dock/tab/area UI with editor type switching and new-tab creation;
- complete SDF-first 3D scene authoring with transform tools, common entity actions, SDF primitive and brush workflows, inspector/component editing, undo/redo, persistence, validation, and viewport feedback;
- an SDF/field-world-first asset pipeline that imports, validates, forms field products, caches, previews, hot-reloads, and publishes authored content;
- authoring workspaces for UI, graphs/materials/textures, SDF modeling, procedural generation, gameplay graph ATR IR, particles, physics, animation, scripting, debugging, and editor design;
- play/simulate/runtime preview with explicit data hot reload and code rebuild boundaries;
- current self-authoring workflows for workspace layouts, UI layouts, menus, shortcuts, themes, command bindings, and tool-surface definitions before later asset/procedural/gameplay editors are built on top of more hard-coded UI.

This is not a deferral list. Every phase below is required for feature completion. Later phases are ordered after their prerequisites, not optional.

## Repo Truth Audit

Current implemented baseline:

- Editor MVP acceptance is closed in `docs-site/src/content/docs/apps/runenwerk-editor/execution-priority-checklist.md`.
- Workspace profile identity exists in `domain/editor/editor_shell/src/workspace/profile.rs::WorkspaceProfile`.
- Profile-addressed workspace layout persistence exists in `apps/runenwerk_editor/src/persistence/workspace_layout.rs::default_workspace_layout_path_for_profile`.
- Structural workspace identities exist in `domain/editor/editor_shell/src/workspace/state.rs::WorkspaceState`.
- Tab reorder, tab rehome, floating host placeholders, and split resizing exist in `domain/editor/editor_shell/src/workspace/reducer.rs::WorkspaceMutation`.
- Provider DTOs exist in `domain/editor/editor_shell/src/surface_provider.rs`.
- Concrete app providers exist in `apps/runenwerk_editor/src/shell/providers/mod.rs::EditorSurfaceProviderRegistry`.
- Retained UI substrate and widgets exist in `domain/ui/*`, including select, tree, table, tabs, toggle, numeric input, text input, scroll, split, and viewport embed.
- Scene file migration, normalization, and formation exist in `domain/editor/editor_persistence/src/scene_migration.rs`, `domain/editor/editor_persistence/src/scene_normalization.rs`, and `domain/editor/editor_persistence/src/scene_formation.rs`.
- SDF primitives, composition, transforms, sampling, gradients/normals, and queries exist in `domain/sdf/src/field.rs::SdfField3`, `domain/sdf/src/primitives/`, `domain/sdf/src/ops/`, and `domain/sdf/src/queries/`.
- World edit operation logs, dirty-region tracking, build queues, and invalidation contracts exist in `domain/world_ops/src/operation_log.rs`, `domain/world_ops/src/dirty.rs`, `domain/world_ops/src/build_queue.rs`, and `domain/world_ops/src/region_invalidation.rs`.
- World-scale SDF chunk/page payload records and collision query contracts exist in `domain/world_sdf/src/storage.rs::SdfChunkStore` and `domain/world_sdf/src/collision.rs::CollisionQueryService`.
- Spatial coordinates, chunk coordinates, clipmap windows, and chunk residency planning exist in `domain/spatial/src/` and `domain/chunking/src/`.
- Scene manifests are discovered from loose RON files by `engine/src/plugins/scene/manifest/catalog.rs::load_scene_manifest_descriptors`.
- Render import contracts in `engine/src/plugins/render/resource/import.rs` describe render resources only; they are not a general asset pipeline.
- Render prepared-frame material plumbing exists in `engine/src/plugins/render/frame/contributions.rs::PreparedMaterialFeatureContribution`, and the material feature slot exists as `engine/src/plugins/render/features/mod.rs::MATERIAL_RENDER_FEATURE_ID`.
- Shader reload helpers exist in `engine/src/plugins/render/shader/hot_reload.rs::poll_shader_hot_reload` and `engine/src/plugins/shared/reload.rs`.
- `assets/editor/config.ron` contains a Blender path, and `assets/models/*.blend` plus `assets/models/*.glb` exist, but those files are foreign-source/reference content today. They are not the canonical world representation and no importer/catalog/cache pipeline currently owns them.

Current post-M3 gaps:

- M1 structural seams are closed: `DocumentKind` has the explicit M1 taxonomy, `EditorSession` owns ordered document tabs, active switching, dirty/save/close transitions, document compatibility validation, and mode ids/descriptors/registry compatibility rules; app-local generic document-tab runtime state is split from scene-specific document state.
- M2 shell seams are closed: tab chrome, editor type switching, new-tab allocation, close/split/duplicate/reset area commands, dynamic split composition, projected-host split resizing, and workspace layout persistence have automated coverage.
- M3 scene-authoring seams are closed: scene command intents cover child creation, subtree duplication, batch delete, SDF primitive creation, transform set/reset, and component add/remove; rotate/scale viewport tools, transform preview, retained outliner tree rows, common reflected inspector editing, SDF authoring DTOs, and normalized save/load paths have focused coverage.
- The M3.5 UI definition/formation closeout is implemented: `domain/ui/ui_definition`, `domain/editor/editor_definition`, checked-in RON fixtures under `assets/editor/ui/`, retained formation, inert route/embed products, toolbar/menu fixture formation, normal shell chrome formation, common provider surface fixture formation, and app-owned fixture validation exist. Provider data, viewport overlays, editor mutations, and route execution remain outside `ui_definition`.
- There is no asset catalog, asset id model, dependency graph, import plan, artifact cache, asset browser, import diagnostics surface, project-wide asset hot reload workflow, field-product formation pipeline, SDF/world asset taxonomy, or `world_sdf` artifact/cache bridge.
- There is no `domain/material_graph`, `domain/texture`, `domain/procgen`, `domain/particles`, `domain/physics`, or `domain/animation`.
- There are no editor providers for material graph editing, procedural texturing, Texture3D/volume inspection, procedural generation preview, particles, physics authoring/debug, animation timeline, curve editing, or simulation preview.
- There is no `domain/gameplay_graph`, Action/Trigger/Rule IR, gameplay graph compiler, gameplay graph to ECS query/event/schedule lowering, or gameplay graph editor/debug provider.

## Implementation Readiness

- M1 through M3 are complete against current editor, shell, UI, scene, SDF, and persistence docs.
- M3.5 is closed as the UI/editor infrastructure slice: the closeout candidate landed on 2026-05-06, and the follow-up toolbar/provider fixture migration seams were closed afterward.
- M3.6 is complete as of 2026-05-06 for authored definition editing, retained preview, and explicit apply/rollback snapshots. Follow-up self-authoring maturity now wires applied theme definitions into the live editor host theme, forms applied workspace layout definitions into live `WorkspaceState`, exports definitions as versioned packages, and activates UI template/editor-binding/menu/shortcut/command-binding/panel-registry/tool-surface-registry catalogs before the next shell frame.
- M3.7 is complete as a no-compromise viewport expression architecture closeout as of 2026-05-08. Multi-viewport previews now have explicit viewport instances, viewport-scoped products, per-viewport render jobs, persisted restore metadata and runtime settings, lifecycle-before-frame-submit sync, viewport-keyed camera/debug/root commands, camera orbit/pan/zoom routing, and duplicate/close lifecycle cleanup. The follow-on provider surface workflow redesign and surface/product maturity pass are also complete as of 2026-05-08 for typed surface wrappers, entity-table query workflows, inspector enum mutation routing, reusable-control polish, visible descriptor-only field/atlas/volume/brickmap/history viewport products, and guard coverage. The next primary product track is an integrated UI/editor/asset foundation: full live consumption of active UI/editor definition catalogs, panel/tool-surface registry replacement in projected shell choices, and the first SDF/field-first asset catalog/import surfaces.
- Native multi-window editing is designed in `docs-site/src/content/docs/design/active/editor-native-multi-window-presentation-design.md`. It follows the render product-surface foundation and should land before second-monitor workflows are treated as product-ready.
- M4 is no longer a standalone asset-only track. It is the integrated UI/editor/asset foundation: UI live replacement comes first, then the new `domain/asset` crate and first asset/import/field-product surfaces land on top of those active editor definition seams. M5 remains the execution and hot-reload track after the M4 contracts exist.
- M6 is not one implementation ticket. It is implementation-ready only per sub-milestone after the owning first-slice design and domain contract docs exist.
- M7 is implementation-ready only for preview/play/session boundaries first. Gameplay graph, particles, physics, animation, procgen, and simulation hot reload depend on their formed-product contracts from M6.
- Later self-authoring packaging/extensibility is implementation-ready for the retained UI path only. Compiled-reactive or ECS-driven UI execution remains blocked; neither strategy was promoted before M2, and any future promotion requires a separate active design or accepted ADR plus a roadmap update.
- M9 is release-readiness verification, not a feature construction phase.

## Milestones

### M0 - Governance And Evidence Baseline

Exit criteria:

- This roadmap is canonical for app-level sequencing.
- Asset pipeline architecture is captured in `docs-site/src/content/docs/design/active/editor-asset-pipeline-and-content-workflow-design.md`.
- Procedural authoring, material/texturing, particles, physics, animation, and simulation workflows are captured in `docs-site/src/content/docs/design/active/editor-procedural-content-and-simulation-workflow-plan.md`.
- Gameplay graph ATR IR, compiler passes, SDF physics relations, and ECS query/event/schedule lowering are captured in `docs-site/src/content/docs/design/active/gameplay-graph-atr-ir-and-ecs-lowering-design.md`.
- The roadmap explicitly follows the SDF-first field-world direction in `docs-site/src/content/docs/design/active/workspace-field-world-and-simulation-platform-design.md`, `docs-site/src/content/docs/domain/sdf/README.md`, and `docs-site/src/content/docs/domain/world-sdf/README.md`.
- UI execution strategy is closed for M1 through M7 and M3.5/M3.6: retained tree UI plus tool-surface/canvas hybrid is the implementation path. Compiled-reactive or ECS-driven UI execution remains deferred and may not enter self-authoring as a first-time decision.
- Existing MVP, UI, editor, render, and runtime design docs link to this roadmap without restating stale phase order.
- `python3 tools/docs/validate_docs.py` passes.

### M1 - Editor Structural Core Closed

Status: complete as of 2026-05-05. The M1 scope is implemented and covered by focused editor core, scene, shell, app, scene-authoring smoke, viewport architecture guard, formatting, and docs validation checks. M2 and M3 are also complete; M3.5 UI definition formation framework is validated, M3.6 UI self-authoring is complete as of 2026-05-06, and the integrated M4 UI/editor/asset foundation is next while procedural domains and gameplay graph remain deferred.

Purpose: close the structural seams that every later feature depends on.

Implementation targets:

- `domain/editor/editor_core/src/document.rs::DocumentKind`
  - replace the coarse `Asset`, `Resource`, `Tool`, and catch-all usage with explicit document kinds for scene, prefab, SDF graph, SDF brush/layer, field-world definition, field product preview, material graph, material, procedural texture, Texture3D/volume texture, procedural generation graph, gameplay graph, gameplay rule/trigger, ability, quest, particle graph, particle emitter, physics scene/config, animation clip, animation graph, timeline, UI layout, graph, script, foreign mesh/reference import, asset catalog, runtime debug, workspace definition, theme, shortcut, menu, command binding, panel registry, and tool-surface definition.
- `domain/editor/editor_core/src/session.rs::EditorSession`
  - add ordered document-tab state, active document switching commands, close/dirty/save state transitions, and compatibility validation hooks.
- `apps/runenwerk_editor/src/editor_runtime/document/mod.rs`
  - split scene runtime document state from generic document-tab runtime state.
- `domain/editor/editor_shell/src/surface_provider.rs`
  - close the provider DTO contract for workspace profile, document kind, surface definition, capabilities, and provider-local route results.
- `apps/runenwerk_editor/src/shell/providers/`
  - split the current monolithic provider module into subdomain modules with `mod.rs` boundaries: `outliner`, `entity_table`, `viewport`, `inspector`, `console`, and provider registry composition.
- `domain/editor/editor_core/src/ratification.rs`
  - add a domain-owned scene-change ratification parameter object for operation orchestration.
- `domain/editor/editor_scene/src/operations/execute_scene_command.rs`
  - move single-command scene operation orchestration behind narrow domain-owned context traits.
- `domain/editor/editor_scene/src/operations/execute_scene_transaction.rs`
  - move transaction orchestration behind the same context family.
- `apps/runenwerk_editor/src/editor_runtime/runtime.rs::RunenwerkEditorRuntime`
  - implement those context traits without moving app IO, renderer, or engine integration into domain crates.
- `domain/editor/editor_core/src/session.rs::EditorMode`
  - replace the global enum with mode ids, mode descriptors, and workspace/document compatibility rules.

Validation:

- `cargo test -p runenwerk_editor -p editor_core -p editor_scene -p editor_shell`
- `cargo test -p runenwerk_editor --test scene_authoring_workflow_smoke`
- `cargo test -p runenwerk_editor --test viewport_architecture_guards`

### M2 - Final Shell, Docking, Tabs, And Area UI

Purpose: productize the editor shell as a real work surface, not a fixed MVP panel layout.

Status: complete as of 2026-05-05. The reducer, persistence, shell command, tab chrome, interaction mapping, app allocator/pruning seams, dynamic workspace graph projection/composition, and projected-host split resizing are implemented and covered by focused shell/app tests. The old fixed-layout projection remains only as a compatibility view for legacy default-layout guards.

Implementation targets:

- `domain/editor/editor_shell/src/workspace/reducer.rs::WorkspaceMutation`
  - add workspace mutations for new panel tab allocation, close tab, close other tabs, split area horizontally, split area vertically, duplicate area, close area, reset area, lock area type, and saved layout preset application.
- `domain/editor/editor_shell/src/workspace/state.rs::WorkspaceState`
  - preserve `PanelInstanceId` and `ToolSurfaceInstanceId` identity through every new structural mutation.
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_strip_from_frame`
  - render editor type selector, area tabs, plus/new-tab button, split/menu/close controls, and consistent area chrome.
- `domain/editor/editor_shell/src/commands/shell_command.rs::ShellCommand`
  - add commands for new panel tab creation, area close, tab close, area split, area menu actions, and document-tab actions.
- `domain/editor/editor_shell/src/commands/map_interactions.rs::map_interactions_to_shell_commands`
  - map `UiInteraction::SelectChanged` to `SwitchPanelToolSurfaceKind`, map tab controls to tab commands, map tree/table/toggle/numeric interactions for shell-owned controls, and keep provider-local actions routed through provider proposals.
- `apps/runenwerk_editor/src/shell/state.rs::RunenwerkEditorShellState`
  - allocate new panel and tool-surface identities through `WorkspaceIdentityAllocator` and prune surface sessions only after structural state is ratified.
- `domain/editor/editor_shell/src/workspace/persisted.rs`
  - persist the full saved workspace layout, including tab stacks, active panels, floating hosts, split fractions, area type locks, and durable layout identity.

Validation:

- `cargo test -p editor_shell`
- `cargo test -p ui_runtime -p ui_widgets`
- `cargo test -p runenwerk_editor -p editor_shell -p ui_runtime`

### M3 - Scene Authoring Feature Complete

Purpose: finish the core 3D editor before expanding into every other workspace.

Status: complete as of 2026-05-05. The M3 scope is implemented and covered by focused scene authoring smoke, app runtime, shell, inspector, `editor_scene`, formatting, docs validation, and full gate checks. M3.5 UI definition formation framework is implemented and validated as of 2026-05-06; M3.6 UI self-authoring is complete and the integrated M4 UI/editor/asset foundation follows it.

Implementation targets:

- `domain/editor/editor_scene/src/scene_command.rs::SceneCommandIntent`
  - add duplicate entity/subtree, create SDF primitive, create child, batch delete, set transform, reset transform, and add/remove common component actions.
- `apps/runenwerk_editor/src/editor_features/scene_commands.rs`
  - expose create, delete, duplicate, rename, reparent, add component, remove component, and common SDF primitive actions as shell/provider commands.
- `apps/runenwerk_editor/src/editor_features/viewport/tools.rs`
  - add rotate and scale tool definitions alongside select and translate.
- `apps/runenwerk_editor/src/editor_features/viewport/interaction.rs`
  - implement rotate and scale gizmo interaction, snap support, cancellation, preview, and commit behavior through scene transactions.
- `apps/runenwerk_editor/src/editor_runtime/transform_preview.rs`
  - generalize preview state for translate, rotate, and scale without making preview state authoritative.
- `domain/editor/editor_inspector/src/model/`, `domain/editor/editor_inspector/src/editing.rs`, and `apps/runenwerk_editor/src/editor_panels/inspector_panel.rs`
  - make common reflected fields editable through reusable controls and typed inspector edit values.
- `domain/editor/editor_scene/src/sdf_authoring/`
  - add scene-authoring contracts for SDF primitives, boolean composition intent, brush/layer metadata, and SDF preview diagnostics without making scene graph ownership replace `domain/sdf` or `domain/world_sdf`.
- `apps/runenwerk_editor/src/editor_features/viewport/sdf_tools.rs`
  - expose SDF brush, add/subtract, smooth/blend, and surface-pick interactions as editor tools routed through scene/world commands.
- `domain/editor/editor_shell/src/composition/build_outliner_panel.rs`
  - replace ad hoc row buttons with the retained tree control where hierarchy semantics require expand/collapse and selection.
- `apps/runenwerk_editor/src/persistence/files.rs`
  - keep scene save/load going through migration, normalization, formation, apply, and retained change log persistence.

Validation:

- `cargo test -p runenwerk_editor --test scene_authoring_workflow_smoke`
- `cargo test -p runenwerk_editor -p editor_inspector -p editor_scene`
- GPU smoke manually when visual acceptance matters:
  `RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored`

### M3.5 - UI Definition Formation Framework

Purpose: introduce the first full UI definition framework before M3.6 and M4, including generic UI templates, RON-authored fixture loading, editor bindings, retained UI formation, and migration of stable editor chrome/common surface structure. M3.6 starts the visual editor and user-authored document lifecycle on top of this framework.

Owning design:

- `docs-site/src/content/docs/design/active/ui-definition-formation-foundation-design.md`

Rationale:

- M4 and later milestones will add many asset, workspace, menu, diagnostics, and preview surfaces; adding them through more shell/app hard-coding would multiply the current toolbar/layout friction.
- The framework belongs above the retained UI substrate, not inside editor-specific shell code.
- Authored UI definitions should be stable source/IR. Retained UI is the first formation target, not the permanent authored format.
- M3.6 should edit and manage definitions immediately after this slice; it should not be the first milestone to define the underlying UI definition contracts.
- The milestone should prove the model by migrating the toolbar, menus, workspace switcher, shell chrome, and common provider surface structure instead of leaving legacy builders as parallel sources of truth.

Implementation targets:

- `domain/ui/ui_definition/src/lib.rs`
  - add the engine-agnostic UI definition crate under the UI domain crate family.
- `domain/ui/ui_definition/src/identity.rs`
  - define stable authored UI ids that are distinct from `WidgetId`, focus/capture ids, `PanelInstanceId`, `ToolSurfaceInstanceId`, and ECS entity ids.
- `domain/ui/ui_definition/src/template.rs`, `src/node.rs`, `src/slot.rs`, `src/menu.rs`, `src/embed.rs`, and `src/availability.rs`
  - model authored UI templates, structural nodes, controls, menus, slots, repeaters, template refs, embed slots, and availability definitions without editor-specific command semantics, retained `UiNodeKind`, runtime `WidgetId`, ECS entity ids, or compiled update functions.
- `domain/ui/ui_definition/src/normalize.rs`
  - canonicalize ordering, resolve generic references, and report duplicate ids or malformed structures.
- `domain/ui/ui_definition/src/validate.rs`
  - validate ids, slot references, template refs, repeaters, embed slots, availability refs, and unsupported node combinations with structured diagnostics.
- `domain/ui/ui_definition/src/form.rs`
  - form validated definitions into retained UI products consumed by `ui_tree`, `ui_widgets`, and `ui_runtime` as the first target; formation emits route slots, embed slots, authored paths, and availability/diagnostic state, not command execution.
- `domain/editor/editor_definition/src/lib.rs`
  - add the editor-specific definition crate above `ui_definition` and below `editor_shell`.
- `domain/editor/editor_definition/src/toolbar.rs`, `src/menu.rs`, `src/workspace.rs`, `src/surface.rs`, `src/command.rs`, `src/availability.rs`, `src/binding.rs`, `src/validate.rs`, and `src/form_editor_ui.rs`
  - define editor menu ids, workspace catalog entries, command route ids, availability descriptors, toolbar bindings, shell chrome bindings, and common provider surface template bindings without importing active shell runtime state or app IO.
- `assets/editor/ui/*.ron`
  - add checked-in RON templates for toolbar, shell chrome, common provider surfaces, and editor bindings.
- `domain/editor/editor_shell/src/composition/build_toolbar.rs::build_toolbar`
  - migrate toolbar structure to formed definitions and remove `stable_name` string routing as the source of truth.
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_strip_from_frame`
  - template-form stable tab-stack/area chrome while keeping workspace mutation and panel identity ownership in `editor_shell`.
- `domain/editor/editor_shell/src/composition/build_inspector_panel.rs::build_inspector_panel`, `build_outliner_panel.rs::build_outliner_panel`, `build_entity_table_panel.rs::build_entity_table_panel`, `build_viewport_panel.rs::build_viewport_panel`, and `build_console_panel.rs::build_console_panel`
  - migrate repeated list/tree/table/form/chrome structure to templates while providers continue supplying data, embed payloads, and route results.
- `DOMAIN_MAP.md` and `docs-site/src/content/docs/domain/00-overview.md`
  - keep the ownership maps aligned with the active workspace crates.

Completeness status:

- Implemented: crate skeletons and metadata, generic source model modules, editor definition modules, checked-in fixtures, fixture parse/validation tests, retained formation, inert route/embed/path products, app-owned fixture loading, toolbar route-slot integration, toolbar popup item routing through editor binding data, normal shell chrome formation, and common provider surface fixture formation for console, inspector, outliner, entity-table, and viewport stable structure.
- Preserved outside `ui_definition`: provider DTO construction, surface-local routes, editor/app mutations, inspector edit commits, entity/outliner selection semantics, viewport options/status overlays, and render-product/embed payload ownership.
- M3.7 truth after this follow-up: explicit viewport instance identity is a runtime resource used by normal frame/input/provider binding paths, workspace persistence restores viewport instance ids and viewport runtime settings, viewport lifecycle sync runs before shell frame projection and persists viewport-owned settings, frame submit no longer owns live viewport singleton state, per-viewport render jobs and viewport-scoped product targets are published from viewport runtime systems, the viewport shader no longer carries multi-rectangle containment, camera/debug/root state has viewport-keyed command paths, and camera orbit/pan/zoom routes through viewport-local state. The surface workflow and product-maturity follow-up is complete: active provider-backed surfaces route through typed surface action/session/domain wrappers, entity-table query workflows and inspector enum controls are implemented, reusable surface controls are broader, descriptor-only field/atlas/volume/brickmap/history products are visible but unavailable, and provider behavior remains outside `ui_definition`. Remaining work is broader product implementation outside M3.7: integrated M4 active UI/editor catalog consumption, asset/catalog/import foundations, real field/asset/volume producers, and concrete temporal/history buffers.

Non-goals:

- no editor workspace/profile catalog implementation in `domain/ui/ui_definition`;
- no app provider registry, file IO, or runtime instantiation;
- no visual workspace layout designer, menu editor, theme editor, or shortcut editor inside M3.5; those are M3.6;
- no compiled-reactive or ECS-driven UI execution path;
- no retained-tree-specific authored source model that would force template rewrites for future accepted execution strategies;
- no authored ids derived from runtime/session ids or ECS entities.

Validation:

- when code lands: `cargo test -p ui_definition -p editor_definition -p ui_tree -p ui_runtime -p ui_widgets -p editor_shell`;
- after crate metadata changes: `cargo metadata --no-deps`;
- for docs-only planning updates: `python3 tools/docs/validate_docs.py`.

### M3.6 - UI Self-Authoring Workspace And Styling

Purpose: move the former final self-authoring/UI design work into the Now track so Runenwerk can style, inspect, validate, preview, and author UI definitions before later asset, procedural, gameplay, runtime, overlay, and in-game UI surfaces are built.

Status: complete as of 2026-05-06 for the self-authoring document lifecycle and retained preview path. Implemented: versioned UI definition migration wrapper, editor-owned workspace/profile/layout/theme/menu/shortcut/command-binding/panel/tool-surface schemas, editor-definition validation guards for runtime/session identity leakage, Editor Design workspace/profile, structural self-authoring tool-surface kinds, app-owned fixture document loading, retained UI preview, validation diagnostics, command diff/apply preview, retained provider control routes, draft UI hierarchy text edits, draft theme color edits, draft workspace-layout add-tab/split-root/close-tab edits, explicit apply/rollback shell commands, live activation for applied theme definitions, and provider surfaces for definition outliner, UI hierarchy, UI canvas, style inspector, bindings, dock/layout preview, theme editor, shortcut editor, menu editor, definition validation, and command diff. Follow-up maturity completed on 2026-05-08: applied workspace layout definitions now form and replace live `WorkspaceState`, UI templates and editor bindings feed live toolbar/shell-chrome formation, menu/shortcut/command-binding definitions activate into app-owned catalogs, panel/tool-surface registries activate with active-workspace compatibility checks, tool-surface registries feed future switch/create choices, reusable field/control polish covers the active editor surfaces, and export writes a versioned package instead of a bare definition document. Deferred beyond this maturity pass and moved into integrated M4: live menu consumption by toolbar/menu products, active shortcut dispatch through a command-binding resolver, full panel/tool-surface registry consumption by projected shell choices, and asset/import/field-product authoring surfaces. Still separate future tracks: payload ECS enum variants, native OS menu/shortcut integration, external marketplace workflows, compiled-reactive UI execution, and ECS-driven UI execution.

Owning design:

- `docs-site/src/content/docs/design/active/editor-self-authoring-and-final-ui-design.md`

Rationale:

- Later editor features will need many custom surfaces. Building those first in Rust and only adding self-authoring at the end would recreate the hard-coded toolbar/surface problem at a larger scale.
- The UI workspace should be the tool used to build later editor UI, debug overlays, and runtime/game UI templates, while behavior and command execution remain owned by their domains/app layers.
- This milestone starts visual authoring, but it remains bounded to definition documents, retained preview/formation, explicit command routing, and ratified changes.

Implementation targets:

- `domain/editor/editor_core/src/document.rs::DocumentKind`
  - activate the UI layout, workspace definition, theme, shortcut, menu, command binding, panel registry, tool-surface definition, and editor definition document kinds for the authoring workspace.
- `domain/editor/editor_definition/src/lib.rs`
  - extend the M3.5 editor binding layer into durable workspace profile, layout, menu, shortcut, command binding, panel registry, and tool-surface definition schemas.
- future `domain/ui/ui_definition/src/migration.rs`
  - add versioned authored UI definition migrations where needed for saved templates and style documents.
- `domain/editor/editor_shell/src/workspace/state.rs`
  - keep active workspace state session-owned and separate from authored workspace definitions while supporting preview/apply/rollback of formed workspace definitions.
- `apps/runenwerk_editor/src/shell/providers/editor_design_outliner.rs::EditorDesignOutlinerProvider`
  - show authored definition hierarchy, references, validation state, and dirty state.
- `apps/runenwerk_editor/src/shell/providers/ui_hierarchy.rs::UiHierarchyProvider`
  - show and edit the authored UI tree through ratified commands.
- `apps/runenwerk_editor/src/shell/providers/ui_canvas.rs::UiCanvasProvider`
  - preview formed UI definitions with selection handles, responsive preview sizes, and state fixtures.
- `apps/runenwerk_editor/src/shell/providers/style_inspector.rs::StyleInspectorProvider`
  - edit theme tokens, style references, layout constraints, spacing, typography, colors, and control state styling.
- `apps/runenwerk_editor/src/shell/providers/bindings_panel.rs::BindingsPanelProvider`
  - edit slot, action route, availability, collection, selection, and embed bindings without executing commands directly.
- `apps/runenwerk_editor/src/shell/providers/dock_layout_preview.rs::DockLayoutPreviewProvider`
  - preview formed workspace definitions before applying them.
- `apps/runenwerk_editor/src/shell/providers/theme_editor.rs::ThemeEditorProvider`
  - edit and preview theme definitions.
- `apps/runenwerk_editor/src/shell/providers/shortcut_editor.rs::ShortcutEditorProvider`
  - edit shortcuts with conflict diagnostics.
- `apps/runenwerk_editor/src/shell/providers/menu_editor.rs::MenuEditorProvider`
  - edit menu definitions and command bindings.
- `apps/runenwerk_editor/src/shell/providers/definition_validation.rs::DefinitionValidationProvider`
  - show blocking errors, warnings, migrations, unresolved references, capability issues, command route issues, and source paths.
- `apps/runenwerk_editor/src/shell/providers/command_diff.rs::CommandDiffProvider`
  - show the exact ratified definition change before apply/activation.
- `docs-site/src/content/docs/design/deferred/ui-model-multiple-execution-strategies-design.md`
  - keep the M0 UI execution-strategy decision visible during self-authoring. M3.6 must not introduce compiled-reactive or ECS-driven UI execution for the first time; any future promotion requires a separate active design or accepted ADR, guard tests, and roadmap update.

Validation:

- Implemented and covered: create, duplicate, rename, delete, import, export, validate, preview, apply, rollback, migrate, retained provider control routing, draft UI node text edits, draft theme color edits, draft workspace-layout add-tab/split-root/close-tab edits, live host theme activation for applied theme definitions, live workspace-state replacement for applied workspace layout definitions, versioned export packaging, and reusable field/control polish at the app/domain seam; validation blocks malformed definitions from becoming active; checked-in UI fixtures load as editable definition documents; retained previews form before apply; active runtime/session-only id vocabulary is rejected in editor authored ids; self-authoring follows the retained UI execution strategy closed in M0 and does not choose compiled-reactive or ECS-driven UI execution for the first time.
- Deferred beyond M3.6 and its 2026-05-08 maturity pass: live menu/shortcut/command-binding/panel/tool-surface consumption in the integrated M4 track; asset/procedural/gameplay/runtime overlay authoring after the asset foundations start; payload ECS enum variants when reflected payload fields exist; native OS menu/shortcut integration; external marketplace workflows; compiled-reactive UI execution; and ECS-driven UI execution.

### M3.7 - Viewport Expression Architecture Closeout

Purpose: replace the tactical split-viewport bridge with the final panel-owned viewport expression architecture before the editor depends on viewports for broader asset, field, material, runtime debug, and procedural previews.

Owning roadmap:

- `docs-site/src/content/docs/apps/runenwerk-editor/viewport-expression-implementation-roadmap.md`

Owning design:

- `docs-site/src/content/docs/design/active/workspace-viewport-expression-upgrade-design.md`
- `docs-site/src/content/docs/design/active/render-product-surface-foundation-bundle-design.md`

Status: complete as of 2026-05-08. Explicit viewport instance identity, persisted restore metadata and runtime settings, lifecycle-before-frame-submit sync, viewport-scoped product targets, per-viewport render jobs, shader containment cleanup, viewport-keyed camera/debug/root commands, viewport-local camera input, duplicate settings copy, and closed-surface cleanup are implemented. Further viewport producer breadth or polish belongs to later product milestones, not M3.7 migration work.

Implementation targets:

- `apps/runenwerk_editor/src/runtime/viewport/instance_registry.rs`
  - introduce explicit viewport instance allocation, duplication, restore, close, and `ToolSurfaceInstanceId -> ViewportId` mapping.
- `apps/runenwerk_editor/src/runtime/viewport/render_state.rs`
  - make bounds, dimensions, camera, presentation/debug mode, source version, throttling, and target status per-viewport state.
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs`
  - derive one `ViewportRenderJob` per visible viewport.
- `apps/runenwerk_editor/src/runtime/viewport/surface_set.rs`
  - replace shared static surface labels with viewport-scoped concrete product targets.
- `apps/runenwerk_editor/src/runtime/viewport/producer_scene.rs`
  - render scene color, picking ids, overlay, and later depth/debug products per viewport job.
- `assets/shaders/editor_viewport_scene_product.wgsl`
  - remove `viewport_b`, `viewport_c`, and `viewport_d`; render one viewport-local target per job.
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::submit_editor_frame_system`
  - stop owning viewport lifecycle and render-product synchronization.
- `apps/runenwerk_editor/tests/viewport_architecture_guards.rs`
  - reject shared viewport products, multi-rect shader uniforms, frame-submit viewport identity seeding, and first-observed viewport fallbacks.

Exit criteria:

- every visible viewport has explicit instance identity and independent scene color, picking, and overlay product targets;
- viewport camera, product selection, overlay selection, diagnostics visibility, bounds, and resize state are independent per viewport;
- UI embeds reference `(ViewportId, ViewportSurfacePresentationSlot)` and never sample a shared global viewport product accidentally;
- picking and transform interaction use viewport-local coordinates and viewport-local camera state;
- closing, splitting, duplicating, hiding, resizing, and restoring viewport surfaces preserve explicit lifecycle semantics;
- no normal runtime path depends on `viewport_b`-style shader containment or shared static viewport product labels.

Validation:

- `cargo test -p runenwerk_editor viewport`
- `cargo test -p runenwerk_editor --test startup_render_smoke`
- `cargo test -p runenwerk_editor --test viewport_architecture_guards`
- `RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`

### M4 - Integrated UI/Editor Live Replacement And Asset Pipeline Foundation

Purpose: finish the active editor-definition consumption path and make project
content explicit and field-world-first instead of loose files discovered by
unrelated runtime systems. Asset browser, import inspector, field-product
viewer, and SDF brush browser surfaces should land through the same active
panel/tool-surface definition seams that the Editor Design workspace can author.

Implementation targets:

- `apps/runenwerk_editor/src/shell/applied_editor_definition/catalogs.rs`
  - validate menu, shortcut, command-binding, panel-registry, and tool-surface-registry replacement before mutation; invalid activation must leave the previous active catalog unchanged.
- `apps/runenwerk_editor/src/shell/toolbar_adapter.rs` and `domain/editor/editor_shell/src/composition/toolbar_definition.rs`
  - consume active menu and command-binding catalogs when forming toolbar/menu view models, while preserving fallback checked-in fixtures.
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs::dispatch_shortcuts`
  - replace hard-coded editor shortcuts with an active shortcut resolver that maps shortcut definitions through active command bindings into existing app/domain commands only.
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs`
  - project active panel/tool-surface registry definitions into editor type switching, new-tab creation, and surface menu choices without mutating the existing workspace unless a workspace layout definition is applied.
- `apps/runenwerk_editor/src/shell/providers/mod.rs`
  - keep future asset/import/field-product surfaces registered through app-owned providers while their availability is surfaced through active panel/tool-surface catalogs.
- `domain/asset/src/lib.rs`
  - add a new engine-agnostic asset domain crate.
- `domain/asset/src/id.rs`
  - define typed ids for assets, sources, imported artifacts, import jobs, and asset revisions.
- `domain/asset/src/kind.rs`
  - define asset kinds for scene, prefab, SDF graph, SDF brush/layer, field-world definition, world edit log, field material channels, formed field product, `world_sdf` chunk/page artifact, clipmap/brickmap product, material graph, material, procedural material, procedural texture, Texture2D, Texture3D/volume texture, gameplay graph, gameplay rule/trigger, gameplay ability, gameplay quest, gameplay ATR IR product, gameplay ECS lowering product, particle graph, particle emitter, physics config, animation clip, animation graph, procgen graph, UI layout, UI definition, graph, script, shader, theme, menu, shortcut, workspace definition, editor definition, and foreign mesh/reference source. Mesh/glTF kinds are interop/reference kinds, not the primary world substrate.
- `domain/asset/src/source.rs`
  - model source files, source hashes, source provenance, external source roots, authored SDF/field documents, and foreign source roots.
- `domain/asset/src/artifact.rs`
  - model imported artifacts, formed field product references, `world_sdf` payload references, gameplay ATR IR products, gameplay ECS query/event/schedule lowering products, source maps, cache keys, generated files, runtime package refs, and artifact validity.
- `domain/asset/src/dependency_graph.rs`
  - model dependencies and reverse dependencies for import invalidation and hot reload.
- `domain/asset/src/import_settings.rs`
  - model import settings for SDF graph/source formation, field-world layers, `world_sdf` brick/page/chunk product formation, material channel policy, procedural material/texturing, PBR parameter validation, triplanar mapping, Texture2D, Texture3D/volume texture products, gameplay graph compiler profile, ATR IR validation, ECS query/event/schedule lowering, SDF physics relation readiness, authority/network policy, particles, physics configs, animation clips/graphs, procgen graphs, shaders, UI definitions, scripts, authored RON documents, and Blender/glTF as foreign-source compatibility.
- `domain/asset/src/import_plan.rs`
  - build deterministic import plans without executing external tools.
- `domain/asset/src/diagnostics.rs`
  - define stable asset pipeline diagnostic codes.
- `domain/asset/src/ratification.rs`
  - ratify imported, migrated, generated, and externally supplied asset candidates.
- `domain/world_sdf/src/product.rs`
  - define engine-agnostic descriptors for formed SDF chunk/page/brick products, product scope, scale band, source lineage, freshness, consumer class, and retention/rebuild policy.
- `domain/world_sdf/src/ratification.rs`
  - ratify formed `world_sdf` payload candidates before they become catalog-visible artifacts.
- `domain/world_ops/src/build_graph.rs` and `domain/world_ops/src/build_queue.rs`
  - connect asset/source changes to field-product invalidation and interactive/background rebuild plans without owning artifact IO.
- `domain/editor/editor_persistence/src/project_file.rs`
  - add `ProjectFileV2` with asset catalog roots, startup document, source roots, field-product cache root, workspace profile defaults, and migration from `ProjectFileV1`.
- `apps/runenwerk_editor/src/persistence/files.rs::read_project_file`
  - load and migrate project files through the asset catalog instead of scene-only entries.
- `DOMAIN_MAP.md` and `CRATES.md`
  - add the new asset domain ownership when the crate lands.

Validation:

- `cargo test -p asset`
- `cargo test -p editor_persistence`
- asset kind and import-settings tests cover gameplay graph, gameplay rule/trigger, ability, quest, ATR IR product, and ECS lowering product entries;
- `cargo metadata --no-deps` after adding the workspace member.

### M5 - Asset Import, Field Product Formation, Preview, And Hot Reload Execution

Purpose: connect the asset domain to real editor and runtime workflows.

Implementation targets:

- `apps/runenwerk_editor/src/asset_pipeline/import_jobs.rs::run_import_job`
  - execute domain import plans with app-owned IO and external process policy.
- `apps/runenwerk_editor/src/asset_pipeline/field_product_jobs.rs::run_field_product_job`
  - execute field-product formation plans for SDF graphs, field-world layers, material channels, and `world_sdf` chunk/page/brick artifacts through app/engine-owned runtime policy.
- `apps/runenwerk_editor/src/asset_pipeline/catalog_runtime.rs`
  - maintain the open project's asset catalog, import job queue, field-product build queue, diagnostics, dirty asset set, dirty field-product set, and preview state.
- `domain/world_ops/src/dirty.rs`, `domain/world_ops/src/region_invalidation.rs`, and `domain/world_ops/src/build_queue.rs`
  - drive explicit invalidation and rebuild scheduling for affected regions, scale bands, and consumer products.
- `domain/world_sdf/src/storage.rs::SdfChunkStore`
  - receive ratified chunk/page/brick payload artifacts through runtime/app integration; do not let the editor write private storage internals directly.
- `tools/assets/blender_export.py::main`
  - support foreign-source/reference import by exporting configured `.blend` sources to deterministic `.glb` artifacts using `assets/editor/config.ron` as host configuration, or emit a missing-tool diagnostic that preserves the prior valid artifact. This is not the canonical field-world pipeline.
- `apps/runenwerk_editor/src/shell/providers/asset_browser.rs::AssetBrowserProvider`
  - add an asset browser surface backed by the asset catalog.
- `apps/runenwerk_editor/src/shell/providers/import_inspector.rs::ImportInspectorProvider`
  - expose import settings, import diagnostics, source/artifact paths, dependencies, and reimport controls.
- `apps/runenwerk_editor/src/shell/providers/field_product_viewer.rs::FieldProductViewerProvider`
  - preview SDF fields, material channels, chunk/page/brick payloads, clipmap/brickmap products, provenance, freshness, and invalidation diagnostics.
- `apps/runenwerk_editor/src/shell/providers/sdf_brush_browser.rs::SdfBrushBrowserProvider`
  - browse and preview SDF brushes/layers as field-world authoring assets.
- `apps/runenwerk_editor/src/shell/providers/texture_viewer.rs::TextureViewerProvider`
  - inspect Texture2D, Texture3D/volume slices, channels, mips, color space, compression, and generated texture provenance.
- `engine/src/plugins/scene/manifest/catalog.rs::load_scene_manifest_descriptors`
  - migrate from loose scene manifest scanning to an asset-catalog-backed scene/template query, with the current loose RON path retained only as an explicit compatibility adapter during migration.
- `engine/src/plugins/render/resource/import.rs`
  - keep render resource imports as renderer-facing contracts and map asset artifact ids to render resource ids at engine integration edges.
- `engine/src/plugins/render/shader/hot_reload.rs::poll_shader_hot_reload`
  - integrate shader asset revisions and diagnostics with the asset reload stream.
- `engine/src/plugins/shared/reload.rs`
  - reuse reload status payloads for asset, field product, `world_sdf` payload, shader, scene, prefab, UI, graph, and script hot reload status.

Validation:

- import or author one SDF/field source and form a ratified field product artifact;
- update an SDF/field source and prove region/product invalidation flows through `world_ops` into rebuild scheduling;
- load or preview a formed `world_sdf` chunk/page/brick payload without treating mesh/glTF as the source of world truth;
- import one `.blend` source into a `.glb` foreign-reference artifact from `assets/models/`, or prove the missing-tool diagnostic path preserves the prior valid artifact;
- load a scene through the project asset catalog;
- reimport a changed source and show diagnostics in the editor;
- prove failed imports leave prior valid artifacts intact;
- `cargo test -p runenwerk_editor -p engine -p asset`.

### M6 - Procedural Authoring Workspaces Beyond Scene Editing

Purpose: make the editor a multi-document procedural authoring environment.

Detailed feature slices, milestone gates, and remaining decisions for material graphs, procedural texturing, Texture3D, procgen, particles, physics, animation, and world processes are owned by `docs-site/src/content/docs/design/active/editor-procedural-content-and-simulation-workflow-plan.md`. Gameplay graph ATR IR and ECS lowering are owned by `docs-site/src/content/docs/design/active/gameplay-graph-atr-ir-and-ecs-lowering-design.md`.

M6 closes by sub-milestone, not as one broad bucket. Each sub-milestone must satisfy its owning design's first-slice exit criteria, source lineage requirements, diagnostics, failed-product preservation, and provider boundary tests before it can be marked complete.

Sub-milestones:

- M6.0 Shared workspace substrate: workspace profiles, scoped mode compatibility, graph canvas hosting, diagnostics, provider routing, runtime debug surfaces, and document tab coverage.
- M6.1 Material and texture first slice: `domain/material_graph`, `domain/texture`, PBR parameters, procedural nodes, triplanar mapping, Texture2D, Texture3D, material previews, and formed material products.
- M6.2 Procgen first slice: deterministic generator documents, seed/scope contracts, bounded preview, world-operation lowering, bake/rollback, and changed-region diagnostics.
- M6.3 Gameplay graph first slice: prerequisite gameplay event/action/state/quest contracts, Action/Trigger/Rule IR, compiler passes, ECS query/event/schedule lowering, SDF physics `HIT` relation readiness, authority diagnostics, and source maps.
- M6.4 Particles first slice: deterministic emitter documents, SDF/field spawn and collision coupling, preview products, count/bounds diagnostics, and backend-neutral formed products.
- M6.5 SDF physics first slice: collision product descriptors, rigid/kinematic/character body contracts, physics material links, field-query readiness, and debug surfaces.
- M6.6 Animation first slice: clips, curves, timeline, state/blend graphs, procedural motion hooks, source maps, and preview diagnostics.
- M6.7 World-process first slice: bounded material-transport previews, timescale/solver-budget contracts, bake/commit to governed `world_ops`, rollback, and product freshness diagnostics.

Implementation targets:

- `domain/editor/editor_shell/src/workspace/profile.rs::default_workspace_profile_registry`
  - add `Field World`, `SDF Modeling`, `Materials`, `Textures`, `Procedural Generation`, `Gameplay`, `Particles`, `Physics`, `Animation`, `Layout`, `UI`, `Graphs`, `Scripting`, `Simulation`, `Debug`, and `Editor Design` workspace profiles.
- `domain/editor/editor_core/src/session.rs`
  - validate mode activation against `(workspace_profile, document_kind)`.
- future `domain/material_graph/src/lib.rs`
  - own material graph semantics, PBR parameter contracts, procedural texturing nodes, triplanar mapping semantics, ratification, lowering, and formed material products.
- future `domain/texture/src/lib.rs`
  - own Texture2D, Texture3D/volume texture descriptors, color space, sampler, compression, generated texture cache metadata, and texture diagnostics.
- future `domain/procgen/src/lib.rs`
  - own procedural generation documents, seed contracts, generator graphs, and lowering to bounded world operation windows.
- future `domain/gameplay/events/src/lib.rs`
  - own gameplay event ids, payload schemas, channel descriptors, authority class, and source-map subjects used by gameplay graph lowering.
- future `domain/gameplay/actions/src/lib.rs`
  - own action request, action plan, action result, and effect vocabulary before gameplay graph transforms can request actions.
- future `domain/gameplay/state/src/lib.rs`
  - own state machine, state membership, transition, and condition contracts before gameplay graph state transforms lower to runtime products.
- future `domain/gameplay/quests/src/lib.rs`
  - own quest, objective, progress, and completion contracts before gameplay graph quest transforms lower to runtime products.
- future `domain/gameplay_graph/src/lib.rs`
  - own gameplay graph semantics, Action/Trigger/Rule IR, compiler passes, SDF physics relation binding, and formed ECS query/event/schedule/runtime products while depending on narrower gameplay event/action/state/quest contracts for their semantics.
- future `domain/particles/src/lib.rs`
  - own emitter definitions, particle graph semantics, SDF/field coupling contracts, and formed particle products.
- future `domain/physics/src/lib.rs`
  - own body, collider, constraint, trigger, physics material, collision product, and readiness contracts.
- future `domain/animation/src/lib.rs`
  - own clips, curves, state machines, blend trees, procedural motion, root motion, animation events, and binding diagnostics.
- `apps/runenwerk_editor/src/shell/providers/ui_hierarchy.rs::UiHierarchyProvider`
  - show and edit UI document hierarchy.
- `apps/runenwerk_editor/src/shell/providers/ui_canvas.rs::UiCanvasProvider`
  - preview retained UI layout documents through the existing UI substrate.
- `apps/runenwerk_editor/src/shell/providers/graph_canvas.rs::GraphCanvasProvider`
  - host graph documents using `domain/graph`.
- `apps/runenwerk_editor/src/shell/providers/field_layer_stack.rs::FieldLayerStackProvider`
  - show and edit authored field-world layers, SDF operation ordering, and material channel bindings through commands.
- `apps/runenwerk_editor/src/shell/providers/sdf_graph_canvas.rs::SdfGraphCanvasProvider`
  - host SDF graph documents while using `domain/sdf` for field semantics and `domain/graph` only for graph substrate behavior.
- `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs::MaterialGraphCanvasProvider`
  - edit material graphs without making graph canvas state material truth.
- `apps/runenwerk_editor/src/shell/providers/material_inspector.rs::MaterialInspectorProvider`
  - edit PBR parameters, procedural node settings, SDF/field inputs, and material channel bindings.
- `apps/runenwerk_editor/src/shell/providers/material_preview.rs::MaterialPreviewProvider`
  - preview material products on SDF primitives, field products, and reference meshes.
- `apps/runenwerk_editor/src/shell/providers/procgen_graph_canvas.rs::ProcgenGraphCanvasProvider`
  - edit procedural generation graphs that lower to deterministic world operation windows.
- `apps/runenwerk_editor/src/shell/providers/procgen_preview.rs::ProcgenPreviewProvider`
  - preview, bake, and rollback bounded generated worlds.
- `apps/runenwerk_editor/src/shell/providers/gameplay_graph_canvas.rs::GameplayGraphCanvasProvider`
  - edit constrained gameplay graphs without making graph canvas state runtime authority.
- `apps/runenwerk_editor/src/shell/providers/gameplay_compiler_diagnostics.rs::GameplayCompilerDiagnosticsProvider`
  - inspect ATR IR, ECS query/event/schedule lowering, SDF physics relation readiness, source maps, and authority diagnostics.
- `apps/runenwerk_editor/src/shell/providers/particle_graph_canvas.rs::ParticleGraphCanvasProvider`
  - edit emitters and particle graphs.
- `apps/runenwerk_editor/src/shell/providers/particle_preview.rs::ParticlePreviewProvider`
  - preview particles, field collision, counts, bounds, and simulation diagnostics.
- `apps/runenwerk_editor/src/shell/providers/physics_authoring.rs::PhysicsAuthoringProvider`
  - edit bodies, colliders, materials, layers, masks, constraints, and triggers.
- `apps/runenwerk_editor/src/shell/providers/physics_debug.rs::PhysicsDebugProvider`
  - inspect contacts, sweeps, activation, constraints, and missing `world_sdf` readiness.
- `apps/runenwerk_editor/src/shell/providers/timeline.rs::TimelineProvider`
  - edit clips, events, and playback ranges.
- `apps/runenwerk_editor/src/shell/providers/curve_editor.rs::CurveEditorProvider`
  - edit typed animation/procedural curves.
- `apps/runenwerk_editor/src/shell/providers/animation_graph_canvas.rs::AnimationGraphCanvasProvider`
  - edit state machines, blend trees, and procedural motion graphs.
- `apps/runenwerk_editor/src/shell/providers/simulation_preview.rs::SimulationPreviewProvider`
  - preview and bake material transport, erosion, snow, water, sediment, and other world-process effects.
- `apps/runenwerk_editor/src/shell/providers/script_editor.rs::ScriptEditorProvider`
  - edit script assets while keeping scripting language-neutral at the domain boundary.
- `apps/runenwerk_editor/src/shell/providers/diagnostics.rs::DiagnosticsProvider`
  - show project, import, runtime, and validation diagnostics.
- `apps/runenwerk_editor/src/shell/providers/runtime_debug.rs::RuntimeDebugProvider`
  - inspect ECS/runtime state without making authored documents ECS entities.

Validation:

- M6.0 through M6.7 each have explicit closeout notes or checklist entries before M6 is marked complete;
- one document tab per implemented document kind opens, saves, closes, reopens, and reports dirty state correctly;
- procedural graph lowering never depends on editor graph canvas state;
- gameplay graph lowering never depends on editor graph canvas state and forms ECS query/event/schedule products with source maps;
- generated products keep source lineage, seed, version, and diagnostics;
- unsupported workspace/document/surface combinations fail closed with provider diagnostics;
- no provider bypasses command or ratification boundaries.

### M7 - Runtime Preview, Play/Simulate, Scripting, And Data Hot Reload

Purpose: make authored content executable inside clear runtime boundaries.

Implementation targets:

- `apps/runenwerk_editor/src/editor_app/state.rs`
  - separate edit, preview, simulate, and play session state.
- `apps/runenwerk_editor/src/runtime/app.rs`
  - compose runtime preview/play resources without making the generic engine editor-shaped.
- `domain/editor/editor_core/src/session.rs`
  - validate play/simulate mode transitions through scoped mode contracts.
- future `domain/script_runtime/src/lib.rs`
  - define language-neutral script asset/runtime command contracts.
- future `adapters/script_rhai/src/lib.rs`
  - implement the first Rhai adapter without leaking Rhai types into domain contracts.
- future `engine/src/plugins/gameplay_graph/`
  - instantiate formed gameplay graph products into ECS query descriptors, event subscriptions, fixed executor descriptors, schedule edges, runtime registries, source maps, and authority/network metadata without interpreting authored graph nodes.
- future `domain/gameplay_graph/src/lowering/sdf_physics.rs`
  - bind `RELATE HIT` and other SDF/field relations to `domain/world_sdf`, future `domain/physics`, or engine collision/query products through readiness diagnostics.
- `apps/runenwerk_editor/src/asset_pipeline/catalog_runtime.rs`
  - route safe data hot reload into SDF graph, field-world, field product, `world_sdf` payload, material graph, generated texture, Texture3D, particle, physics config, animation, procgen, gameplay graph formed product, scene, prefab, UI, graph, material, shader, and script refresh workflows.
- `docs-site/src/content/docs/design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`
  - keep the data hot reload versus Rust rebuild/restart boundary aligned with implementation.

Validation:

- data/assets reload live where safe;
- procedural material, texture, particle, physics, animation, and procgen products fail closed when hot reload cannot preserve runtime safety;
- gameplay graph products fail closed when ECS query/event/schedule lowering, SDF physics readiness, source maps, or authority metadata cannot be preserved;
- structural Rust changes require registry refresh and play-session restart;
- play/simulate does not mutate authored documents without explicit commands;
- runtime execution consumes formed gameplay graph products, not live authored graph traversal;
- script actions request domain/runtime commands instead of directly mutating ECS internals.

### M8 - UI Authoring Packaging, Extensibility, And Externalization

Purpose: harden the M3.6 self-authoring system for specialized editor packaging, external definition exchange, and long-lived project migration after the main asset/procedural/runtime feature tracks exist.

Implementation targets:

- `domain/editor/editor_definition/src/package.rs`
  - define exportable editor-definition package manifests, dependency references, compatibility ranges, and migration metadata.
- `domain/ui/ui_definition/src/package.rs`
  - define exportable UI-template/theme package manifests without app IO or runtime execution.
- `apps/runenwerk_editor/src/persistence/editor_definition_package.rs`
  - import/export packages through app-owned file IO and project policy.
- `apps/runenwerk_editor/src/shell/providers/editor_package_publisher.rs::EditorPackagePublisherProvider`
  - validate and publish specialized editor packages from authored definitions.
- `apps/runenwerk_editor/src/shell/providers/definition_migration_report.rs::DefinitionMigrationReportProvider`
  - show cross-project migration reports and unresolved compatibility issues.
- `docs-site/src/content/docs/design/deferred/ui-model-multiple-execution-strategies-design.md`
  - keep the M0 UI execution-strategy decision visible. M8 must not introduce compiled-reactive or ECS-driven UI execution for the first time; any future promotion requires a separate active design or accepted ADR, guard tests, and roadmap update.

Validation:

- package import/export preserves authored ids, source maps, diagnostics, migrations, and compatibility metadata;
- published specialized editor packages cannot activate invalid UI/editor definitions;
- package publishing follows the retained UI execution strategy closed in M0 and does not choose compiled-reactive or ECS-driven UI execution for the first time;
- active runtime/session-only ids are never exported as authored ids.

### M9 - Hardening, Release Readiness, And Full Gate

Purpose: close the editor as a product-quality tool.

Implementation targets:

- `docs-site/src/content/docs/apps/runenwerk-editor/current-architecture.md`
  - update current architecture after each closed milestone.
- `docs-site/src/content/docs/workspace/roadmap-index.md`
  - keep source-of-truth links current.
- `docs-site/src/content/docs/workspace/repo-execution-priority-checklist.md`
  - keep operational Now/Next status aligned.
- `apps/runenwerk_editor/tests/`
  - add smoke tests for project open/save, SDF/field asset import, field-product formation, procedural material/texturing, Texture3D/volume inspection, procgen preview/bake, gameplay graph ATR IR lowering to ECS query/event/schedule products, SDF physics `HIT` relation diagnostics, particles, physics authoring/debug, animation preview, world-process preview/bake, document tabs, docking, scene authoring, UI authoring, runtime preview, and self-authoring.
- `domain/editor/editor_shell/src/tests.rs`
  - keep projection golden tests authoritative for workspace structure and routing.
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs`
  - keep UI frame snapshot expectations stable where render-data output is contractual.

Full validation:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh` for milestone closeout and final release-readiness validation.

## Completion Criteria

The roadmap is complete when a user can:

1. create or open a project;
2. import source assets and see catalog diagnostics;
3. create and edit SDF scenes, field-world layers, prefabs, UI layouts, graphs, scripts, materials, procedural textures, Texture3D assets, gameplay graphs, gameplay rules/triggers/abilities/quests, particles, physics configs, animation clips/graphs, procedural generators, menus, themes, shortcuts, and workspace/editor definitions;
4. use document tabs, workspace switching, scoped modes, docking, editor type switching, area tabs, plus/new-tab controls, and saved layouts;
5. author a 3D SDF scene with create/delete/duplicate, hierarchy operations, SDF primitive/brush actions, translate/rotate/scale, snapping, inspector/component editing, undo/redo, and persistence;
6. preview imported assets, SDF fields, `world_sdf` chunk/page/brick products, procedural PBR materials, triplanar/Texture3D products, gameplay graph lowering products, ECS query/event/schedule source maps, particles, physics debug products, animation products, procedural generation outputs, and authored documents in appropriate tool surfaces;
7. run play/simulate/preview with safe data hot reload and clear restart boundaries for structural Rust changes;
8. validate, migrate, import, export, and publish authored definitions, including gameplay graph ATR IR products that lower into deterministic runtime products;
9. recover from failed imports, invalid documents, bad definitions, and stale runtime previews without corrupting project state;
10. pass the full repository validation gate.

## Rule

Do not add new editor/UI feature paths by bypassing these ownership seams. If a needed feature does not fit, update the owning design first, then update this roadmap, then implement through the appropriate domain, engine, app, or tool boundary.
