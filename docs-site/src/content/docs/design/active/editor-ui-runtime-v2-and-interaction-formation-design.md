---
title: Editor UI Runtime V2 And Interaction Formation Design
description: Active design gate for execution-neutral UI interaction contracts formed before retained UI runtime execution.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-15
related:
  - ./ui-definition-formation-foundation-design.md
  - ./editor-self-authoring-and-final-ui-design.md
  - ./editor-shell-menu-and-tab-chrome-polish-design.md
  - ./editor-ui-popup-adornment-drop-preview-contract.md
  - ../deferred/ui-model-multiple-execution-strategies-design.md
  - ../../adr/accepted/0009-ui-interaction-formation-v2.md
  - ../../domain/ui/roadmap.md
  - ../../apps/runenwerk-editor/roadmap.md
---

# Editor UI Runtime V2 And Interaction Formation Design

## Status

Active design for the accepted Interaction V2 architecture in
`docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md`.
This document does not promote compiled-reactive UI, ECS-driven UI, or an
external UI framework.

The first implementation target remains the existing retained UI stack. The new long-term decision is to form interaction contracts before retained UI execution so repeated menu, popup, scroll, chrome, docking, and viewport-input regressions are solved at the shared contract layer.

## Problem

Recent editor shell bugs repeat the same pattern:

- popup menus are built as local retained nodes without a shared popup stack contract;
- scroll containers return unhandled input at boundaries, allowing viewport zoom to receive menu wheel events;
- submenu anchoring depends on parent popup nodes that disappear when a single active menu enum switches kind;
- menu width and item stretch are controlled by ad hoc button and popup defaults;
- tab close and active indicators are overlays instead of structural chrome slots;
- docking split previews compete with tab-strip reorder hit targets;
- viewport status/statistics controls lack an explicit overflow policy.

These are not independent polish defects. They are missing interaction contracts in the accepted retained UI path.

## Core Decision

Add an execution-neutral interaction formation step above retained UI:

```text
AuthoredUiTemplate
  -> validate
  -> normalize
  -> FormedInteractionModel
  -> FormedRetainedUiProduct
  -> ui_runtime enforcement
  -> render/product-surface output
```

`FormedInteractionModel` is not a second runtime and not a compiled reactive program. It is the validated interaction contract that retained UI consumes first and that future execution targets may consume later.
Renderer and product-surface output remain derived products, not UI authority.

## Migration Spine

Interaction V2 is a contract spine, not a broad shell polish implementation.
Each migrated slice follows the same order:

```text
authored/editor definition vocabulary
  -> validation rule
  -> normalized interaction contract
  -> retained UI formation adapter
  -> ui_runtime enforcement
  -> editor/app guard
```

This order is the boundary between WR-025 and later retained-UI work such as
WR-024. WR-025 defines the vocabulary, validation, ownership rule, migration
adapter boundary, and required guard. WR-024 may then implement the retained UI
slice only after the relevant contract has an explicit owner and expected
fitness function.

Compatibility-only work is allowed when it records current behavior without
claiming long-term ownership. Compatibility work must name the old path, the new
contract it will migrate toward, and the guard that prevents the old path from
expanding.

## Ownership

`domain/ui/ui_definition` owns generic authored interaction vocabulary, validation, normalization, source maps, and formation outputs that do not mention editor commands, runtime `WidgetId`, ECS entities, or concrete shell sessions.

`domain/ui/ui_runtime` owns runtime input routing, focus, scroll ownership, layout measurement, clipping, hit testing, and frame output for formed retained UI products.

`domain/editor/editor_definition` owns editor-specific menu, workspace, chrome, command binding, panel registry, and tool-surface definition descriptors that reference generic UI contracts without moving editor semantics into `domain/ui`.

`domain/editor/editor_shell` owns shell composition, workspace host semantics, command routing, provider presentation, and Strangler adapters from current shell state into formed interaction contracts.

`apps/runenwerk_editor` owns app/runtime integration, viewport input arbitration, file/project IO, fixture loading policy, and concrete command execution.

## Interaction Contracts

The first V2 contract set must cover:

