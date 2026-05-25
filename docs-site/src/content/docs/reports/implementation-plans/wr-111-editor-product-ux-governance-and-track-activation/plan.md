---
title: WR-111 Editor Product UX Governance And Track Activation
description: Design-first implementation contract for activating PT-EDITOR-UX and its native Story Lab, all-surface certification, and game-UI-ready foundation milestones.
status: active
owner: editor
layer: workspace/domain/app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-story-lab-and-game-ui-ready-foundations-design.md
  - ../../../design/accepted/ui-lab-perfectionist-audit-design.md
  - ../../../design/active/ui-designer-interface-lab-platform-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# WR-111 Editor Product UX Governance And Track Activation

## Goal

Execute `PM-EDITOR-UX-001` / `WR-111` as a governance and track-activation
slice. This row creates `PT-EDITOR-UX`, anchors the active editor product UX
design, records source-truth boundaries, captures the current code-truth audit,
and decomposes the remaining editor UX perfection program into bounded
milestones and WR labels.

No Rust, app, renderer, shader, or product runtime code is in scope for
`WR-111`.

## Architecture Governance Result

Architecture governance for this slice classifies the work as editor product
governance with these owners:

- DDD bounded context owner: `editor`.
- Generic UI support owners: `domain/ui/ui_theme`,
  `domain/ui/ui_definition`, `domain/ui/ui_tree`, `domain/ui/ui_widgets`,
  `domain/ui/ui_runtime`, and `domain/ui/ui_graph_editor`.
- Editor product owners: `domain/editor/editor_definition` and
  `domain/editor/editor_shell`.
- Native evidence and concrete app workflow owner: `apps/runenwerk_editor`.
- Planning and generated evidence owner: workspace docs under
  `docs-site/src/content/docs/workspace` and `docs-site/src/content/docs/reports`.

Dependency direction remains:

```text
foundation -> domain/ui + domain/editor -> engine/runtime -> apps/tools
```

`domain/ui` must not depend on editor shell, app providers, native screenshots,
runtime sessions, renderer handles, or game UI semantics. `apps/runenwerk_editor`
may execute native evidence but must not own generic UI truth. Future
game-runtime UI remains owned by `PT-GAME-RUNTIME-UI` and must not import editor
shell ownership.

ADR decision: no ADR is required for this governance row. Require an ADR or
accepted design update before any follow-on milestone changes dependency
direction, source-truth authority, persistent public formats, app/runtime
evidence ownership, or game-runtime UI ownership.

ATAM-lite priority order:

1. ownership and source-truth correctness;
2. fail-closed diagnostics and explicit readiness;
3. native reproducible evidence;
4. product author ergonomics and visual polish;
5. performance and long-term maintainability.

Team Topologies label: stream-aligned editor product work with
complicated-subsystem support from UI definition, UI runtime, graph editor,
editor shell, and app evidence owners.

## Source Of Truth

- Production track:
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap sources:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`,
  `docs-site/src/content/docs/workspace/roadmap-archive.yaml`, and
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml`.
- Active design:
  `docs-site/src/content/docs/design/active/editor-product-ux-story-lab-and-game-ui-ready-foundations-design.md`.
