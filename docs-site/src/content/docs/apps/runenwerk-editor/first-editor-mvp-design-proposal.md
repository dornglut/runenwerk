---
title: First Editor MVP Design Proposal
description: "MVP scope and success criteria for Runenwerk's first live 3D scene authoring editor."
---

# First Editor MVP Design Proposal

## Purpose

The first editor MVP exists to prove that Runenwerk can support live 3D scene authoring on top of the current engine/runtime architecture.

It is not a full production editor.
It is not a UI editor.
It is not a 2D-first tool.

Its job is to validate the core authoring loop:

- readable editor UI
- engine-owned window/runtime integration
- document-driven scene state
- projection into runtime/world state
- viewport rendering of real scene entities
- viewport picking and hit detection
- outliner, inspector, and viewport selection sync
- transform editing
- translate gizmo interaction
- undo and redo
- scene save/load shortly after MVP

## MVP Goal

A user can launch the editor and build a small 3D SDF graybox scene without touching code.

That means they can:

- open the editor window
- read panel titles and field labels
- see at least one real SDF entity in the viewport
- select entities from viewport and outliner
- inspect the selected entity
- edit basic properties
- move the entity with a translate gizmo
- undo and redo those edits
- save and load the scene soon after the first usable slice

## Core MVP Outcome

The first MVP should be capable of authoring:

- a simple 3D scene
- built from basic SDF primitives
- arranged spatially in a real viewport
- with basic scene hierarchy and transform editing

Example MVP scene:

- floor
- 3 to 10 primitive objects
- named entities
- visible transforms
- selection and movement working end-to-end

This is enough to prove the editor architecture.

## What the MVP Is

The first MVP is a 3D scene authoring shell.

It focuses on:

- scene structure
- selection
- inspection
- transform editing
- viewport interaction
- history
- basic persistence

It should support graybox authoring, not final content authoring.

## What the MVP Is Not

The first MVP does not include:

- advanced docking system
- asset browser
- prefab/blueprint authoring
- material editor
- animation/timeline tools
- scripting tools
- multiplayer/session tools
- terrain/world streaming authoring
- advanced component authoring breadth
- UI editor mode
- 2D scene editor mode
- polished text/layout system beyond what is needed for usability

These are later phases.

## Primary Authoring Domain

### First Domain: 3D

The editor MVP should target 3D scene authoring first.

Reason:

- it proves the main runtime/editor loop
- it validates projection into the engine world
- it validates picking and hit detection
- it validates gizmos and transforms
- it fits the SDF-native engine direction

### Not First: UI

UI authoring is important, but should come later as a separate authoring mode once the editor shell and runtime loop are stable.

### Not First: 2D

2D should not be first unless it becomes a primary product requirement.

## MVP Layout

The first MVP should use a fixed desktop workspace with no general docking system.

### Top Bar

Contains:

- File menu
- Edit menu
- View menu
- Save button
- dirty-state indicator
- Undo / Redo
- tool mode buttons
  - Select
  - Move
- optional play/simulate controls if already available

### Left Panel

Outliner only.

Purpose:

- scene structure
- entity list/tree
- selection from hierarchy

### Center Panel

Viewport only.

Purpose:

- render real scene content
- select entities spatially
- manipulate selection with gizmo

### Right Panel

Inspector only.

Purpose:

- inspect selected entity
- edit basic properties

### Bottom Panel

Console only.

Purpose:

- logs
- bring-up diagnostics
- hit detection debug visibility
- command/history visibility during early development

### Layout Policy

- left, right, and bottom panels may be resizable
- bottom panel may be collapsible
- viewport stays dominant
- no arbitrary docking in MVP
- no tab overload in MVP

## Required Panels and Contents

## 1. Outliner

### Purpose

The outliner is the structural navigation surface for the scene.

### MVP capabilities

- list entities in scene
- selected row highlight
- click row to select entity
- basic hierarchy/tree if already supported
- search field
- entity display name

### Nice after MVP

- create/delete/duplicate
- lock/visibility toggles
- context menus
- folders/collections/layers

## 2. Viewport

### Purpose

The viewport is the primary spatial authoring surface.

### MVP capabilities

- render at least one real SDF scene entity
- render multiple simple SDF entities shortly after first bring-up
- selectable viewport entities
- translate gizmo for selected entity
- visible selection state
- camera navigation
  - orbit
  - pan
  - zoom
- temporary hit detection debug info in console/log

### Required rendered content

The viewport should show at least one real projected scene entity, ideally an SDF primitive such as:

- box
- sphere
- capsule

The first usable viewport must not be a fake placeholder if the goal is validating authoring architecture.

### Viewport toolbar

MVP:

- Select tool
- Move tool
- Focus selection optional

Later:

- Rotate
- Scale
- Local/World toggle
- Snap toggle
- Debug overlays

## 3. Inspector

### Purpose

The inspector edits the selected entity.

### MVP capabilities

- selected entity header
- selected entity name/id
- editable transform section
  - position x/y/z
- optional editable name field
- optional simple SDF shape section if ready

### Inspector structure

Use sections rather than one flat list.

Recommended MVP sections:

