---
title: Runenwerk UI Platform Capability Roadmap
description: UI-only product roadmap for turning the story-driven golden workflow into an ergonomic platform covering base controls, interaction, text, graph canvas, timeline, world-space UI, effects, and visual authoring.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-16
related_designs:
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./ui-runtime-rendering-pipeline-roadmap.md
  - ./ui-program-architecture.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../domain/ui/story-acceptance-and-review-checklist.md
---

# Runenwerk UI Platform Capability Roadmap

## Status

This is an active UI-only productization roadmap. It sequences platform
capabilities that must be story-gated before advanced editor, game, or product
surfaces depend on them.

This roadmap does not authorize implementation, new crates, generated planning
document edits, or production-track completion by itself. Each implementation
slice still requires normal WR, production-track, validation, and closeout
authority.

## Decision

Runenwerk UI must be perfected as a platform before advanced editor, game, or
product surfaces depend on it.

Correct order:

```text
Story-driven golden workflow
  -> base component foundation
  -> interaction platform
  -> text platform
  -> GraphCanvas
  -> Timeline
  -> world-space UI
  -> UI effects
  -> visual UI builder
  -> docs/training/productization
```

This roadmap is UI-only. It does not define material, scene, render, gameplay,
or editor product tracks except as future consumers of the UI platform.

## Governing Rule

Existing components should be cheap to use.

New reusable component classes are platform work.

```text
Normal surface with existing controls:
  authored template + story manifest + host data/routes

New advanced control/platform feature:
  domain crate/module work + schemas + runtime behavior + stories + tests
  + renderer contract if needed
```

Do not hide real platform capability work behind fake simplicity.

## PT-UI-001 - Story-Driven Golden Workflow

### Goal

Make `UiStory` the canonical unit for authoring, preview, validation,
inspection, proof, and mount eligibility.

### Owns

```text
domain/ui/ui_story/
apps/runenwerk_editor/src/runtime/ui_gallery.rs
assets/ui_gallery/stories/
tools/ui/
```

### Required Outcome

The current hardcoded button gallery path is converted to manifest-driven
stories and a domain-owned story runner.

### Acceptance Criteria

- `domain/ui/ui_story` exists or an approved existing crate owns the same
  public story contract.
- Button basic and selected fixtures are represented by story manifests.
- `apps/runenwerk_editor/src/runtime/ui_gallery.rs::UI_GALLERY_FIXTURES` is
  removed.
- Gallery discovery reads story manifests from assets.
- `UiStoryRunReport` contains source, definition, formation, compiler, runtime
  view, binding, routes, layout/style/text/accessibility, interaction, render,
  static mount, preview, and verdict sections.
- At least one intentionally failing story proves diagnostic expectation
  matching.
- Gallery preview consumes selected story report output.
- No renderer-owned UI semantics are added.

### Non-Goals

- No visual UI builder.
- No new advanced component class.
- No broad product/editor feature work.

## PT-UI-002 - Base Component Foundation

### Goal

Make existing and near-term basic controls complete under the story matrix.

### Component Set

```text
Label
Button
Panel
Row
Column
Stack
Scroll
Toggle
Slider
TextInput
NumericInput
Select
Tabs
ListView
TreeView
TableView
InspectorField
ColorPicker
ActionPrompt
Tooltip
Popover
Modal
ContextMenu
```

### Owns

```text
domain/ui/ui_controls/
domain/ui/ui_definition/
domain/ui/ui_schema/
domain/ui/ui_program_lowering/
domain/ui/ui_runtime_view/
domain/ui/ui_render_primitives/
assets/ui_gallery/stories/controls/
```

### Acceptance Criteria

Every component has:

- property schema;
- state schema;
- event payload schema;
- layout kernel;
- interaction kernel;
- visual kernel;
- accessibility kernel;
- inspection kernel;
- runtime view projection;
- render primitive lowering if visual;
- default/hover/focused/disabled/invalid stories as applicable;
- failure stories for missing required fields and invalid properties;
- docs page.

## PT-UI-003 - Interaction Platform

### Goal

Make interaction reusable across controls, GraphCanvas, Timeline, UI Builder,
world-space UI, and game UI.

### Owns