- Completed upstream UI Lab perfection input:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-006-final-no-gap-certification-closeout/closeout.md`.
- UI Designer target-profile and readiness input:
  `docs-site/src/content/docs/design/accepted/ui-designer-target-projection-profiles-design.md` and
  `docs-site/src/content/docs/design/accepted/ui-designer-production-readiness-and-evidence-design.md`.

## Required Changes

- Add `PT-EDITOR-UX` with milestones `PM-EDITOR-UX-001` through
  `PM-EDITOR-UX-009`.
- Mark `PM-EDITOR-UX-001` complete as bounded governance evidence only.
- Keep `PM-EDITOR-UX-002` through `PM-EDITOR-UX-009` designing and dependent
  on prior milestones.
- Add `WR-111` as completed governance evidence in the roadmap archive.
- Add `WR-112` through `WR-119` as blocked-deferred follow-on labels so the
  production milestones have stable future roadmap anchors without granting
  implementation permission.
- Add active design and design index link.
- Add this contract and the PM001 closeout.
- Render production and roadmap generated docs after the source changes.

## Follow-On Milestone Contracts

The follow-on work is intentionally one legal slice at a time:

- `WR-112`: native Editor UX Story Lab and evidence harness.
- `WR-113`: layered design system migration.
- `WR-114`: standalone UI Designer workbench.
- `WR-115`: graph canvas/node editor productization.
- `WR-116`: shell and product pattern polish.
- `WR-117`: all registered visible surface wave.
- `WR-118`: game UI readiness seam.
- `WR-119`: final local-native no-gap certification.

Each follow-on WR must get its own fresh `task production:plan` contract before
code changes. `WR-111` does not authorize product implementation.

## Code-Truth Matrix

| Area | Current repo truth | Track blocker | Owning future milestone |
|---|---|---|---|
| Surface registry | `domain/editor/editor_shell/src/workspace/surface_contract.rs` registers core, UI Designer, asset/import, field/SDF, material, texture, procgen, gameplay, particle, physics, animation, simulation, and diagnostics surfaces. | Registered visible surfaces need explicit readiness: product, fallback-only, diagnostic, or hidden. | `PM-EDITOR-UX-007` |
| Primitive widgets | `domain/ui/ui_tree/src/tree/node.rs` and `domain/ui/ui_widgets/src` expose retained node kinds and constructors. | Primitive widgets need story coverage, state matrices, visible-widget scans, and native evidence. | `PM-EDITOR-UX-002`, `PM-EDITOR-UX-003` |
| Evidence substrate | `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs` already contains retained, screenshot, visual diff, focus, contrast, timing, provider, diagnostics, accessibility, performance, platform-impossible, unsupported-check, and manifest vocabulary. | Editor product UX needs a Story Lab catalog and local-native final evidence policy over this substrate. | `PM-EDITOR-UX-002`, `PM-EDITOR-UX-009` |
| UI Designer | UI Designer contracts exist in `domain/ui` and `domain/editor`; editor-design profile/surfaces exist in shell/workbench code. | The product-grade standalone UI Designer workbench still needs canvas, hierarchy, inspector, property editing, scenario matrix, and native proof. | `PM-EDITOR-UX-004` |
| Graph canvases | Material graph has real graph projection. SDF/procgen/gameplay/particle/animation graph surfaces are registered but need hide-or-certify product policy. | Node editor UX must be productized with palettes, node cards, sockets, links, selection, drag, marquee, shortcuts, overlays, diagnostics, and dense-graph evidence. | `PM-EDITOR-UX-005` |
| Shell/product patterns | Shell, inspector, palette, diagnostics, preview, tables, trees, tabs, toolbar/status, split/dock patterns exist across editor shell and app code. | Product patterns need reusable adapters, state coverage, focus/keyboard coverage, overflow policy, and hard zero budgets. | `PM-EDITOR-UX-006` |
| Game UI readiness | `PT-GAME-RUNTIME-UI` owns actual runtime HUD/platform work. UI Designer target profiles already include `game.runtime`. | Editor UX must prove compatibility axes only; no editor-shell coupling and no game UI implementation. | `PM-EDITOR-UX-008` |

## Evidence Matrix

| Evidence target | Minimum acceptable proof | Invalid proof |
|---|---|---|
| Story catalog | Typed story IDs, kind, args, interactions, fixtures, scenario matrix, expected diagnostics, and evidence manifest. | Ad hoc screenshot commands with no reusable catalog. |
| Visible-widget scan | Per-story widget tree report with layout bounds, label/accessibility, focus reachability, overflow behavior, and state coverage. | Human review notes without widget identity. |
| Native screenshot | Local-native capture artifact or explicit platform-impossible diagnostic only when the runtime cannot perform capture. | Retained-preview text used as pixel proof. |
| Product surface readiness | `ToolSurfaceReadiness` classification for every registered visible surface and explicit diagnostic/fallback surface. | Visible product-shaped placeholder surfaces. |
| Graph canvas UX | Dense graph, selection, drag, link, palette, overlay, diagnostic, keyboard, and degraded-state evidence. | Text/action panels titled "Graph Canvas". |
| Game UI seam | Target-profile compatibility evidence for safe area, input modality, fixture, evidence descriptor, and readiness axes. | Editor shell or provider vocabulary imported into game-runtime UI. |

## Non-Goals

- Do not implement `EditorUxStoryCatalog` or product code in `WR-111`.
- Do not create or edit `domain/`, `apps/`, `engine/`, shader, benchmark, or
  runtime code in this row.
- Do not mark any follow-on milestone complete.
- Do not reopen completed `PT-UI-LAB` or implement `PT-GAME-RUNTIME-UI`.
- Do not claim `runtime_proven` or `perfectionist_verified` from governance.

## Validation

Run after source edits:

```text
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task planning:validate
task puml:validate
task docs:validate
git diff --check
```

`./quiet_full_gate.sh` is not required for this docs-only row because no Rust,
app, engine, shader, or runtime behavior changes.

## Closeout Requirements

`WR-111` can close only when:

- `PT-EDITOR-UX` exists with nine ordered milestones.
- `PM-EDITOR-UX-001` is completed with bounded governance evidence.
- `WR-111` is archived with a completed closeout path in its write scopes.
- `WR-112` through `WR-119` are present only as follow-on blocked/deferred
  labels, not implementation permission.
- The active design records source-truth, Story Lab, readiness, evidence,
  game-UI seam, quality gate, and stop-condition doctrine.
- Generated production and roadmap docs are rendered and validation passes.
