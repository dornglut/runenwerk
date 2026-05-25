---
title: WR-115 Graph Canvas And Node Editor Productization Contract
description: Current-candidate implementation contract for PM-EDITOR-UX-005 graph canvas and node editor productization across generic graph contracts, editor product surfaces, and app-owned native evidence.
status: active
owner: editor
layer: domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
related_reports:
  - ../wr-114-standalone-ui-designer-workbench/plan.md
  - ../../closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-115 Graph Canvas And Node Editor Productization Contract

## Goal

Prepare `PM-EDITOR-UX-005` for a bounded current-candidate implementation of
graph canvas and node editor productization.

This contract is produced from:

```text
task production:plan -- --milestone PM-EDITOR-UX-005 --roadmap WR-115
```

This document is the implementation contract. It verifies that `WR-115` is the
legal `current_candidate` row, names the exact ownership and evidence chain, and
keeps graph product code blocked until this contract validates and a later
coordinator action starts implementation after rerunning `task ai:goal -- --track
PT-EDITOR-UX`.

Expected production outcome:

- material graph UX has product-grade canvas, palette, nodes, sockets, links,
  selection, drag, marquee, shortcuts, overlays, diagnostics, dense-graph, and
  degraded-provider evidence;
- SDF, procgen, gameplay, particle, and animation graph surfaces are either
  productized, explicit fallback/diagnostic surfaces, or hidden until
  productized;
- generic graph interaction truth remains in `domain/ui/ui_graph_editor`;
- material, SDF, procgen, gameplay, particle, and animation semantics remain
  owned by their domain crates and editor adapters, not by the graph widget;
- native execution, provider fixtures, screenshots or platform-impossible
  reports, and evidence manifests remain app-owned in `apps/runenwerk_editor`;
- text/action panels titled as graph canvases cannot satisfy product graph
  evidence.

## Source Of Truth

- Production milestone: `PM-EDITOR-UX-005` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-115` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Active editor UX doctrine:
  `docs-site/src/content/docs/design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md`.
- Active tool-suite/workbench doctrine:
  `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`.
- Active procedural/SDF graph workflow doctrine:
  `docs-site/src/content/docs/design/active/editor-procedural-content-and-simulation-workflow-plan.md`.
- Active asset/material workflow doctrine:
  `docs-site/src/content/docs/design/active/editor-asset-pipeline-and-content-workflow-design.md`.
- Completed prerequisites:
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-002-native-editor-ux-story-lab-and-evidence-harness/closeout.md`,
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md`, and
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md`.
- Generic graph substrate:
  `domain/ui/ui_graph_editor/src/lib.rs` module `ui_graph_editor`.
- Material graph product surface:
  `domain/editor/editor_shell/src/composition/build_material_graph_surface.rs`
  function `build_material_graph_surface`,
  `domain/editor/editor_shell/src/surfaces/material.rs` module `material`, and
  `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs` module
  `material_graph_canvas`.
- Registered graph readiness and future-family policy:
  `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `tool_surface_readiness` and
  `tool_surface_readiness_for_definition_id`.
- Current fallback graph providers:
  `apps/runenwerk_editor/src/shell/providers/sdf_graph_canvas.rs` module
  `sdf_graph_canvas` and
  `apps/runenwerk_editor/src/shell/providers/procgen_graph_canvas.rs` module
  `procgen_graph_canvas`.
- App-owned Story Lab evidence:
  `domain/editor/editor_shell/src/story_lab` module `story_lab` and
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab` module
  `editor_ux_story_lab`.

## Readiness