- `FormedInteractionModel`: one formed interaction artifact associated with a normalized template or editor shell surface;
- popup stack and menu scopes: parent/child menus, submenu anchors, focus return, outside dismiss, escape behavior, and layer policy;
- scroll ownership policy: nearest owner selection, axis ownership, boundary consumption, scrollbar drag ownership, and viewport fallback rejection;
- focus scope and dismissal policy: menus, popups, text inputs, modal-like surfaces, and ordinary panels;
- menu sizing policy: max intrinsic item measurement, popup clamp, scroll fallback, and item fill width;
- dock/drop-zone policy: tab reorder zones, split zones, precedence, invalid targets, and preview-only state;
- chrome slot policy: fixed close slot, active indicator slot, label slot, and drag region for tabs and workspace buttons;
- status/metadata overflow policy: scrollable or wrapped status bars with stable priority for essential metrics.

## Guardrail Contract

Every Interaction V2 slice must answer these questions before retained UI code
changes:

- owning definition vocabulary: generic `domain/ui/ui_definition` or
  editor-specific `domain/editor/editor_definition`;
- formed contract output: the record in `FormedInteractionModel` that retained
  UI consumes;
- runtime enforcement point: the `domain/ui/ui_runtime` layout/input/focus/hit
  testing behavior that must become deterministic;
- shell adapter: the `domain/editor/editor_shell` composition or workspace
  projection path that maps existing shell state into the formed contract;
- app boundary: the `apps/runenwerk_editor` viewport/input/command behavior that
  must not become UI authority;
- regression guard: definition validation, runtime tests, shell tests, app
  guards, and screenshot or primitive-order checks where frame order matters.

If a slice cannot name those six parts, it is not ready for implementation
outside explicit compatibility evidence.

## Strangler Migration

Phase 1 - Design Gate

- Accept ADR 0009, update roadmap source/indexes, and keep subordinate links from the existing polish design.
- Do not change runtime behavior in this phase.
- Exit when roadmap render/check and docs validation pass.

Phase 2 - Menus And Scroll

- Introduce popup stack/menu scope contracts.
- Update retained runtime wheel dispatch to report ownership separately from mutation.
- Migrate toolbar menus, viewport tools/options/details, tab action menus, and Switch Type submenu.
- Exit only after tests prove outside-dismiss, focus return, menu layer order,
  nearest scroll owner selection, and viewport wheel rejection when UI owns the
  event.

Phase 3 - Chrome

- Replace overlay close/indicator behavior with structural tab and workspace chrome slots.
- Remove Unicode glyph dependence for active indicators.
- Keep authored/runtime identity boundaries intact.
- Exit only after shell tests prove close slot, active indicator slot, label
  slot, and drag region do not overlap or reorder unpredictably.

Phase 4 - Docking

- Make tab-strip reorder zones suppress split previews.
- Represent split previews through semantic drop zones and preview-only state.
- Exit only after split-border precedence, tab reorder precedence, invalid
  targets, and preview-only state are covered.

Phase 5 - Metrics And Status

- Project FPS/frame time from an always-on runtime/editor metric source.
- Apply explicit overflow policy to viewport statistics/details/status bars.
- Exit only after status overflow cannot steal viewport input and app guards
  prove viewport arbitration remains fail-closed.

## Deferred Execution Targets

Compiled-reactive UI and ECS-driven UI remain deferred. If a future accepted design or ADR promotes either path, it must consume the same normalized UI definitions and formed interaction contracts as an additional target. No future target may replace authored UI identity, source maps, command ratification, or the renderer-as-derived-product rule:

```text
NormalizedUiTemplate
  -> FormedInteractionModel
  -> FormedRetainedUiProduct
  -> future CompiledUiProgram
  -> future EcsUiSpawnPlan
```

No implementation in this design may make compiled-reactive or ECS-driven UI the default editor path.

## Fitness Functions

The design is enforceable only with guard tests:

- UI definition/editor definition validation rejects submenu anchors without stable parent scope, scrollable popups without scroll ownership policy, menu lists without sizing/stretch policy, and tab strips without reorder-zone priority.
- UI runtime tests prove scroll boundary wheel input is owned, nearest scroll owner selection is deterministic, popup stack anchoring survives parent menu presence, and menu layout stretches all items to measured width.
- Editor shell tests prove Switch Type submenu anchoring, viewport stats overflow, left chrome slot order, and tab-strip reorder precedence over split previews.
- App guard tests prove viewport zoom only receives wheel after UI explicitly declines ownership and deferred UI execution strategies remain absent from production paths.

## Non-Goals

- No external UI framework adoption.
- No compiled-reactive runtime implementation.
- No ECS UI runtime implementation.
- No movement of editor command semantics into `domain/ui`.
- No rewrite of self-authoring source identity, route slots, or provider ratification boundaries.
