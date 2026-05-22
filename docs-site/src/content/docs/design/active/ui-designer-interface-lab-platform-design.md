---
title: UI Designer And Interface Lab Platform
description: Active design for a generic UI/interface Designer and Lab platform spanning editor/workbench UI and game-runtime UI.
status: active
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-21
related_adrs:
  - ../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/accepted/0006-editor-surface-provider-plugin-seam.md
  - ../../adr/accepted/0010-graph-substrate-canvas-boundary.md
  - ../../adr/accepted/0012-capability-workbench-clean-break.md
related_designs:
  - ../accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../accepted/ui-designer-target-projection-profiles-design.md
  - ../accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md
  - ../accepted/ui-designer-component-surface-and-widget-recipe-library-design.md
  - ../accepted/ui-designer-view-model-capability-and-intent-binding-design.md
  - ../accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../accepted/ui-designer-persistence-migration-diff-and-activation-design.md
  - ../accepted/ui-designer-production-readiness-and-evidence-design.md
  - ./editor-tool-suite-registry-and-workbench-host-design.md
  - ./runenwerk-capability-workbench-target-architecture.md
  - ./editor-ui-workspace-tool-surface-architecture.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# UI Designer And Interface Lab Platform

## Status

This is an active design, not an accepted implementation contract.

It defines the long-term direction for the `PT-UI-DESIGN` production track. It
does not authorize code, schema, runtime, UI-surface, or roadmap execution-state
changes by itself. Implementation must still move through accepted designs,
architecture governance when ownership changes, WR roadmap legality, validation,
and closeout evidence.

## Purpose

Runenwerk needs a generic UI/interface authoring platform that can design,
inspect, validate, preview, migrate, and prove both editor/workbench UI and
game-runtime UI.

The platform is definition-driven. It owns UI/interface definition mechanics,
not product, editor, gameplay, render, material, asset, scene, simulation,
save-game, network, or project truth.

The target pipeline is:

```text
authored UI/interface definitions
  -> Canonical UI IR
  -> deterministic composition
  -> target projection plans
  -> editor workbench and game runtime UI projections
  -> app/runtime/provider consumers
```

## Relationship To PT-WB-CAP

`PT-WB-CAP` is the completed Workbench substrate for the editor/workbench target
profile. It provides typed suite/profile/provider identity, host capability
policy, product/service declarations, and multi-host Workbench composition.

`PT-UI-DESIGN` sits above that substrate as the generic UI/interface
Designer/Lab platform. The editor/workbench target profile uses `PT-WB-CAP`; the
game-runtime UI target must not depend on editor shell ownership.

Workbench is one target profile, not the whole Designer platform.

## Architecture Domains

`domain/ui` owns runtime-agnostic UI primitives, retained UI model, layout
primitives, style primitives, focus/input/navigation substrate, accessibility
substrate, and reusable interaction substrate.

`domain/ui/ui_definition` is the current owner for generic authored
UI/interface definition documents, Canonical UI IR, validation, normalization,
migration, diagnostics, source maps, and retained UI formation. The accepted
PM-UI-DESIGN-002 design fixes this as the near-term owner. A future standalone
`domain/ui_definition` crate remains a possible extraction only after a new
accepted design or ADR.

`domain/editor/editor_definition` owns editor/workbench-specific extensions:
workbench profiles, suites, panels, menus, shortcuts, docking/splits/tabs, tool
surfaces, provider families, host policy references, and editor command
declarations.

`domain/editor/editor_shell` owns Workbench host contracts,
suite/profile/provider declarations, host policy, and fail-closed projection
vocabulary for the editor/workbench target.

Future `domain/game_ui` or `domain/game/interface` owns game-runtime UI target
extensions: HUDs, health bars, stamina/mana bars, inventory screens, equipment
screens, hotbars, quest trackers, dialogue UI, minimaps, damage numbers,
nameplates, boss frames, world-space UI, split-screen UI, safe-area rules,
gamepad/touch navigation, platform prompt glyphs, and game-runtime UI projection
declarations.

