---
title: Game Runtime, Editor, ECS, Scripting, and Hot Reload Design
description: Current-vs-target architecture map for runtime/editor/ECS boundaries, linked to a deferred preserved target draft.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-29
related_designs:
  - ./editor-ui-workspace-tool-surface-architecture.md
  - ./workspace-field-world-and-simulation-platform-design.md
  - ../deferred/engine-game-runtime-editor-ecs-scripting-hot-reload-preserved-target-draft.md
related_roadmaps:
  - ../../apps/runenwerk-editor/roadmap.md
---

# Game Runtime, Editor, ECS, Scripting, and Hot Reload Design

## Read This First

This page is the current-state grounded architecture/gap document.

The prior long-form target draft is preserved in a separate deferred document so detail is not lost while this page remains implementation-grounded.

## Purpose and Scope

This design defines boundaries between code, editor-authored content, runtime execution, ECS world state, scripting, instantiation, and reload workflows in Runenwerk.

This document is architecture-focused. It does **not** commit a final implementation sequence; sequencing belongs in roadmap docs.

## Current Repository Anchors (Implemented Today)

- Workspace ownership/layering is defined by current members in [`Cargo.toml`](/Users/joshua/Projekte/gamedev/Runenwerk/Cargo.toml) and root domain ownership in [`DOMAIN_MAP.md`](/Users/joshua/Projekte/gamedev/Runenwerk/DOMAIN_MAP.md).
- Runtime app composition currently flows through `engine::App` (`add_plugin`, `add_systems`, `add_scene`, `add_scene_template`, `run`) in [`engine/src/app/domain/app.rs`](/Users/joshua/Projekte/gamedev/Runenwerk/engine/src/app/domain/app.rs).
- Runnable editor composition is in [`apps/runenwerk_editor/src/runtime/app.rs`](/Users/joshua/Projekte/gamedev/Runenwerk/apps/runenwerk_editor/src/runtime/app.rs).
- Editor runtime already exposes explicit authored/instantiated/simulated/session reality views in [`apps/runenwerk_editor/src/editor_runtime/runtime.rs`](/Users/joshua/Projekte/gamedev/Runenwerk/apps/runenwerk_editor/src/editor_runtime/runtime.rs).
- Scene persistence format today is `SceneFileV2` RON records in [`domain/editor/editor_persistence/src/scene_file.rs`](/Users/joshua/Projekte/gamedev/Runenwerk/domain/editor/editor_persistence/src/scene_file.rs), not `.rwscene/.rwprefab/.rwinput` assets.
- Scene manifest discovery currently reads `.ron` files from `assets/scenes` and `game/assets/scenes` in [`engine/src/plugins/scene/manifest/catalog.rs`](/Users/joshua/Projekte/gamedev/Runenwerk/engine/src/plugins/scene/manifest/catalog.rs).
- Current app-level editor MVP scope intentionally excludes prefab/material/scripting tooling for the first milestone, per [`docs-site/src/content/docs/apps/runenwerk-editor/mvp/first-3d-editor-mvp.md`](/Users/joshua/Projekte/gamedev/Runenwerk/docs-site/src/content/docs/apps/runenwerk-editor/mvp/first-3d-editor-mvp.md).

## Current-to-Target Gap Matrix

| Capability Area | Current State | Target Direction | Owning Areas |
|---|---|---|---|
| Runtime/editor boundary | Single-process runtime/editor composition is active. | Optional external runtime process boundary later. | `engine`, `apps/runenwerk_editor` |
| Scene authoring/persistence | Scene document + migration/normalization/formation exists. | Broader authored asset families (prefab/input/ability/etc.) with explicit contracts. | `domain/editor/editor_persistence`, future domain crates |
| ECS state ownership | Live runtime world + reflected component registration are active. | Keep ECS as live state only; expand contract catalog without ownership leakage. | `domain/ecs`, `apps/runenwerk_editor` |
| Plugin/type registration | Concrete registration points exist in runtime/editor app code. | Stronger registry-driven discoverability across more gameplay domains. | `engine`, `apps/runenwerk_editor` |
| Hot reload model | Data-driven scene/template reload paths exist; structural code changes still restart-oriented. | Explicit refresh/restart boundaries with clearer UX and diagnostics. | `engine`, `apps/runenwerk_editor` |
| Gameplay scripting boundary | No script runtime crate is implemented in workspace yet. | Language-neutral script contract boundary + adapter implementation later; Rhai is the first concrete adapter candidate. | future domain/runtime/adapter crates |
| Runtime UI attachment binding | Overlay UI is currently scene-template driven (`overlay_ui` + `ui_template`), not entity-attachment driven. | World-space/screen-projected UI attachment binding is deferred post-MVP and should be added only through explicit authored binding contracts and runtime formation seams. | `engine` scene runtime + future domain/runtime contracts |
| Gameplay content breadth | Current editor MVP is 3D graybox scene authoring focused. | Post-MVP expansion for richer authored domains when contracts exist. | `apps/runenwerk_editor`, future domain crates |
| Architectural reality model | Doctrine exists in guideline docs and editor runtime reality views. | Deeper consistency across authored/normalized/formed/instantiated/simulated pipelines. | guidelines + editor/runtime domains |

## Canonical Boundary Rules for This Topic

- ECS stores **live simulated state**, not arbitrary authored source documents.
- Authored data should pass explicit migration/normalization/formation boundaries before runtime instantiation.
- Runtime may mutate entity/component instances, but mutation authority must remain explicit and domain-owned.
- Editor composition must stay capability-driven; the editor should not invent runtime semantics absent engine/domain contracts.
- Any future scripting integration must preserve Rust/domain ownership of correctness and invariants.
- Scripting boundaries remain language-neutral even when Rhai is used as the first adapter; adapter-specific types must not leak into domain/runtime contracts.
- World-space/screen-projected UI attachment flows are post-MVP and should not be treated as implied by current scene-overlay UI template support.

## Deferred Detailed Draft

The prior long-form target draft has been moved (verbatim) to keep this active document concise and implementation-grounded:

- [`../deferred/engine-game-runtime-editor-ecs-scripting-hot-reload-preserved-target-draft.md`](../deferred/engine-game-runtime-editor-ecs-scripting-hot-reload-preserved-target-draft.md)

Use this active doc for current boundaries and gap analysis; use the deferred preserved draft for deeper aspirational details.
