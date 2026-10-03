---
title: Runenwerk Workspace Package Census
description: Exact workspace-package size and disposition census for the repository-wide architecture audit.
status: active
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-10-03
publication: reference
pagefind: false
---

# Runenwerk Workspace Package Census

Evidence baseline:

```text
34ad02d9b8fac925936ac0d5971abe4d293f1749
```

This appendix records all root-workspace members. Layer/disposition is architectural review metadata; it is not a package-merge plan.

| Workspace member | Layer | Tracked files | Rust files | Production Rust files | Rust bytes | Disposition |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `domain/geometry` | domain | 22 | 21 | 11 | 40923 | keep unless owner-specific evidence changes |
| `domain/asset` | domain | 14 | 13 | 13 | 102908 | keep unless owner-specific evidence changes |
| `domain/product` | domain | 13 | 12 | 12 | 83186 | keep unless owner-specific evidence changes |
| `domain/world_ops` | domain | 13 | 12 | 12 | 30485 | keep unless owner-specific evidence changes |
| `domain/world_sdf` | domain | 11 | 10 | 9 | 106062 | keep unless owner-specific evidence changes |
| `domain/physics` | domain | 3 | 2 | 1 | 30557 | keep unless owner-specific evidence changes |
| `domain/scene` | domain | 5 | 4 | 4 | 10918 | keep unless owner-specific evidence changes |
| `domain/graph` | domain | 6 | 5 | 5 | 20736 | keep unless owner-specific evidence changes |
| `domain/texture` | domain | 7 | 6 | 6 | 25284 | keep unless owner-specific evidence changes |
| `domain/material_graph` | domain | 11 | 10 | 10 | 140302 | keep unless owner-specific evidence changes |
| `domain/procgen` | domain | 14 | 13 | 13 | 105719 | keep unless owner-specific evidence changes |
| `domain/drawing` | domain | 45 | 44 | 42 | 171509 | keep crate; tile formation decomposition candidate |
| `domain/simulation` | domain | 6 | 5 | 5 | 4339 | keep unless owner-specific evidence changes |
| `domain/replay` | domain | 9 | 8 | 7 | 9803 | keep unless owner-specific evidence changes |
| `domain/ui/ui_math` | domain-ui-separate-owner | 8 | 7 | 7 | 5520 | defer to RunenUI/cutover authority |
| `domain/ui/ui_input` | domain-ui-separate-owner | 16 | 15 | 12 | 46564 | defer to RunenUI/cutover authority |
| `domain/ui/ui_layout` | domain-ui-separate-owner | 11 | 10 | 9 | 24318 | defer to RunenUI/cutover authority |
| `domain/ui/ui_text` | domain-ui-separate-owner | 14 | 13 | 12 | 67990 | defer to RunenUI/cutover authority |
| `domain/ui/ui_theme` | domain-ui-separate-owner | 21 | 20 | 13 | 52579 | defer to RunenUI/cutover authority |
| `domain/ui/ui_render_data` | domain-ui-separate-owner | 28 | 27 | 25 | 58468 | defer to RunenUI/cutover authority |
| `domain/ui/ui_composition` | domain-ui-separate-owner | 51 | 50 | 40 | 261154 | defer to RunenUI/cutover authority |
| `domain/ui/ui_adaptive_composition` | domain-ui-separate-owner | 15 | 14 | 13 | 51817 | defer to RunenUI/cutover authority |
| `domain/ui/ui_surface` | domain-ui-separate-owner | 16 | 15 | 15 | 54666 | defer to RunenUI/cutover authority |
| `domain/ui/ui_graph_editor` | domain-ui-separate-owner | 2 | 1 | 1 | 32083 | defer to RunenUI/cutover authority |
| `domain/ui/ui_definition` | domain-ui-separate-owner | 67 | 66 | 60 | 394610 | defer to RunenUI/cutover authority |
| `domain/ui/ui_schema` | domain-ui-separate-owner | 4 | 3 | 3 | 28341 | defer to RunenUI/cutover authority |
| `domain/ui/ui_program` | domain-ui-separate-owner | 26 | 25 | 25 | 50364 | defer to RunenUI/cutover authority |
| `domain/ui/ui_controls` | domain-ui-separate-owner | 100 | 99 | 70 | 538623 | defer to RunenUI/cutover authority |
| `domain/ui/ui_artifacts` | domain-ui-separate-owner | 27 | 26 | 24 | 57805 | defer to RunenUI/cutover authority |
| `domain/ui/ui_runtime_view` | domain-ui-separate-owner | 3 | 2 | 1 | 43218 | defer to RunenUI/cutover authority |
| `domain/ui/ui_render_primitives` | domain-ui-separate-owner | 2 | 1 | 1 | 25413 | defer to RunenUI/cutover authority |
| `domain/ui/ui_headless_render` | domain-ui-separate-owner | 3 | 2 | 1 | 24151 | defer to RunenUI/cutover authority |
| `domain/ui/ui_headless_render_data` | domain-ui-separate-owner | 3 | 2 | 1 | 21191 | defer to RunenUI/cutover authority |
| `domain/ui/ui_static_mount` | domain-ui-separate-owner | 9 | 8 | 1 | 26867 | defer to RunenUI/cutover authority |
| `domain/ui/ui_compiler` | domain-ui-separate-owner | 8 | 7 | 6 | 40199 | defer to RunenUI/cutover authority |
| `domain/ui/ui_evaluator` | domain-ui-separate-owner | 8 | 7 | 6 | 14307 | defer to RunenUI/cutover authority |
| `domain/ui/ui_state` | domain-ui-separate-owner | 2 | 1 | 1 | 19366 | defer to RunenUI/cutover authority |
| `domain/ui/ui_binding` | domain-ui-separate-owner | 2 | 1 | 1 | 21437 | defer to RunenUI/cutover authority |
| `domain/ui/ui_hosts` | domain-ui-separate-owner | 2 | 1 | 1 | 17605 | defer to RunenUI/cutover authority |
| `domain/ui/ui_testing` | domain-ui-separate-owner | 11 | 10 | 9 | 35350 | defer to RunenUI/cutover authority |
| `domain/ui/ui_accessibility` | domain-ui-separate-owner | 2 | 1 | 1 | 5158 | defer to RunenUI/cutover authority |
| `domain/ui/ui_geometry` | domain-ui-separate-owner | 2 | 1 | 1 | 8547 | defer to RunenUI/cutover authority |
| `domain/ui/ui_tree` | domain-ui-separate-owner | 15 | 14 | 14 | 46494 | defer to RunenUI/cutover authority |
| `domain/ui/ui_runtime` | domain-ui-separate-owner | 111 | 110 | 82 | 682887 | defer to RunenUI/cutover authority |
| `domain/ui/ui_widgets` | domain-ui-separate-owner | 24 | 23 | 23 | 34047 | defer to RunenUI/cutover authority |
| `domain/ui/ui_program_lowering` | domain-ui-separate-owner | 11 | 10 | 6 | 69106 | defer to RunenUI/cutover authority |
| `domain/ui/ui_story` | domain-ui-separate-owner | 37 | 36 | 35 | 178193 | defer to RunenUI/cutover authority |
| `domain/ui/ui_app_integration` | domain-ui-separate-owner | 12 | 11 | 9 | 47564 | defer to RunenUI/cutover authority |
| `domain/editor/editor_core` | domain-editor | 18 | 17 | 17 | 36092 | keep unless owner-specific evidence changes |
| `domain/editor/editor_definition` | domain-editor | 20 | 19 | 18 | 122097 | keep unless owner-specific evidence changes |
| `domain/editor/editor_preview` | domain-editor | 11 | 10 | 10 | 30815 | keep unless owner-specific evidence changes |
| `domain/editor/editor_shell` | domain-editor | 99 | 98 | 97 | 1443301 | keep crate; predecessor cleanup + local decomposition candidate |
| `domain/editor/editor_viewport` | domain-editor | 13 | 12 | 12 | 30224 | keep unless owner-specific evidence changes |
| `domain/editor/editor_scene` | domain-editor | 42 | 41 | 41 | 198220 | keep unless owner-specific evidence changes |
| `domain/editor/editor_inspector` | domain-editor | 18 | 17 | 17 | 47651 | keep unless owner-specific evidence changes |
| `domain/editor/editor_persistence` | domain-editor | 9 | 8 | 8 | 56152 | keep unless owner-specific evidence changes |
| `foundation/id` | foundation | 9 | 7 | 7 | 23485 | keep boundary |
| `foundation/id_macros` | foundation | 3 | 2 | 1 | 4575 | keep boundary |
| `foundation/diagnostics` | foundation | 14 | 13 | 12 | 74320 | keep boundary |
| `foundation/ratification` | foundation | 9 | 8 | 7 | 24754 | keep boundary |
| `foundation/schema` | foundation | 15 | 14 | 14 | 57844 | keep boundary |
| `foundation/commands` | foundation | 12 | 11 | 11 | 37058 | keep boundary |
| `foundation/resource_ref` | foundation | 2 | 1 | 1 | 7124 | keep boundary |
| `engine` | engine/runtime | 496 | 487 | 353 | 4704409 | keep runtime owner; automation persistence candidate; render separately owned |
| `engine_render_macros` | engine/runtime | 2 | 1 | 1 | 6915 | keep unless owner-specific evidence changes |
| `apps/runenwerk_editor` | application | 271 | 270 | 243 | 3821205 | keep app; shell/runtime/self-authoring decomposition candidates |
| `apps/runenwerk_draw` | application | 35 | 34 | 31 | 438394 | keep unless owner-specific evidence changes |
| `apps/runenwerk_runtime_preview` | application | 5 | 4 | 3 | 37431 | keep unless owner-specific evidence changes |
| `apps/runenwerk_render_lab` | application | 10 | 9 | 8 | 314532 | keep; no size-only refactor |
| `apps/runenwerk_arena` | application | 14 | 13 | 8 | 151620 | keep unless owner-specific evidence changes |
| `adapters/native_tablet_input` | adapter | 9 | 8 | 8 | 129602 | keep unless owner-specific evidence changes |

## Dependency-direction disposition

The manifest audit for #1153 found no foundation package depending upward on domain/engine/apps and no reviewed domain package depending upward on Engine/apps. Engine composes downward domain/foundation contracts and accepted external frameworks; applications compose Engine/domain/adapter contracts. The only reviewed app-to-app edge, `runenwerk_editor -> runenwerk_runtime_preview`, is a dev-dependency used by external-preview integration evidence rather than a production semantic dependency.

The current layer graph is therefore retained:

```text
foundation -> domain -> engine/runtime -> apps/adapters/tools
```

Package count or source size alone is not a merge/extraction criterion.