```text
domain/ui/ui_input/
domain/ui/ui_runtime/
domain/ui/ui_focus/
domain/ui/ui_navigation/
domain/ui/ui_drag_drop/
```

If `ui_focus`, `ui_navigation`, or `ui_drag_drop` do not exist yet, initial work
must either stay inside authorized current owners or go through accepted
crate-creation procedure.

### Required Capabilities

- pointer capture;
- drag lifecycle;
- drop target negotiation;
- selection model;
- keyboard focus;
- gamepad focus;
- shortcuts;
- context menus;
- popovers;
- modal stack;
- focus restore;
- scroll plus focus interaction;
- input ownership reports.

### Acceptance Criteria

- Pointer click and keyboard activation emit equivalent semantic route
  proposals.
- Disabled controls do not emit activation.
- Drag/drop uses a reusable platform contract, not component-local ad hoc drag
  state.
- Interaction traces are replayable in stories.
- GraphCanvas and Timeline are blocked until this track provides their shared
  interaction primitives.

## PT-UI-004 - Text Platform

### Goal

Make text robust enough for labels, inputs, rich text, code editor,
localization, and accessibility.

### Owns

```text
domain/ui/ui_text/
domain/ui/ui_accessibility/
domain/ui/ui_theme/
domain/ui/ui_runtime_view/
assets/ui_gallery/stories/text/
```

### Required Capabilities

- shaping;
- wrapping;
- ellipsis;
- glyph fallback;
- selectable text;
- cursor movement;
- clipboard;
- IME path where supported;
- rich spans;
- monospace/code layout;
- syntax decoration hooks;
- localization expansion;
- source-mapped text diagnostics.

### Acceptance Criteria

- Text layout requests and results are visible in story reports.
- Missing glyphs are diagnostic-bearing.
- Text measurement affects layout.
- Rich text and code editor are not started until text
  selection/cursor/clipboard primitives are proven.

## PT-UI-005 - GraphCanvas Platform Component

### Goal

Create a reusable `GraphCanvas` platform component for node-based authoring.

### Consumers

- material graph;
- shader graph;
- behavior graph;
- dialogue graph;
- state machine graph;
- UI graph tooling if needed.

### Owns

```text
domain/ui/ui_graph_canvas/
domain/ui/ui_controls/
domain/ui/ui_program_lowering/
domain/ui/ui_runtime_view/
domain/ui/ui_render_primitives/
assets/ui_gallery/stories/platform/graph_canvas/
```

The `ui_graph_canvas` crate path is preferred only after accepted authority.

### Required Capabilities

- pan/zoom canvas;
- node layout;
- port rendering;
- edge routing;
- selection;
- marquee selection;
- drag node;
- connect edge;
- disconnect edge;
- context menu;
- keyboard shortcuts;
- minimap later only if core proof is stable.

### Acceptance Criteria

- `GraphCanvas` is package-backed and story-gated.
- Domain-specific graph semantics stay outside generic UI.
- Generic UI owns canvas interaction and visuals.
- Host/domain owns graph mutation and validation.
- No material-specific graph behavior is baked into generic UI.

## PT-UI-006 - Timeline Platform Component

### Goal

Create a reusable timeline/sequencer UI platform.

### Consumers

- animation editor;
- cutscene editor;
- audio timeline;
- keyframe editor;
- event sequencing;
- simulation playback tooling.

### Owns

```text
domain/ui/ui_timeline/
domain/ui/ui_controls/
domain/ui/ui_runtime_view/
domain/ui/ui_render_primitives/
assets/ui_gallery/stories/platform/timeline/
```

The `ui_timeline` crate path is preferred only after accepted authority.

### Required Capabilities

- time ruler;
- tracks;
- clips;
- keyframes;
- playhead;
- range selection;
- zoom/pan;
- snap policy;
- drag keyframe/clip;
- resize clip;
- route proposals for edit actions.

### Acceptance Criteria

- Timeline is generic.
- Animation/cutscene/audio semantics are host/domain data, not generic UI
  truth.
- Drag/selection reuses the interaction platform.
- Text measurement and layout are reused.

## PT-UI-007 - World-Space UI

### Goal

Make UI surfaces usable in world-space/diegetic 3D contexts without duplicating
the UI system.

### Owns

