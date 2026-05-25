---
title: UI Lab Direct Manipulation UX Closure Design
description: Accepted design for PM-UI-LAB-PERF-004 direct-manipulation Editor Lab UX closure after command/surface source-truth completion.
status: accepted
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-25
related_adrs:
  - ../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/accepted/0006-editor-surface-provider-plugin-seam.md
  - ../../adr/accepted/0010-graph-substrate-canvas-boundary.md
  - ../../adr/accepted/0012-capability-workbench-clean-break.md
related_designs:
  - ./ui-lab-perfectionist-audit-design.md
  - ./ui-lab-operation-driven-visual-authoring-design.md
  - ./ui-lab-command-surface-source-truth-closure-design.md
  - ./ui-lab-runtime-evidence-platform-closure-design.md
  - ./ui-lab-app-hosted-editor-lab-surface-shell-design.md
  - ./ui-designer-visual-layout-and-interface-composition-design.md
  - ./ui-designer-view-model-capability-and-intent-binding-design.md
related_reports:
  - ../../reports/closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md
  - ../../reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# UI Lab Direct Manipulation UX Closure Design

## Status

Accepted for `PM-UI-LAB-PERF-004`.

This design clears only the direct-manipulation UX closure design gate. It does
not authorize product code until a linked WR row is selected, `task
production:plan` creates a decision-complete implementation contract, and the
roadmap gates allow implementation.

## Goal

Editor Lab V1 already has app-hosted surfaces, operation-driven mutation,
typed runtime evidence, and command/surface source-truth audits. The remaining
PM004 no-gap question is whether a normal author can complete the core Editor
Lab workflow through direct product surfaces without relying on debug action
lists, text-only status panels, provider implementation knowledge, or
console-only feedback.

The target workflow is:

```text
author action in hierarchy, palette, canvas, inspector, diagnostics, or review
  -> visible command affordance with typed disabled reason when blocked
  -> EditorLabOperation or session selection intent
  -> deterministic operation report and diagnostics
  -> visible preview, inspector, operation diff, and history refresh
  -> runtime evidence artifact for apply, undo, redo, and failure states
```

## Current Code Truth

Completed inputs:

- `PM-UI-LAB-PERF-002` added typed captured-or-platform-impossible runtime
  evidence results.
- `PM-UI-LAB-PERF-003` completed catalog and registry source-truth audits.
- `PM-UI-LAB-004` and `WR-096` already implemented typed
  `EditorLabOperation` contracts, deterministic diffs, diagnostics,
  app-owned operation history, undo, redo, and retained preview refresh.

Remaining no-gap UX blockers:

- hierarchy, palette, canvas, inspector, diagnostics, operation diff, preview
  console, undo, and redo must be proven as one normal author workflow;
- normal authoring must not depend on action-list buttons, text-only review,
  console spelunking, or retained debug strings as the only evidence of state;
- operation affordances must expose typed disabled reasons before dispatch;
- invalid operations must keep draft state, keep selection understandable, and
  surface diagnostics in the product UI;
- supported canvas, hierarchy, and inspector entry points must produce the
  same operation report shape for equivalent edits;
- runtime evidence must inspect product surfaces, not only provider internals.

## Architecture Governance

Architecture governance for this design-only action:

```text
task ai:architecture-governance -- --task "PM-UI-LAB-PERF-004 direct manipulation Editor Lab UX closure design" --scope "Editor Lab hierarchy, palette, canvas, inspector, operation diff, diagnostics, preview console, undo, redo, and runtime evidence UX closure; design-only action, no product code"
```

Governance decisions:

- DDD owner: the `editor` bounded context owns Editor Lab authoring UX,
  operation review, diagnostics, history, undo/redo presentation, and runtime
  proof.
- Supporting owner: `domain/ui/ui_definition` owns generic, behavior-free UI
  visual layout operations and target projection contracts only.
- Supporting owner: `domain/editor/editor_definition` owns editor definition
  document vocabulary, operation contracts, validation, and deterministic
  diffs.
- Supporting owner: `domain/editor/editor_shell` owns app-neutral surface view
  models, retained composition helpers, widget/action routing contracts, and
  product-surface projection.
- App owner: `apps/runenwerk_editor` owns concrete draft mutation, operation
  dispatch, provider execution, session history, preview refresh, diagnostics
  capture, artifact writing, and native/runtime evidence probes.
- Clean Architecture direction: domain crates may define contracts and derived
  reports; app runtime owns execution and mutable session state. Generic UI
  crates must not gain editor command semantics or provider behavior.
- ADR need: no new ADR is required while PM004 preserves existing
  description/execution, derived projection, provider seam, graph canvas, and
  capability workbench decisions. Add an ADR or accepted design update before
  creating a global operation engine, moving app draft mutation into
  `ui_definition`, treating retained previews as source truth, or reusing
  scene-runtime history as Editor Lab definition history.
- ATAM-lite priority order: ownership correctness first, product workflow
  completeness second, deterministic diagnostics third, runtime evidence
  fourth, breadth of supported widgets fifth.
- Ownership mode: stream-aligned editor product work with
  complicated-subsystem support from UI definition, editor definition, and
  editor shell owners.

## Direct-Manipulation Contract

PM004 closes only workflows that are already in Editor Lab V1 ownership:

