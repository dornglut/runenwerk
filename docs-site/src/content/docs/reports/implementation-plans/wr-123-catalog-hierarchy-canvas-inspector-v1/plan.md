---
title: WR-123 Catalog Hierarchy Canvas Inspector V1 Contract
description: Ready-next implementation contract for PM-UI-DESIGNER-WB-004 product catalog, hierarchy, canvas, inspector, diagnostics, and diff/review surfaces.
status: active
owner: editor
layer: domain/editor / app
canonical: false
last_reviewed: 2026-05-26
related:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../reports/closeouts/pm-ui-designer-wb-003-standalone-app-shell-and-embedded-host-parity/closeout.md
---

# WR-123 Catalog Hierarchy Canvas Inspector V1 Contract

## Goal

Implement `PM-UI-DESIGNER-WB-004` by replacing normal UI Designer debug-facing
authoring surfaces with product-grade V1 surfaces for the shared component
catalog, hierarchy, canvas, inspector, diagnostics, and diff/review workflow.

This contract covers product surface projection and selection parity only:

- catalog rows expose recipe provenance, target compatibility, slot/token
  requirements, accessibility requirements, and typed disabled reasons;
- hierarchy, canvas, and inspector selection stay synchronized by source
  version and authored id;
- diagnostics and diff/review surfaces are visible product surfaces, not hidden
  debug text panels;
- app providers execute the retained surface shell over editor-owned
  app-neutral view models;
- generic UI package, document, Canonical UI IR, token, recipe, and evidence
  truth remain outside `apps/runenwerk_editor`.

It must not implement operation diff/apply/rollback, undo/redo, deterministic
textual patch generation, source-versioned scenario evidence capture,
performance baselines, or game-runtime HUD behavior. Those remain owned by
`PM-UI-DESIGNER-WB-005` through `PM-UI-DESIGNER-WB-008`.

## Source Of Truth

- Production milestone: `PM-UI-DESIGNER-WB-004` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-123` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Accepted recipe/catalog design:
  `docs-site/src/content/docs/design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md`.
- Accepted visual layout and composition design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`.
- Accepted Canonical UI IR and composition design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`.
- Completed host-parity closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-003-standalone-app-shell-and-embedded-host-parity/closeout.md`.
- Existing editor-shell view-model boundary:
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`.
- Existing retained UI Designer composition:
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`.
- Existing app provider boundary:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`.
- Existing app session boundary:
  `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring`.

## Readiness

`PM-UI-DESIGNER-WB-004` starts from `designing` after `PM-UI-DESIGNER-WB-003`
completed. The accepted product design and supporting accepted UI Designer
designs cover the V1 catalog, hierarchy, canvas, inspector, diagnostics, and
diff/review surface shape, but no linked WR row or implementation contract
existed.

Architecture governance kickoff was run for this scope on 2026-05-26. The
bounded owner split remains:

- `domain/ui` owns generic recipe, widget, component, UI document, Canonical UI
  IR, token, target-profile, diagnostics, and evidence descriptor truth.
- `domain/editor` owns editor-facing UI Designer view models, source-versioned
  selection projection, catalog row projection, hierarchy/canvas/inspector
  product surface models, and composition contracts.
- `apps/runenwerk_editor` owns concrete provider execution, shell session
  filters/selection, command bridging, fixtures, and future native/runtime
  evidence.

No ADR is required while the implementation preserves these ownership and
dependency boundaries. Require an ADR or accepted design update before moving
generic recipe, UI document, Canonical UI IR, token, or evidence truth into app
code, or before adding a new owner crate.

After this planning action, expected next workflow is:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-004 --roadmap WR-123
```

The command must report the next promotion or implementation action before any
product code changes start.

## Promotion Readiness

After the ready-next intake row and this contract were applied,
`task production:plan -- --milestone PM-UI-DESIGNER-WB-004 --roadmap WR-123`
reported:

- production milestone state: `ready_next`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-004:support_only`, `WR-046:support_only`,
  `WR-108:completed`, `WR-120:completed`, and `WR-122:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-123 --state current_candidate --evidence "<accepted evidence>"
```

Accepted promotion evidence:

- accepted UI Designer product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`;
- accepted recipe/catalog design:
  `docs-site/src/content/docs/design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md`;
- accepted visual layout and composition design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`;
- accepted Canonical UI IR design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`;
- completed PM003 host-parity closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-003-standalone-app-shell-and-embedded-host-parity/closeout.md`;
- this active WR-123 product-surface contract.

Promotion may proceed only while this evidence remains true and the production
goal still selects `PM-UI-DESIGNER-WB-004`.

## Implementation Scope

Allowed future source scopes:

- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition`: add or extract app-neutral V1 view models for catalog
  rows, hierarchy nodes, canvas projection, inspector fields, diagnostics, and
  diff/review readiness.