```text
domain/ui/ui_hosts/
domain/ui/ui_runtime/
domain/ui/ui_render_data/
domain/ui/ui_world_space/
engine/render integration adapters
assets/ui_gallery/stories/platform/world_space/
```

The `ui_world_space` crate path is preferred only after accepted authority.

### Required Capabilities

- world-space host profile;
- ray/pointer projection into UI plane;
- depth and occlusion policy;
- focus/selection policy;
- scale policy;
- controller/gamepad interaction;
- readable text constraints;
- route mapping through host/domain.

### Acceptance Criteria

- No separate world-space button system.
- World-space UI consumes the same story-proven surface source.
- World-space differences are host projection/input/render-profile
  differences.

## PT-UI-008 - UI Effects Platform

### Goal

Support shader-driven or advanced visual effects without moving UI semantics
into the renderer.

### Owns

```text
domain/ui/ui_effects/
domain/ui/ui_render_primitives/
domain/ui/ui_render_data/
engine/render UI adapter
assets/ui_gallery/stories/platform/effects/
```

The `ui_effects` crate path is preferred only after accepted authority.

### Required Capabilities

- effect tokens;
- effect parameters;
- safe effect variants;
- renderer capability checks;
- fallback diagnostics;
- snapshot-proofable output;
- no component semantic ownership in renderer.

### Acceptance Criteria

- Effects are declared as UI visual policy, not renderer-authored controls.
- Unsupported effects fail with diagnostics or declared fallback policy.
- Renderer adapter consumes effect primitives/contracts only.

## PT-UI-009 - Visual UI Builder

### Goal

Create a visual authoring product that edits the same authored UI source and
story manifests.

### Owns

```text
apps/runenwerk_ui_builder/
domain/ui/ui_story/
domain/ui/ui_definition/
domain/ui/ui_schema/
domain/ui/ui_controls/
domain/ui/ui_layout/
domain/ui/ui_theme/
```

### Required Capabilities

- story browser;
- component palette;
- drag/drop component placement;
- property editor;
- token picker;
- layout inspector;
- focus inspector;
- route/binding editor;
- source preview;
- live validation;
- save authored UI and story manifests;
- open story in UI gallery.

### Non-Negotiable

The builder must not create a second file format.

It edits:

```text
AuthoredUiTemplate / UiNodeDefinition source
UiStoryManifest
```

## PT-UI-010 - Docs, Training, And Productization

### Goal

Make the UI platform learnable and maintainable.

### Owns

```text
docs-site/src/content/docs/domain/ui/
docs-site/src/content/docs/design/active/
docs-site/src/content/docs/tutorials/ui/
apps/runenwerk_ui_gallery/
tools/ui/
```

### Required Docs

- UI mental model;
- story workflow;
- create a component story;
- create a surface story;
- binding/action route workflow;
- inspect layout;
- inspect style/tokens;
- inspect text;
- inspect accessibility;
- inspect render primitives;
- story matrix requirements;
- mount eligibility;
- advanced platform component guide;
- clean cutover guide from hardcoded gallery to story-driven gallery.

## Recommended Implementation Order

```text
PT-UI-001 Story-Driven Golden Workflow
PT-UI-002 Base Component Foundation
PT-UI-003 Interaction Platform
PT-UI-004 Text Platform
PT-UI-005 GraphCanvas
PT-UI-006 Timeline
PT-UI-007 World-Space UI
PT-UI-008 UI Effects
PT-UI-009 Visual UI Builder
PT-UI-010 Docs/Training/Productization
```

Do not start GraphCanvas, Timeline, UI Builder, or world-space UI before the
story workflow, interaction platform, and text platform are sufficient for them.

## Stop Conditions

Stop and redesign if any implementation:

- renders directly from authored `.ron`;
- keeps hardcoded gallery fixtures as a production path;
- adds a button/control-specific gallery pipeline instead of story reports;
- infers package truth from control-kind strings;
- lets unknown control kinds pass formation;
- allows renderer-owned component semantics;
- allows app/editor/game state mutation from generic UI;
- creates a second visual-builder UI format;
- creates component-specific drag/drop/selection/focus systems instead of
  platform interaction primitives;
- mounts a surface before story mount eligibility passes.
