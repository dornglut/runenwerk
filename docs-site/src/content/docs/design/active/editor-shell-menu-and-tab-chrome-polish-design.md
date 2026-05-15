---
title: Editor Shell Menu And Tab Chrome Polish Design
description: Active design for popup contrast, scrollable menus, viewport statistics, submenu anchoring, and left-side close/active indicators.
status: active
owner: domain/editor/editor_shell
layer: editor-ui
canonical: true
last_reviewed: 2026-05-15
related_designs:
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ./editor-ui-popup-adornment-drop-preview-contract.md
  - ./editor-self-authoring-and-final-ui-design.md
related_adrs:
  - ../../adr/accepted/0009-ui-interaction-formation-v2.md
related_roadmaps:
  - ../../apps/runenwerk-editor/roadmap.md
  - ../../domain/ui/roadmap.md
---

# Editor Shell Menu And Tab Chrome Polish Design

## Status

Active supporting evidence. This is retained UI polish, not a new UI execution
strategy and not the long-term interaction contract owner.

This document is subordinate to ADR 0009 and
`editor-ui-runtime-v2-and-interaction-formation-design.md`. Keep it active while
the immediate polish work is useful, but do not treat it as the long-term owner
for popup stack, scroll ownership, focus return, chrome slots, docking/drop-zone
precedence, menu sizing, or status overflow contracts.

Do not start durable popup, scroll, focus, docking-zone, menu-sizing,
status-overflow, or viewport-input policy from this document before the
corresponding WR-025 Interaction V2 contract slice is defined. Work from this
document is valid only as a retained-UI implementation slice that consumes
Interaction V2 contracts, or as explicitly bounded compatibility evidence.

## Decisions

Owning code paths:

- `domain/editor/editor_shell/src/composition/surface_definition_context.rs::contrast_popup_theme`
- `domain/ui/ui_runtime/src/layout/engine.rs::layout_popup`
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_strip_from_frame`
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::register_tab_stack_chrome_routes`
- `domain/editor/editor_shell/src/composition/build_viewport_panel.rs::viewport_statistics_text`
- `domain/editor/editor_shell/src/composition/toolbar_definition.rs::project_workspace_close_buttons`

## Requirements

- Popup/menu contrast is centralized in shell theme helpers.
- Menus with more content than available screen space use scrollable popup content.
- Switch Type opens as a level-2 menu next to the Switch Type row.
- Tab and workspace close buttons sit on the left side of their anchor.
- The opposite side carries a fixed-size active indicator: open circle for inactive, filled dot for active.
- Viewport statistics include frame rate and frame time when metrics are available.

## Non Goals

- No change to the retained UI execution strategy.
- No icon-font dependency change.
- No hard-coded one-off popup colors outside shared helpers.

## Tests

Required coverage:

- `domain/ui/ui_runtime/src/layout/engine.rs::layout_popup` stretches a single scroll child when popup height clamps.
- toolbar, viewport, and tab-stack popups use the contrast helper.
- Switch Type route anchors to `tab_stack_surface_submenu_anchor_widget_id`.
- tab/workspace close overlays use left placement.
- active indicators reserve the same 18x18 space as close affordances.