- `domain/editor/editor_shell/src/lib.rs` module `lib`: re-export product
  surface view models that belong on the normal public editor-shell import path.
- `domain/editor/editor_shell/src/surfaces/` module subtree: add subdomain
  modules if the V1 surface contracts outgrow the current
  `editor_definition` module.
- `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface`: render the V1 product surfaces from
  editor-shell view models without app-owned UI truth.
- `domain/editor/editor_shell/src/composition/` module subtree: add focused
  composition modules if required to keep catalog, hierarchy, canvas,
  inspector, diagnostics, and review builders separated by responsibility.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` module
  `self_authoring`: feed V1 product view models into the existing UI Designer
  provider and retire normal debug/action-list dependence for PM004 surfaces.
- `apps/runenwerk_editor/src/shell/providers/` module subtree: add provider
  tests or provider helpers needed to prove product-surface readiness and
  source-versioned selection parity.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` module `self_authoring`:
  keep concrete selection/filter/session state reconstructable and app-owned;
  do not make it package or document source truth.
- `apps/runenwerk_editor/src/shell/tests.rs` module `shell::tests`: add app
  integration tests proving normal UI Designer workflows use product surfaces.

Explicit non-goals:

- no operation diff/apply/rollback implementation;
- no undo/redo or deterministic textual patch generation;
- no package persistence/reload implementation beyond existing bounded support;
- no source-versioned scenario evidence capture;
- no performance baseline or resize instrumentation;
- no game-runtime HUD behavior or game UI runtime projection implementation;
- no final usage docs, examples, or track handoff closeout.

## Acceptance Criteria

- Catalog entries expose target compatibility, disabled reasons, slots, tokens,
  states, accessibility requirements, and package/source provenance.
- Hierarchy, canvas, and inspector selection stay synchronized by source
  version and authored id.
- Normal V1 authoring workflows do not expose generic text/action
  self-authoring panels as their primary product surface.
- Unsupported or unavailable catalog/canvas/inspector operations fail closed
  with typed diagnostics and visible product readiness classification.
- Existing standalone UI Designer and embedded `Editor Design` host parity from
  `WR-122` remains intact.
- Generic UI truth remains outside `apps/runenwerk_editor`.

## Validation

Future implementation validation must include at minimum:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-004 --roadmap WR-123
cargo fmt --package editor_shell --package runenwerk_editor
cargo test -p editor_shell ui_designer
cargo test -p editor_shell editor_lab
cargo test -p runenwerk_editor ui_designer
cargo test -p runenwerk_editor workbench_host
cargo test -p runenwerk_editor direct_manipulation
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
```

Run `./quiet_full_gate.sh` only if the implementation expands beyond the
focused product-surface slice or before a later final runtime-proven closeout
requires full validation.

## Stop Conditions

Stop before product code changes if:

- `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` no longer selects
  `PM-UI-DESIGNER-WB-004` or linked `WR-123` work;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-004 --roadmap WR-123`
  reports a design, metadata, dependency, promotion, or current-candidate
  blocker;
- implementation would put generic recipe, package, UI document, Canonical UI
  IR, token, or evidence truth into `apps/runenwerk_editor`;
- hierarchy/canvas/inspector edits require direct runtime-widget mutation
  instead of typed operation contracts;
- a required behavior belongs to `PM-UI-DESIGNER-WB-005` or later;
- validation or generated roadmap/production checks fail.

## Closeout Requirements

The later implementation closeout must include:

- this contract path;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-004 --roadmap WR-123`
  output classification used before coding;
- focused test evidence for catalog compatibility/disabled reasons,
  source-versioned hierarchy/canvas/inspector selection parity, typed
  diagnostics, and normal product-surface workflow;
- confirmation that `WR-122` standalone and embedded host parity did not
  regress;
- roadmap evidence update for `WR-123`;
- production milestone evidence update only when PM004 acceptance criteria are
  actually met;
- rerun `task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH`.

Expected completion quality for `WR-123` is `runtime_proven` only if focused
runtime or headless product-surface evidence proves the actual provider and
composition paths. Otherwise close as `bounded_contract` with explicit known
gaps.

## Perfectionist Closeout Audit

No perfectionist audit is intended for `WR-123`. The row must keep later gaps
visible:

- operation diff/apply/rollback remains in `PM-UI-DESIGNER-WB-005`;
- scenario evidence and performance baselines remain in
  `PM-UI-DESIGNER-WB-006`;
- game-runtime compatibility seam proof remains in `PM-UI-DESIGNER-WB-007`;
- runtime-proven final closeout, usage docs, examples, and handoff remain in
  `PM-UI-DESIGNER-WB-008`.