`apps/runenwerk_editor` owns concrete Designer/Lab app surfaces, project IO,
live preview orchestration, provider implementation, and app command bridges. It
must not own canonical UI/interface truth.

## Non-Negotiable Architecture Invariants

- Designer documents are source truth only for UI/interface definitions.
- Designer documents may include editor/workbench UI and game-runtime UI
  definitions.
- Designer documents are not source truth for editor semantics, gameplay
  semantics, render semantics, material semantics, scene semantics, asset truth,
  simulation, save-game state, network state, or project truth.
- UI definitions may bind only to domain-owned view models.
- UI definitions may emit validated command/intent declarations only.
- UI definitions must not mutate domain state directly.
- Runtime projections must be reproducible from authored definitions, target
  profile, policy, fixtures, and validated composition.
- Preview, runtime, app, and provider layers are consumers of projection output,
  not source truth.
- Denied capabilities fail closed with typed diagnostics.
- Visual edits preserve stable ids and produce reviewable textual diffs.

## Canonical Definition Pipeline

```text
Authored UI Definition Documents
  -> parse + schema/version check
  -> Canonical UI IR
  -> composition / resolve / validate
  -> Target Projection Plan
     -> Editor Workbench Projection
     -> Game Runtime UI Projection
  -> runtime/app/provider layers consume projections but do not own source truth
```

The Canonical UI IR is the convergence point for visual and textual editing.
Every accepted visual edit must round-trip through this IR before activation.

## Target Profiles

### Editor Workbench UI Target

The editor/workbench target covers:

- workbench profiles;
- suites;
- panels;
- docking, splits, and tabs;
- menus;
- shortcuts;
- tool surfaces;
- provider families;
- host policy;
- capability gates;
- editor command routing;
- diagnostics and tool-lab surfaces.

This target may depend on `domain/editor/editor_shell` and `PT-WB-CAP`
contracts.

### Game Runtime UI Target

The game-runtime target covers:

- HUD layers;
- health bars;
- stamina and mana bars;
- inventory screens;
- equipment screens;
- hotbars;
- quest trackers;
- dialogue UI;
- minimaps;
- damage numbers;
- nameplates;
- boss frames;
- world-space UI;
- split-screen UI;
- safe areas;
- gamepad, touch, keyboard, and mouse navigation;
- platform prompt glyphs;
- accessibility modes;
- game-runtime performance and readability budgets.

This target must not depend on editor shell ownership. It may use shared
`domain/ui` and future `domain/ui_definition` contracts plus game-specific
target extensions.

## Definition Composition And Strength Ordering

The composition model must support deterministic strength ordering for:

- base UI library;
- shared component recipes;
- target profile defaults;
- project, game, and editor overrides;
- skin and theme packages;
- platform overrides;
- accessibility modes;
- localization and text expansion modes;
- user preferences;
- preview overrides;
- host/runtime policy overlays.

The model is inspired by layered composition systems, but Runenwerk must define
its own deterministic UI/interface composition semantics. Conflict resolution
must be inspectable and must report the losing source, winning source, affected
target profile, and activation impact.

## Styling Model

The shared styling model must include:

- primitive tokens;
- semantic tokens;
- component tokens;
- state tokens;
- token alias graph;
- token modes;
- theme packages;
- skin packages;
- platform overrides;
- accessibility overrides;
- token cycle diagnostics;
- deterministic style resolution.

Style resolution must be reproducible from authored definitions and the selected
target profile. Token cycles, missing aliases, incompatible modes, and
accessibility overrides must produce typed diagnostics.

## Binding Model

UI definitions may bind to read-only domain-owned view models.

Editor/workbench examples include:

- asset browser;
- material inspector;
- diagnostics;
- graph editors;
- tool surfaces.

Game examples include:

- combat HUD;
- inventory;
- abilities;
- quests;
- dialogue;
- minimap;
- player status;
- party frames;
- boss frames;
- world-space UI.