- hierarchy selection and ordering for authored UI nodes;
- palette insertion for supported node families;
- canvas selection and direct edit affordances for supported UI elements;
- inspector field edits for supported text, value, theme, and workspace
  properties;
- operation diff review for accepted, rejected, undo, and redo states;
- diagnostics review for invalid operations and degraded-provider states;
- preview console feedback scoped to the selected operation or document;
- undo and redo controls that update history, draft state, and preview or
  inspector output.

Normal PM004 controls must route through existing command catalog, surface
registry, and operation boundaries. They must not create a second command
catalog, second operation history, or second surface metadata path.

## Evidence Matrix

The implementation closeout may claim `runtime_proven` only when runtime
evidence proves all of these states through product surfaces:

| Evidence target | Required proof |
|---|---|
| Hierarchy workflow | Select an authored node, perform a supported operation, and show operation report plus selection refresh. |
| Palette workflow | Insert a supported authored node or explicitly fail closed with typed disabled reason and diagnostic. |
| Canvas workflow | Select or edit through canvas affordance and prove it reaches the same operation report shape as hierarchy/inspector. |
| Inspector workflow | Edit a supported field, produce deterministic diff, and refresh retained preview or structured inspector state. |
| Diagnostics workflow | Rejected operation preserves draft state and renders typed diagnostics in the Editor Lab surface. |
| Operation diff workflow | Accepted, rejected, undo, and redo states are visible without reading console logs. |
| Preview console workflow | App-owned console feedback is visible when relevant but is not the only operation proof. |
| Undo/redo workflow | Undo and redo update draft state, operation history, preview/inspector state, and availability. |

Valid evidence may use retained product-surface debug artifacts where native
screenshots are unavailable, but it must name the surface, route, operation id,
diagnostics, and artifact path. PM004 must not rely on descriptor-only proof.

## Implementation Shape

Use a Strangler migration over the existing PM-UI-LAB-004 operation path:

1. audit the current Editor Lab surfaces for normal author workflows that still
   depend on action-list or text-only debug presentation;
2. route the smallest useful hierarchy, palette, canvas, and inspector
   affordances through the existing `EditorLabOperation` report path;
3. expose disabled reasons and diagnostics in the product surface before and
   after dispatch;
4. make operation diff, history, undo, redo, and preview refresh visible in one
   product workflow;
5. keep compatibility controls private or explicitly marked as debug-only
   until the product path proves parity;
6. write runtime evidence that exercises normal controls rather than provider
   helpers alone.

The implementation should split modules only when stable responsibilities
emerge. Acceptable future subdomains include `self_authoring/operations`,
`self_authoring/history`, `self_authoring/selection`, or Editor Lab surface
composition modules. Do not add catch-all `utils`, `helpers`, or `_internal`
modules.

## Required Fitness Functions

The linked implementation WR must include focused validation for:

```text
cargo fmt
cargo test -p ui_definition visual_layout
cargo test -p editor_definition operation
cargo test -p editor_shell editor_lab
cargo test -p runenwerk_editor editor_lab_operation
cargo test -p runenwerk_editor direct_manipulation
cargo test -p runenwerk_editor pm_ui_lab_perf_004
task docs:validate
task puml:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
git diff --check
```

Tests must prove:

- product-surface controls dispatch operation reports, not direct draft
  mutation shortcuts;
- equivalent hierarchy, canvas, and inspector edits produce compatible report
  shapes;
- disabled controls expose typed reasons before dispatch;
- rejected operations preserve draft state and surface diagnostics;
- undo and redo update history availability and product surfaces;
- preview or inspector refresh is visible after accepted, undo, and redo
  paths;
- `ui_definition` remains behavior-free;
- project IO, persisted operation logs, public API examples, final no-gap
  certification, and game-runtime UI projection stay out of PM004 scope.

## Roadmap Candidate

The implementation row must be created through roadmap intake after this design
gate. The intake should assign the next available WR id instead of reusing the
older placeholder ids from PM001 planning notes.

Primary write scopes should include:

- `domain/editor/editor_definition/src/operation.rs`
- `domain/editor/editor_definition/src/workflow.rs`
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs`
- `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
- `apps/runenwerk_editor/src/shell/self_authoring`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/tests.rs`
- PM004 implementation plan, closeout, roadmap, production, and generated
  planning docs.

## Non-Goals

PM004 does not implement:

- project packages, save/load/import/export, persisted operation logs,
  structural diff/apply productization, failed activation preservation, or
  rollback review owned by `PM-UI-LAB-PERF-005`;
- public API ergonomics, preludes, guides, or examples owned by
  `PM-UI-LAB-PERF-005`;
- final no-gap certification or `perfectionist_verified` owned by
  `PM-UI-LAB-PERF-006`;
- game-runtime UI projection;
- a global operation bus or global mutable registry;
- editor execution behavior in `domain/ui/ui_definition`.

## Stop Conditions

Stop before implementation if PM004 would:

- require `ui_definition` to own editor semantics, app history, command
  execution, provider behavior, or runtime preview refresh;
- treat retained preview output, provider frames, screenshots, or console text
  as source truth;
- reuse scene-runtime history as authoritative Editor Lab definition history;
- skip operation reports for normal supported authoring controls;
- broaden into PM005 persistence/API or PM006 certification scope.