`task production:plan -- --milestone PM-EDITOR-UX-005 --roadmap WR-115`
currently reports:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-112:completed, WR-113:completed
Next action: write_implementation_contract
```

Implementation planning is honest because:

- `WR-112` is completed with native Story Lab, visible-widget scan, and
  app-owned manifest evidence;
- `WR-113` is completed with runtime-proven token/recipe/state Story Lab
  evidence;
- `WR-114` is completed with runtime-proven standalone UI Designer workbench
  evidence;
- `PM-EDITOR-UX-005` is `ready_next` and links `WR-115`;
- `WR-115` is `current_candidate`, dependency-legal, and uses this active
  WR-115 contract as its implementation gate;
- the promotion action completed through `task roadmap:promote` with accepted
  evidence and post-promotion roadmap/production validation;
- the current material graph surface already has a typed graph canvas product
  path that can be hardened instead of replaced;
- SDF/procgen and future graph families still need a product, explicit
  fallback/diagnostic, or hidden readiness policy before implementation.

This action is still contract-only. The next coordinator run must rerun
`task ai:goal -- --track PT-EDITOR-UX` after validation before making product
code changes.

## Architecture Governance Review

Recommendation: execute a bounded vertical graph implementation only after this
contract validates and `task ai:goal -- --track PT-EDITOR-UX` still selects
`PM-EDITOR-UX-005` and `WR-115`.

Owner:

- Generic graph editor interaction, hit testing, selection, marquee, drag,
  link, overlay, and viewport/pan/zoom contracts belong to
  `domain/ui/ui_graph_editor`.
- Editor graph product semantics, surface readiness, Story Lab scenario
  metadata, and adapters belong to `domain/editor`.
- Material, SDF, procgen, gameplay, particle, and animation graph meaning stays
  in the owning domain/editor adapters. `ui_graph_editor` must not know those
  vocabularies.
- Provider execution, app fixtures, screenshots or platform-impossible reports,
  and evidence manifests belong to `apps/runenwerk_editor`.

Dependency direction:

```text
domain/ui/ui_graph_editor -> domain/editor/editor_shell -> apps/runenwerk_editor
```

`domain/ui/ui_graph_editor` must not import material, SDF, procgen, gameplay,
particle, animation, editor shell, app provider, or runtime preview code.
`domain/editor` may adapt generic graph contracts into product graph surfaces.
`apps/runenwerk_editor` may host and prove graph surfaces, but it must not own
generic graph interaction truth or domain graph semantics.

ADR need: no new ADR is required while implementation preserves generic graph
substrate ownership, domain semantic ownership, app-owned evidence, and
hide-or-certify readiness. Require an ADR or accepted design update before
making `ui_graph_editor` own material/SDF/procgen/gameplay semantics, changing
dependency direction, or claiming future gameplay/particle/animation graph
productization without accepted domain product designs.

ATAM-lite:

- Quality attributes in tension: graph interaction depth, domain ownership,
  native evidence, dense-graph performance, and hide-or-certify honesty.
- Chosen option: harden material graph as the first product graph while using
  explicit fallback/diagnostic or hidden readiness for graph families that do
  not yet have product-grade node-editor semantics.
- Sensitivity points: graph-looking text panels, generic graph widgets that
  know domain vocabulary, provider-only line dumps, material-only proof
  generalized to unfinished graph families, and Story Lab stories without drag,
  selection, link, keyboard, overlay, dense, or degraded states.
- Risk: registered future graph surfaces can appear in product chrome as
  product-shaped placeholders.
- Non-risk: keeping native evidence app-owned, because the active editor UX
  doctrine assigns evidence execution to the app.

Migration shape: use a Strangler Fig migration. Keep current fallback graph
providers available only as explicit fallback/diagnostic paths, harden the
material graph product path first, add Story Lab evidence and readiness guards,
then promote other graph families only when they have product-grade contracts.

Fitness functions:

- `cargo test -p ui_graph_editor` for generic interaction, hit testing,
  selection, drag, marquee, link, overlay, and dense-graph contracts;
- `cargo test -p editor_shell graph` for editor graph view models, routes,
  surface readiness, and hide-or-certify classification;
- `cargo test -p runenwerk_editor graph` for material graph provider evidence,
  app fallback classification, and native Story Lab manifest proof;
- planning validators for docs, roadmap, production, and PUML after metadata
  changes.

Ownership mode: complicated-subsystem graph substrate support for stream-aligned
editor product owners.

## Critical Review Gate

Source truth:

- `domain/ui/ui_graph_editor/src/lib.rs` is source truth only for generic graph
  interaction contracts and geometry. It is not source truth for material, SDF,
  procgen, gameplay, particle, or animation graph semantics.
- `domain/editor/editor_shell/src/surfaces/material.rs` and
  `domain/editor/editor_shell/src/composition/build_material_graph_surface.rs`
  own editor product view models and retained composition for the material
  graph surface.
- `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs`,
  `sdf_graph_canvas.rs`, and `procgen_graph_canvas.rs` are provider/runtime
  consumers, not generic graph source truth.
- Story Lab manifests, screenshots, retained debug output, and timing reports
  are runtime proof, not graph semantic source truth.

Required source-to-runtime chain:

```text
Domain graph document or projection
  -> editor-owned graph product view model
  -> generic ui_graph_editor interaction contracts
  -> retained graph canvas, palette, inspector, overlay, diagnostics, and shortcut routes
  -> app-owned provider frame execution
  -> Editor UX Story Lab graph scenario/state matrix
  -> native screenshot or typed platform-impossible artifact
  -> visible-widget, focus, interaction, diagnostics, dense-graph, degraded-provider, and timing evidence
