---
title: UI Designer Live Preview Fixtures Scenarios And Target Matrix Design
description: Accepted design for PM-UI-DESIGN-008 target-profile-aware fixtures, scenarios, preview matrices, and replayable evidence contracts.
status: accepted
owner: editor
layer: domain/ui-definition
canonical: true
last_reviewed: 2026-05-22
related_adrs:
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/superseded/0006-editor-surface-provider-plugin-seam.md
  - ../../adr/superseded/0012-capability-workbench-clean-break.md
related_designs:
  - ./ui-designer-canonical-ir-and-composition-design.md
  - ./ui-designer-target-projection-profiles-design.md
  - ./ui-designer-visual-layout-and-interface-composition-design.md
  - ./ui-designer-theme-tokens-modes-skins-and-state-variants-design.md
  - ./ui-designer-component-surface-and-widget-recipe-library-design.md
  - ./ui-designer-view-model-capability-and-intent-binding-design.md
  - ../active/ui-designer-interface-lab-platform-design.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# UI Designer Live Preview Fixtures Scenarios And Target Matrix Design

## Status

This is the accepted implementation design for `PM-UI-DESIGN-008`.

It defines the ownership and contract shape for target-profile-aware fixtures,
interaction scenarios, preview matrices, and replayable evidence. It does not
implement code, select a WR roadmap row, or authorize product code until a
linked WR row exists and passes `task production:plan`.

## Goal

The UI Designer needs reproducible previews across editor/workbench and
game-runtime target profiles:

```text
authored UI definition
  -> fixture package
  -> scenario steps
  -> target matrix
  -> derived preview run
  -> diagnostics and evidence packet
```

Fixtures and scenarios describe expected inputs, policy overlays, target
profiles, diagnostics, and state transitions. They do not own runtime state,
provider sessions, screenshots, renderer resources, app windows, gameplay truth,
or editor command execution.

## Architecture Governance Result

Architecture governance accepts this implementation direction:

- `domain/ui/ui_definition` owns runtime-neutral fixture ids, scenario ids,
  matrix ids, preview policy references, expected diagnostics, expected UI state
  references, replay steps, and evidence packet descriptors.
- `domain/editor/editor_definition` owns editor/workbench-specific fixture
  package adapters for provider state, command policy, tool surfaces, menus,
  shortcuts, and workbench host policy references.
- Future game UI target domains own game-runtime fixture adapters for HUDs,
  inventories, dialogue, quests, safe areas, input schemes, accessibility modes,
  and runtime performance envelopes.
- `apps/runenwerk_editor` may orchestrate previews, captures, visual diffs, and
  user-facing Designer/Lab UI, but it must not own generic fixture, scenario, or
  matrix truth.
- Renderer, runtime, provider, and app layers consume derived preview plans and
  emit evidence; they do not become authored fixture source truth.

No new ADR is required for the first PM-008 design because ADR-0004 separates
description from execution, ADR-0005 treats projections as derived state,
ADR-0006 keeps provider proposals behind command boundaries, and ADR-0012
keeps host capability policy explicit. A future ADR or accepted design update
is required before generic preview fixtures own provider sessions, runtime
state, renderer handles, screenshots, app windows, or gameplay truth.

## Fixture Contract

Every generic fixture declaration includes:

- stable fixture id;
- target-profile compatibility;
- source package provenance;
- referenced UI definition, recipe, token, and binding package ids;
- data-state kind such as empty, loading, error, denied, offline, heavy, or
  accessibility mode;
- host/runtime policy overlay references;
- expected diagnostics;
- expected UI state references;
- missing-data and denied-capability expectations.

Fixtures provide deterministic inputs to preview formation. They do not execute
commands, fetch live provider data, mutate domain truth, or create renderer
resources.

## Scenario Contract

Every generic scenario declaration includes:

- stable scenario id;
- fixture reference;
- target-profile compatibility;
- ordered interaction steps;
- expected state transitions;
- expected diagnostics;
- timeout and budget references;
- accessibility and input-mode requirements;
- replay determinism requirements.

Scenario steps describe user intent and preview inputs, not direct domain
mutation. Activation is allowed only when all required capabilities, target
profile rules, data packages, policy overlays, and expected diagnostics are
valid.

## Target Matrix Contract

Every target matrix declaration includes:

- stable matrix id;
- fixture and scenario references;
- target profiles;
- platform, accessibility, localization, input, size, and performance axes;
- expected evidence packet names;
- activation mode: preview, dry-run, or acceptance evidence;
- fail-closed policy for missing targets or stale evidence.

The matrix produces derived preview runs and evidence packets. Evidence packets
may reference diagnostics, projection snapshots, accessibility reports,
performance budgets, and visual capture artifacts, but the first generic slice
only defines the contracts that make those artifacts reproducible.

## Diagnostics

Fixture, scenario, and matrix diagnostics include:

- stable diagnostic code;
- severity;
- source location when available;
- fixture, scenario, or matrix id;
- target profile;
- matrix axis;
- owning domain;
- source package;
- expected and actual diagnostic references;
- activation impact;
- suggested fix.

The first taxonomy covers unknown fixture references, unknown scenario
references, unsupported target profiles, missing data packages, denied
capabilities, stale evidence, incompatible matrix axes, invalid scenario steps,
expected diagnostic mismatches, accessibility requirement mismatches,
performance budget mismatches, and preview-only activation attempts.

## Implementation Row

No PM-008 implementation WR row is selected by this design action.

The next legal production-track action after this design is accepted is to add
or select one bounded WR row. That row should cover only the first generic
`domain/ui/ui_definition` fixture, scenario, target matrix, and evidence
descriptor contract slice and should not implement app-hosted Preview Lab UI,
visual screenshot capture, renderer golden comparison, provider sessions,
runtime replay, persistence activation, or production readiness.

## Required Fitness Functions

The first implementation row must add focused validation for:

- stable fixture, scenario, and matrix id preservation;
- target-profile compatibility diagnostics;
- fixture data-state coverage for empty, loading, error, denied, offline, heavy,
  and accessibility modes;
- denied capability and missing data package diagnostics;
- invalid scenario step diagnostics;
- matrix axis compatibility diagnostics;
- expected diagnostic mismatch diagnostics;
- preview-only activation rejection;
- editor/workbench and game-runtime target-profile examples without sharing
  runtime or provider ownership.

## Non-Goals

PM-008 design acceptance does not:

- implement app-hosted Preview Lab UI;
- implement visual capture, screenshot diffing, or renderer golden comparison;
- implement provider session orchestration;
- implement game-runtime replay loading;
- implement persistence activation from PM-UI-DESIGN-009;
- implement production readiness from PM-UI-DESIGN-010;
- move editor, gameplay, render, material, scene, asset, simulation, save-game,
  network, project, provider, app, runtime, or screenshot truth into generic UI
  fixture contracts.

## Acceptance Bar

PM-008 can move from `designing` to ready-next planning when:

- this accepted design exists;
- the production milestone points to this accepted design gate;
- production, roadmap, docs, and planning validators pass;
- a bounded WR row can be added or selected for the first generic fixture,
  scenario, and target matrix contract slice.
