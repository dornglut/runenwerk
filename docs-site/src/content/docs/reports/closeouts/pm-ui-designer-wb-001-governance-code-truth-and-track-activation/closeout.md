---
title: PM-UI-DESIGNER-WB-001 Governance Code Truth And Track Activation Closeout
description: Completed bounded-contract closeout evidence for WR-120 UI Designer Workbench product-track governance, code-truth reconciliation, follow-on scope decomposition, and validation.
status: completed
owner: editor
layer: workspace / domain/ui / domain/editor / app
canonical: false
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/active/ui-designer-interface-lab-platform-design.md
  - ../../../design/active/editor-product-ux-lab-and-game-ui-ready-foundations-design.md
related_reports:
  - ../../implementation-plans/wr-120-ui-designer-workbench-product-track-governance/plan.md
  - ../pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md
  - ../../roadmap-intake/2026-05-25-create-a-new-pt-ui-designer-workbench-pr/proposal.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# PM-UI-DESIGNER-WB-001 Governance Code Truth And Track Activation Closeout

## Summary

`PM-UI-DESIGNER-WB-001` / `WR-120` completed the bounded governance slice for
`PT-UI-DESIGNER-WORKBENCH`. The slice installed the product track, kept the
accepted UI Designer Workbench Product Design as the product source of truth,
classified current code-truth and planning-truth drift, preserved `WR-114` /
`PM-EDITOR-UX-004` as historical bounded workbench-route evidence, and split
follow-on product work into separate candidate slices.

No product runtime code, app code, domain code, engine code, binaries,
fixtures, benchmarks, package persistence, catalog, canvas, inspector,
operation, scenario evidence, performance baseline, or game-runtime UI behavior
changed in this slice.

## Changed Artifacts

- Added product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Added design-first planning contract:
  `docs-site/src/content/docs/reports/implementation-plans/wr-120-ui-designer-workbench-product-track-governance/plan.md`.
- Added `PT-UI-DESIGNER-WORKBENCH` and milestone sequence in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Archived `WR-120` as completed bounded governance evidence.
- Updated generated roadmap and production planning views.

## Governance Decisions

- `PT-UI-DESIGNER-WORKBENCH` is a new productization track. It does not reopen
  `PT-EDITOR-UX` and does not rewrite the completed `WR-114` closeout.
- `WR-114` / `PM-EDITOR-UX-004` remains valid only as bounded workbench-route
  evidence, not as proof of the full V1 package, catalog, canvas, inspector,
  operation, scenario, performance, or game-runtime compatibility contracts.
- Generic UI definition, package, document, Canonical UI IR, recipe, target
  profile, token, binding, and evidence descriptor truth remains in `domain/ui`.
- Editor workbench product semantics, app-neutral view models, readiness
  states, routes, shell layout, and editor-host adapters remain in
  `domain/editor`.
- Concrete launch paths, app-owned session state, provider fixtures, runtime
  execution, screenshots, manifests, and platform-impossible evidence remain in
  `apps/runenwerk_editor`.
- Game-runtime UI remains downstream. This track may prove compatibility
  descriptors, but runtime HUD behavior belongs to `PT-GAME-RUNTIME-UI` or a
  later game UI track.

No ADR is required for this governance closeout because ownership and
dependency direction remain unchanged. A new ADR or accepted design update is
required before adding a game UI owner crate, changing dependency direction,
making projection/evidence artifacts authoritative, or moving generic UI truth
into app code.

## Code Truth And Planning Truth

The completed planning contract records the drift matrix that governs follow-on
work:

- `apps/runenwerk_editor/src/runtime/app.rs` enum
  `RunenwerkRuntimeWorkbench` has `FullEditor` and `MaterialLab`; there is no
  standalone UI Designer workbench variant.
- `apps/runenwerk_editor/src/bin/runenwerk_material_lab.rs` exists; there is no
  `runenwerk_ui_designer.rs` binary.
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` function
  `ui_designer_workbench_view_model` builds a UI Designer workbench view model,
  but that does not complete the new V1 product package, catalog, direct
  operation, persistence, source-versioned evidence, or performance contracts.
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs` module
  `editor_definition` and
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
  module `build_editor_lab_surface` contain bounded workbench view-model and
  composition contracts from earlier editor UX work.
- Completed `WR-114` evidence is code-backed and evidence-backed only for the
  earlier workbench route. It must not be used to skip the stricter
  `PT-UI-DESIGNER-WORKBENCH` milestones.

## Follow-On Rows

The planning contract reserves these follow-on candidates without applying them
as active roadmap rows:

- `WR-121`: V1 package, document, session, and evidence model.
- `WR-122`: standalone app shell and embedded host parity.
- `WR-123`: catalog, hierarchy, canvas, and inspector V1.
- `WR-124`: operation diff, apply, and rollback.
- `WR-125`: scenario evidence and performance baselines.
- `WR-126`: game-runtime compatibility seam.
- `WR-127`: runtime-proven closeout and handoff.

Each candidate still needs roadmap intake or promotion evidence, production
planning, accepted design gates, validation, and closeout before product code
can start. Candidate names are planning labels, not completed or active WR
rows.

## Validation Results

Validation run on 2026-05-26:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-001 --roadmap WR-120 reported design_first before closeout.
task ai:architecture-governance -- --task "PT-UI-DESIGNER-WORKBENCH product track governance" --scope "docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md; docs-site/src/content/docs/workspace/production-tracks.yaml; docs-site/src/content/docs/workspace/roadmap-items.yaml; docs-site/src/content/docs/workspace/roadmap-archive.yaml" printed the governance checklist and stop conditions.
task docs:validate passed.
task roadmap:render passed.
task roadmap:validate passed.
task roadmap:check passed.
task production:render passed.
task production:validate passed.
task production:check passed.
task planning:validate passed.
task puml:validate passed.
git diff --check passed.
```

`./quiet_full_gate.sh` is intentionally not part of this closeout because this
slice changed planning and docs only.

## Completion Quality

Completion quality is `bounded_contract`.

Known quality gaps remain by design:

- `PM-UI-DESIGNER-WB-002` still owns the V1 package, document, session, and
  evidence model.
- `PM-UI-DESIGNER-WB-003` still owns standalone app shell and embedded host
  parity.
- `PM-UI-DESIGNER-WB-004` still owns catalog, hierarchy, canvas, and inspector
  V1.
- `PM-UI-DESIGNER-WB-005` still owns operation diff, apply, and rollback.
- `PM-UI-DESIGNER-WB-006` still owns scenario evidence and performance
  baselines.
- `PM-UI-DESIGNER-WB-007` still owns game-runtime compatibility seam proof.
- `PM-UI-DESIGNER-WB-008` still owns runtime-proven closeout and handoff.

## Drift Check

The closeout matches the WR-120 contract:

- Generic UI truth remains in `domain/ui`.
- Editor workbench product semantics remain in `domain/editor`.
- App-hosted execution and evidence remain in `apps/runenwerk_editor`.
- The accepted product design is not treated as implementation evidence.
- `WR-114` remains historical bounded evidence.
- Product implementation remains deferred until a follow-on row has accepted
  roadmap evidence and a production implementation contract.

Next legal work is `PM-UI-DESIGNER-WB-002` design acceptance or design repair.
No `PM-UI-DESIGNER-WB-002` product code, standalone app shell, catalog,
canvas, inspector, operation pipeline, scenario evidence, performance baseline,
or game-runtime compatibility seam was implemented by this closeout.