- General
- Transform
- Shape (if available)

### Rules

- all edits go through commands/history
- no silent direct world mutation shortcuts
- selection changes must update inspector immediately

## 4. Console

### Purpose

The console is a bring-up and diagnostics panel.

### MVP capabilities

- show logs
- clear logs
- auto-scroll
- level prefix if already available

### Expected MVP log usefulness

During bring-up, the console should be used for:

- pointer routing traces
- viewport click traces
- hit detection results
- selected entity id/name
- command execution logs
- undo/redo logs
- projection sync confirmation

This is intentionally temporary-heavy during MVP.

## Text Rendering Requirement

Readable text is a hard blocker for the editor MVP.

Because the current editor has no usable fallback text path, the first MVP requires panel text bring-up before further authoring interaction work can be considered complete.

### Minimum required readable text

- panel titles
- outliner row labels
- inspector section titles
- inspector field labels
- at least basic field values

### Current implication

If MSDF is the only intended text path, then MSDF text bring-up for editor panels is part of MVP-critical work, not polish.

## Selection Model

Selection must be synchronized across all three primary views:

- viewport
- outliner
- inspector

### Required MVP selection flows

1. Click entity in viewport -> outliner selection updates -> inspector updates
2. Click entity in outliner -> viewport selection highlight updates -> inspector updates
3. Selection is singular in MVP

### Later

- multi-select
- marquee
- selection sets
- lock/pin selection

## Hit Detection and Picking

Viewport hit detection is part of the first real authoring loop.

### MVP requirement

A viewport click on a rendered entity must resolve to a selectable entity.

### Bring-up strategy

Temporary console/debug logging is explicitly recommended during MVP for:

- pointer position
- viewport-local coordinates
- hit result
- selected entity id
- gizmo handle hit if relevant

This is not final UX; it is bring-up instrumentation.

## Transform Editing

### MVP requirement

The user must be able to modify entity position through:

1. inspector numeric editing
2. translate gizmo in viewport

### Required behavior

- edit updates the document-authoritative state
- projection updates runtime/world state
- viewport updates visibly
- command/history receives the change

### Deferred

- rotate gizmo
- scale gizmo
- snapping
- coordinate space switching

## Undo and Redo

Undo/redo is part of the MVP, not post-MVP polish.

### MVP requirement

The user must be able to:

- move an entity
- undo the move
- redo the move
- edit an inspector value
- undo the edit
- redo the edit

### Why included early

Undo/redo proves:

- command routing correctness
- document authority correctness
- projection consistency
- editor history integration

## Persistence

Persistence is not strictly required for the very first visible shell bring-up, but it should follow immediately after the first usable interaction slice.

### MVP-adjacent persistence requirement

The user should soon be able to:

- save a scene
- load a scene
- recover entity names/transforms/basic structure

### Reason

Without persistence, the editor proves interaction, but not authoring continuity.

## Recommended MVP Entity Support

Start with a very small supported set.

### Required

- entity name
- transform
- one simple SDF primitive kind

### Good first primitive set

- Box
- Sphere
- Capsule

### What this enables

- graybox rooms
- test layouts
- spatial composition
- proof of projection/render/picking/editing loop

## MVP User Workflow

The first valid user workflow should be:

1. launch editor
2. read panel titles and labels
3. see one or more SDF entities in viewport
4. click entity in viewport
5. see outliner selection update
6. inspect entity in inspector
7. edit transform value
8. move entity with translate gizmo
9. undo
10. redo
11. save/load soon after initial interaction slice

If this works, the MVP is real.

## MVP Success Criteria

The first editor MVP is successful if all of the following are true:

### Runtime/UI

- editor launches in engine-owned runtime
- panels render every frame
- readable text exists in the panels

### Scene/selection

- at least one real SDF entity renders in viewport
- selection works from viewport and outliner
- inspector reflects selected entity

### Editing

- position can be edited numerically
- translate gizmo works
- edits visibly affect the viewport

### History

- undo works
- redo works

### Diagnostics

- console shows bring-up useful logs
- hit detection and command flow are observable during development

### Persistence shortly after

- scene can be saved and loaded

## Implementation Priority Order

Recommended next order:

1. Panel text bring-up (MSDF path if that is the only text path)
2. Live editor shell in engine runtime
3. Viewport rendering of a real SDF entity
4. Viewport hit detection with debug logging
5. Selection sync between viewport, outliner, and inspector
6. Inspector transform editing
7. Translate gizmo
8. Undo / Redo
9. Persistence

## Post-MVP Expansion

After the first MVP is stable, the next likely expansions are:

- rotate/scale gizmos
- more component/shape editing
- better viewport overlays
- docking/layout persistence
- create/delete/duplicate flows
- UI authoring mode
- 2D mode if needed
- asset workflows
- deeper inspector/component authoring

## Final Recommendation

The first editor MVP should be treated as:

> a 3D SDF scene authoring MVP with readable text, fixed workspace layout, synced selection, transform editing, translate gizmo interaction, undo/redo, and near-immediate persistence.

This is the smallest version that proves the editor architecture instead of merely showing an empty editor window.