```

The implementation must not stop at descriptor registration, text/action
control panels, status rows, provider line dumps, or graph labels. A graph
surface is productized only when tests prove typed graph canvas interaction and
native/app-owned evidence. A future graph family is acceptable only if it is
productized to that bar, explicitly fallback/diagnostic, or hidden until
productized.

Forbidden fallbacks:

- text/action panels titled "Graph Canvas" used as product proof;
- generic graph contracts importing material, SDF, procgen, gameplay,
  particle, or animation vocabulary;
- graph providers claiming product status without Story Lab graph evidence;
- registered future graph surfaces visible as product-shaped placeholders;
- material graph evidence used to complete SDF/procgen/gameplay/particle or
  animation graph UX.

Architecture guard tests must prevent descriptor-only, status-panel-only,
fallback-only, provider-line-only, material-only, and unconsumed-contract
completion claims.

Expected completion quality is `runtime_proven` if native Story Lab manifest
evidence proves at least the productized material graph path and every other
registered graph family is product, explicit fallback/diagnostic, or hidden. Use
`bounded_contract` if implementation lands only a narrower graph contract slice.
`perfectionist_verified` is forbidden for `PM-EDITOR-UX-005`.

## Implementation Scope

The future implementation slice should productize graph UX and hide-or-certify
unfinished graph families, not perform final shell-wide polish.

Expected generic UI work:

- `domain/ui/ui_graph_editor/src/lib.rs` module `ui_graph_editor`: extend or
  reuse graph hit testing, selection, drag, marquee, link, overlay, keyboard,
  dense graph, and degraded-state contracts without domain-specific semantics.

Expected editor domain work:

- `domain/editor/editor_shell/src/composition/build_material_graph_surface.rs`
  function `build_material_graph_surface`: harden material graph product
  composition for palette, node cards, sockets, links, selection, drag,
  marquee, shortcuts, overlays, diagnostics, dense graph, and degraded states.
- `domain/editor/editor_shell/src/surfaces/material.rs` module `material`: keep
  material graph view models typed and source-backed.
- `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `tool_surface_readiness` and
  `tool_surface_readiness_for_definition_id`: classify unfinished graph
  families as product, fallback-only/diagnostic, or hidden.
- `domain/editor/editor_shell/src/story_lab` module `story_lab`: add graph
  stories and scenario matrices for material graph, dense graph, diagnostic
  overlays, keyboard/focus, link/selection/drag, degraded provider, and hidden
  future graph families.

Expected app work after promotion:

- `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs` module
  `material_graph_canvas`: prove material graph provider frames through native
  Story Lab evidence.
- `apps/runenwerk_editor/src/shell/providers/sdf_graph_canvas.rs` and
  `procgen_graph_canvas.rs`: either productize with typed graph view models or
  expose explicit fallback/diagnostic/hide policy.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab` module
  `editor_ux_story_lab`: require graph stories to emit retained UI, native or
  platform-impossible proof, interaction/focus/accessibility/diagnostics/timing
  reports, and graph-specific evidence.

## Non-Goals

- Do not implement product code in this implementation-contract action.
- Do not claim shell-wide pattern polish; that remains `PM-EDITOR-UX-006`.
- Do not claim all-surface certification; that remains `PM-EDITOR-UX-007`.
- Do not implement game-runtime HUD or gameplay graph product semantics without
  their owning accepted designs.
- Do not move material, SDF, procgen, gameplay, particle, or animation graph
  semantics into `domain/ui/ui_graph_editor`.
- Do not treat text/action fallback panels as product graph canvases.

## Acceptance Criteria

- The implementation contract names product graph owners, fallback policy,
  graph family readiness decisions, and the exact implementation boundary before
  product code starts.
- Material graph productization proves canvas, palette, nodes, sockets, links,
  selection, drag, marquee, shortcuts, overlays, diagnostics, dense graph, and
  degraded-provider states.
- SDF/procgen/gameplay/particle/animation graph surfaces are productized,
  explicit fallback/diagnostic, or hidden until productized.
- App-owned evidence manifests fail graph product claims that lack native or
  typed platform-impossible proof, interaction/focus/accessibility diagnostics,
  timing, visible-widget scans, and graph-specific evidence.
- Future game-runtime UI compatibility remains at target-profile and evidence
  descriptor seams only.

## Implementation Steps

1. Rerun
   `task production:plan -- --milestone PM-EDITOR-UX-005 --roadmap WR-115`
   and confirm `WR-115` remains `current_candidate` with next action
   `write_implementation_contract`.
2. Validate this implementation contract, then rerun
   `task ai:goal -- --track PT-EDITOR-UX`.
   Do not start product code in the same contract-only action.
3. In the implementation action, inspect `ui_graph_editor`, material graph
   surface composition, graph providers, surface readiness, and Story Lab
   evidence modules before code changes.
4. Add the smallest vertical product graph chain that proves material graph
   canvas interaction and evidence while applying hide-or-certify policy to
   unfinished graph families.
5. Keep fallback graph providers explicit and temporary.
6. Run focused validation, write closeout evidence, update roadmap and
   production metadata, render/check generated docs, and rerun
   `task ai:goal -- --track PT-EDITOR-UX`.

## Validation

Required validation for this implementation contract:

```text
task production:plan -- --milestone PM-EDITOR-UX-005 --roadmap WR-115
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task puml:validate
git diff --check
task ai:goal -- --track PT-EDITOR-UX
```

Promotion-contract action closeout on 2026-05-25:

- `task production:plan -- --milestone PM-EDITOR-UX-005 --roadmap WR-115`
  passed and reported `Next action: write_promotion_contract` with promotion
  preflight status `promotable`;
- `task docs:validate` passed;
- `task planning:validate` passed, including roadmap validate/check,
  production validate/check, and docs validation;
- `task puml:validate` passed;
- `git diff --check` passed;
- no product code changed and no roadmap or production state was promoted by
  this contract-writing action.

Promotion execution evidence on 2026-05-25:

- `task roadmap:promote -- --id WR-115 --state current_candidate --evidence
  "Accepted WR-115 graph canvas and node editor productization promotion
  contract at
  docs-site/src/content/docs/reports/implementation-plans/wr-115-graph-canvas-and-node-editor-productization/plan.md;
  completed native Story Lab, layered design-system, and standalone UI Designer
  workbench closeouts; PM-EDITOR-UX-005 ready-next production state; production
  plan preflight status promotable."` passed and promoted `WR-115` to
  `current_candidate`;
