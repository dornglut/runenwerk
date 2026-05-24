---
title: UI Lab Productization Design
description: Active productization design for the runtime-proven app-hosted Editor Interface Lab track.
status: active
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-24
related_adrs:
  - ../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/accepted/0006-editor-surface-provider-plugin-seam.md
  - ../../adr/accepted/0010-graph-substrate-canvas-boundary.md
  - ../../adr/accepted/0012-capability-workbench-clean-break.md
related_designs:
  - ./ui-designer-interface-lab-platform-design.md
  - ./editor-tool-suite-registry-and-workbench-host-design.md
  - ./editor-ui-workspace-tool-surface-architecture.md
  - ../implemented/editor-self-authoring-and-final-ui-design.md
  - ../implemented/surface-workflow-contract-redesign.md
  - ../accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../accepted/ui-designer-target-projection-profiles-design.md
  - ../accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md
  - ../accepted/ui-designer-component-surface-and-widget-recipe-library-design.md
  - ../accepted/ui-designer-view-model-capability-and-intent-binding-design.md
  - ../accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../accepted/ui-designer-persistence-migration-diff-and-activation-design.md
  - ../accepted/ui-designer-production-readiness-and-evidence-design.md
  - ../accepted/ui-lab-command-catalog-and-surface-registry-design.md
  - ../accepted/ui-lab-app-hosted-editor-lab-surface-shell-design.md
  - ../accepted/ui-lab-operation-driven-visual-authoring-design.md
  - ../accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md
  - ../accepted/ui-lab-preview-lab-runtime-evidence-design.md
  - ../accepted/ui-lab-api-docs-examples-runtime-closeout-design.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
  - ../../reports/roadmap-intake/2026-05-24-pt-ui-lab-runtime-proven-editor-interfac/proposal.yaml
---

# UI Lab Productization Design

## Status

This is the active productization design for `PT-UI-LAB`.

It does not reopen `PT-UI-DESIGN`. The completed `PT-UI-DESIGN` milestones are
design-contract input with bounded evidence. They are not runtime-proven proof
for an app-hosted Editor Lab.

This design authorizes planning, governance, and roadmap intake for
`PM-UI-LAB-001`. It does not authorize implementation of later milestones until
their WR rows, implementation contracts, validation, and closeout requirements
are accepted.

## Product Goal

`PT-UI-LAB` turns the completed UI Designer contracts into a real Editor
Interface Lab inside the editor application. The first product target is Editor
Lab V1, not the full game-runtime UI platform.

The lab must let authors inspect, edit, validate, preview, persist, diff,
apply, and recover authored editor/UI definitions through direct product UI. It
must prove behavior in the launched editor with runtime evidence rather than
only descriptors, docs, or retained preview data.

The target path is:

```text
PT-UI-DESIGN contracts
  -> code-truth reconciliation
  -> Editor Lab product contracts
  -> WR implementation rows
  -> app-hosted Editor Lab runtime evidence
  -> runtime_proven closeout
  -> separate no-gap audit intake
```

## Code-Truth Reconciliation

Current code already contains useful foundations, but the product experience is
not yet the planned Editor Interface Lab.

- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` and
  `apps/runenwerk_editor/src/shell/self_authoring.rs` provide self-authoring
  behavior through checked-in fixtures and explicit actions. This is a useful
  proving path, but it is not a normal visual lab workflow.
- `domain/editor/editor_shell/src/composition/build_self_authoring_control_panel.rs`
  exposes action/text control panels. Editor Lab V1 must replace normal author
  workflows with direct hierarchy, palette, canvas or preview, inspector, diff,
  diagnostics, and console controls.
- `apps/runenwerk_editor/src/shell/command_resolution.rs`,
  `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`,
  `domain/editor/editor_shell/src/composition/build_editor_shell.rs`, and
  `assets/editor/ui/editor_bindings.ron` currently split command identity,
  routing, toolbar/menu projection, keybinding projection, availability, and
  disabled reasons. Editor Lab V1 needs one command catalog source of truth.
- `domain/editor/editor_shell/src/workspace/state.rs`,
  `domain/editor/editor_shell/src/workspace/surface_contract.rs`, and
  `apps/runenwerk_editor/src/shell/providers/mod.rs` still carry legacy
  enum-centered surface registration and provider-family knowledge. Editor Lab
  must move toward registry-owned surface metadata while keeping legacy
  persistence compatibility explicit.
- `domain/ui/ui_definition/src/visual_layout/apply.rs` already provides a
  generic visual layout operation foundation. Editor-specific workspace, theme,
  menu, binding, and activation behavior must stay in editor-owned contracts
  and app-owned adapters.
- Public API ergonomics remain incomplete for normal users of
  `ui_definition` and `editor_definition`; examples and focused usage guides
  are part of the product closeout, not optional polish.

## Architecture Governance

The repository architecture-governance kickoff was run for this scope:

```text
PT-UI-LAB Editor Interface Lab productization from completed PT-UI-DESIGN
contracts into runtime-proven app-hosted Editor Lab
```

Governance findings for this design:

- DDD bounded context owner: `editor`, with generic UI definition mechanics
  owned by `domain/ui/ui_definition` and reusable editor-definition mechanics
  owned by `domain/editor/editor_definition`.
- App adapter owner: `apps/runenwerk_editor` owns concrete Editor Lab provider
  behavior, project IO, runtime activation, screenshot or evidence capture, and
  command execution bridges.
- Core vocabulary: Editor Lab, authored definition, target profile, surface,
  provider family, command catalog, visual operation, project package,
  activation report, rollback, runtime evidence, and closeout.
- Invariants: UI definitions describe UI/interface structure; they do not
  execute editor or game semantics. Projection output is derived state.
  Provider/runtime/app layers consume definitions and reports but do not become
  source truth.
- Translation boundaries: definition documents cross into editor shell through
  typed projection and operation contracts; app providers translate user
  interactions into typed proposals or commands; activation reports cross back
  as diagnostics and review data.
- Clean Architecture direction: `domain/ui/ui_definition` must not import app,
  shell-provider, project IO, renderer, or runtime execution concerns.
  `domain/editor/editor_shell` may host structural shell contracts, but
  semantic execution remains app/provider owned.
- ADR need: no new ADR is required for `PM-UI-LAB-001`; accepted ADRs already
  cover domain-owned commands, separation of description from execution,
  derived projections, provider seams, graph substrate boundaries, and
  Workbench clean-break direction. Later milestones must add or update an ADR
  if they change durable ownership, dependency direction, or cross-domain
  authority.
- ATAM-lite priority order: correctness and ownership first, runtime evidence
  second, author ergonomics third, migration compatibility fourth, performance
  and accessibility evidence fifth. A milestone may not trade ownership
  boundaries for short-term UI convenience.
- Team Topologies label: stream-aligned editor product work, supported by
  complicated-subsystem owners for UI definition, editor shell, runtime
  evidence, and public API review.
- Next legal action: update the design and production track, then promote
  disjoint WR candidates through roadmap review before code implementation.

## Product Contracts

`EditorCommandCatalog` is the single editor command source for labels,
availability, disabled reasons, menu projection, toolbar projection,
keybinding projection, palette/search projection, route targets, diagnostics,
and command dispatch proposals. Existing command-specific compatibility can
remain behind adapters while the catalog becomes the normal authoring path.

`ToolSurfaceDefinitionRegistry` is the single structural source for surface
identity, capability set, retention class, provider family, creation policy,
default workbench placement, and target-profile support. Legacy
`ToolSurfaceKind` and `PanelKind` may remain for migration and compatibility
but must not be the long-term extension point.

`EditorLabOperation` is a typed operation facade. It uses generic UI visual
layout operations for UI-definition layout edits and editor-owned commands for
workspace, theme, menu, binding, surface, and activation edits. It must record
stable ids, source locations, diagnostics, and reviewable diffs.

`EditorLabDocumentStore` is app-owned project IO for Editor Lab packages. It
owns load, save, import, export, migration, last-applied state, failed
activation preservation, and reload behavior.

`DefinitionApplyReview` is the review contract for diff, validation,
migration, activation, diagnostics, failure, rollback, and user-facing apply
results.

Focused public API entry points and usage examples are required for normal
`ui_definition` and `editor_definition` workflows. Broad glob exports are not
an ergonomic product API by themselves.

## Milestone Boundaries

`PM-UI-LAB-001` is design and governance only. It registers the track, records
this design, captures code-truth findings, and prepares WR candidates. It does
not edit runtime code.

`PM-UI-LAB-002` owns command catalog and surface registry source-of-truth
cleanup.

`PM-UI-LAB-003` owns the app-hosted Editor Lab shell and direct authoring
panels. Its accepted design is
`docs-site/src/content/docs/design/accepted/ui-lab-app-hosted-editor-lab-surface-shell-design.md`.

`PM-UI-LAB-004` owns operation-driven visual authoring, deterministic diffs,
history, validation, and diagnostics. Its accepted design is
`docs-site/src/content/docs/design/accepted/ui-lab-operation-driven-visual-authoring-design.md`.

`PM-UI-LAB-005` owns app project IO, diff/apply, activation reports, and
rollback. Its accepted design is
`docs-site/src/content/docs/design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md`.

`PM-UI-LAB-006` owns preview scenarios, screenshot or equivalent visual
evidence, diagnostics snapshots, accessibility checks, performance evidence,
and runtime-proof artifacts. Its accepted design is
`docs-site/src/content/docs/design/accepted/ui-lab-preview-lab-runtime-evidence-design.md`.

`PM-UI-LAB-007` owns public API ergonomics, usage docs, examples, final
runtime-proven closeout, and intake for the later no-gap audit.

Game-runtime UI projection execution is out of scope for Editor Lab V1 until
the editor/workbench target is runtime-proven.

## WR Candidate Set

The generated roadmap intake seed is:

```text
docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-pt-ui-lab-runtime-proven-editor-interfac/proposal.yaml
```

Roadmap review should split or promote it into these disjoint implementation
rows:

| Candidate | Milestone | Primary write scope | Runtime evidence |
|---|---|---|---|
| UI Lab command catalog and surface registry | PM-UI-LAB-002 | `apps/runenwerk_editor/src/shell`, `domain/editor/editor_shell/src/{tool_suite,workspace,composition}`, editor bindings assets | Menu, toolbar, keybinding, palette, routing, disabled states, and unavailable-command diagnostics derive from one catalog. |
| Editor Lab shell product surface | PM-UI-LAB-003 | `apps/runenwerk_editor/src/shell/providers`, `domain/editor/editor_shell/src/composition`, editor workspace fixtures | Launched editor shows hierarchy, palette, canvas or preview, inspector, diff, diagnostics, and console workflows without debug action lists. |
| Visual authoring operations | PM-UI-LAB-004 | `domain/ui/ui_definition`, `domain/editor/editor_definition`, editor app adapters | Canvas, hierarchy, and inspector edits round-trip through typed operations with deterministic diffs and undo/redo. |
| Project IO and activation review | PM-UI-LAB-005 | `apps/runenwerk_editor`, editor project package docs and fixtures | Save, reload, import, export, diff, apply, reject, failed activation preservation, and rollback are proven in app. |
| Preview and evidence capture | PM-UI-LAB-006 | editor preview/evidence harness, closeout artifact folders, diagnostics fixtures | Success, warning, error, reload, apply, rollback, degraded-provider, accessibility, and performance states have captured artifacts. |
| Public API and closeout ergonomics | PM-UI-LAB-007 | `domain/ui/ui_definition`, `domain/editor/editor_definition`, docs and examples | Usage examples compile or run, docs validate, public API review passes, and runtime-proven closeout links all evidence. |

Each promoted WR row must name exact dependencies, write scopes, validation
commands, closeout artifact paths, and stop conditions before implementation.

## Runtime-Proven Acceptance

No `PT-UI-LAB` milestone may claim runtime_proven unless closeout evidence
includes:

- launched editor behavior, not only data descriptors;
- screenshots or equivalent visual artifacts for supported visible states;
- command, provider, validation, persistence, activation, and rollback
  diagnostics;
- focused automated tests for owned contracts;
- docs validation and production-track validation;
- explicit known gaps when an expected check is unsupported by the current
  runtime.

`perfectionist_verified` is not available to this track. The final milestone
must open a separate no-gap audit intake if runtime-proven evidence is strong
enough to justify it.

## Stop Conditions

Stop before implementation if a milestone would:

- require `ui_definition` to execute editor, game, app, project IO, or runtime
  behavior;
- make provider output or retained preview data source truth;
- bypass WR roadmap review or production implementation contracts;
- claim runtime proof from descriptors, docs, static fixtures, or status panels
  alone;
- add game-runtime UI projection execution before Editor Lab V1 is proven;
- require durable ownership or dependency changes without an accepted ADR or
  design update.