UI definitions may emit validated intents only. Editor intent examples include
editor commands, menu intents, and shortcut intents. Game intent examples
include `UseItem`, `SelectInventorySlot`, `OpenQuest`, `ChooseDialogueOption`,
`PingMinimap`, and `ActivateHotbarSlot`.

UI definitions must not own or directly mutate domain truth.

## Typed Diagnostics Taxonomy

Designer diagnostics must be typed and grouped by:

- schema/version;
- id/reference;
- token/theme;
- layout/composition;
- command/shortcut/focus;
- host-policy/capability;
- provider/surface;
- accessibility;
- performance-budget;
- migration/compatibility.

Each diagnostic must define:

- source location;
- affected target/profile/host/suite;
- owning domain;
- severity;
- activation impact;
- suggested fix.

Denied capabilities and unsupported target-profile features must fail closed with
typed diagnostics.

## Fixture, Scenario, And Golden Evidence Model

The Designer/Lab platform must support evidence-oriented authoring:

- fixtures for empty, loading, error, denied, offline, and heavy states;
- replayable interaction scenarios;
- expected diagnostics and UI state transitions;
- golden projection snapshots;
- visual captures;
- accessibility reports;
- migration reports;
- performance budget reports.

Fixtures and scenarios must be target-profile-aware so the same Canonical UI IR
can be validated against editor/workbench and game-runtime expectations.

## Round-Trip Editing Guarantees

Visual editing must preserve:

- stable ids;
- deterministic formatting;
- visual/textual convergence through Canonical UI IR;
- compatible unknown field preservation where possible;
- schema fail-closed behavior;
- reviewable diffs;
- migration dry-run before activation.

Any edit that cannot produce a deterministic textual diff must remain a preview
only and must not activate.

## Non-Goals

This first slice does not include:

- Rust source changes;
- schema changes;
- runtime behavior changes;
- UI surface changes;
- formatter changes unless validators explicitly require them;
- renderer, product, or domain semantic ownership changes;
- external plugin sandbox work;
- roadmap execution-state changes except registering the planning track.

This design also does not create a standalone `domain/ui_definition`,
`domain/game_ui`, or `domain/game/interface` crate. Those are future boundary
decisions.

## Promotion Criteria

This design can move from `active` to `accepted` only after:

- the production track validates;
- docs validators pass;
- the domain split is unambiguous;
- editor/workbench and game-runtime target profiles are explicit;
- architecture governance decides whether an ADR or future
  `domain/ui_definition` / `domain/game_ui` crate boundary decision is required;
- first implementation slices have clear WR roadmap legality.

## Validation

The planning slice must pass:

```text
task production:render
task docs:validate
task production:validate
task production:check
task roadmap:validate
task roadmap:check
task planning:validate
```

## Acceptance Criteria

- `ui-designer-interface-lab-platform-design.md` exists and uses the generic
  title `UI Designer And Interface Lab Platform`.
- The design treats Workbench as one target profile and includes a separate
  game-runtime UI target profile.
- The design defines `domain/ui`, `domain/ui_definition`,
  `domain/editor/editor_definition`, `domain/editor/editor_shell`, future
  `domain/game_ui` or `domain/game/interface`, and `apps/runenwerk_editor`
  ownership boundaries.
- The canonical pipeline uses `Canonical UI IR`, `Target Projection Plan`,
  `Editor Workbench Projection`, and `Game Runtime UI Projection`.
- Workbench-only wording is not used as the platform-level description.
- The composition, styling, binding, diagnostics, fixture/golden, and round-trip
  sections are present.
- `PT-UI-DESIGN` and all ten `PM-UI-DESIGN-*` milestones validate against the
  existing production-track schema.
- Generated production docs are current after `task production:render`.
- No Rust source, schemas, runtime behavior, UI surfaces, roadmap execution
  state beyond registering the planning track, staging, or commits change.