- `task roadmap:render` and `task production:render` refreshed generated
  roadmap and production views after promotion;
- `task production:plan -- --milestone PM-EDITOR-UX-005 --roadmap WR-115`
  passed after promotion and reported `Next action:
  write_implementation_contract`;
- `task planning:validate`, `task puml:validate`, and `git diff --check`
  passed after promotion.

Implementation-contract action closeout on 2026-05-25:

- `task production:plan -- --milestone PM-EDITOR-UX-005 --roadmap WR-115`
  passed and reported `WR-115` as `current_candidate` with `Next action:
  write_implementation_contract`;
- `task docs:validate` passed;
- `task planning:validate` passed, including roadmap validate/check,
  production validate/check, and docs validation;
- `task puml:validate` passed;
- `git diff --check` passed;
- no product code changed in this contract-only action.

Expected implementation validation after the next action selects code work:

```text
cargo test -p ui_graph_editor
cargo test -p editor_shell graph
cargo test -p editor_shell story_lab
cargo test -p runenwerk_editor graph
cargo test -p runenwerk_editor editor_ux
RUNENWERK_WRITE_PM_EDITOR_UX_005_EVIDENCE=1 cargo test -p runenwerk_editor pm_editor_ux_005 -- --nocapture
```

Use `./quiet_full_gate.sh` for broad closeout if implementation changes shared
graph contracts, editor shell graph composition, app evidence, Story Lab, or
validation infrastructure.

## Stop Conditions

Stop before implementation if:

- `task ai:goal -- --track PT-EDITOR-UX` no longer selects
  `PM-EDITOR-UX-005`;
- `WR-115` is not ready for the required roadmap action;
- implementation would make `domain/ui/ui_graph_editor` depend on editor, app,
  material, SDF, procgen, gameplay, particle, or animation code;
- a graph family would remain visible as a product-shaped placeholder;
- evidence would be text/action panel-only, descriptor-only, material-only, or
  missing graph interaction/state matrix coverage;
- native evidence cannot produce screenshot or typed platform-impossible
  reports for product graph scenarios.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/pm-editor-ux-005-graph-canvas-and-node-editor-productization/closeout.md
```

The closeout must state:

- changed files and owning modules;
- graph family readiness decisions;
- material graph interaction and evidence coverage;
- fallback/hide decisions for SDF/procgen/gameplay/particle/animation graph
  families;
- Story Lab stories and state matrices added;
- app-owned evidence artifacts generated;
- validation commands and results;
- known quality gaps that remain owned by `PM-EDITOR-UX-006` through
  `PM-EDITOR-UX-009`.

Expected completion quality is `runtime_proven` only if native Story Lab
manifest evidence proves the productized graph path and hide-or-certify policy.
Use `bounded_contract` if the implementation lands only a narrower graph
contract slice without native evidence.

## Perfectionist Closeout Audit

`PM-EDITOR-UX-005` must not claim `perfectionist_verified`. The final no-gap
audit remains `PM-EDITOR-UX-009`.

The closeout must keep visible gaps for:

- shell and product pattern polish;
- all registered visible surface migration;
- game UI readiness seam;
- final local-native no-gap certification.

Only `PM-EDITOR-UX-009` may remove those gaps after final native screenshots,
accessibility, interaction, visual/performance, roadmap, production, and full
validation evidence agree.
