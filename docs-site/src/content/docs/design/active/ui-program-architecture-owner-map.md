---
title: UI Program Architecture Owner Map
status: active
owner: ui
---

# UI Program Architecture Owner Map

## Status And Authority

This document is the PM-UI-PROGRAM-ARCH-002 owner-map contract for
`PT-UI-PROGRAM-ARCHITECTURE`. It records the long-term UI owner boundaries that
later implementation milestones must satisfy. It does not authorize product
implementation, crate creation, MaterialProgram work, or `foundation/meta`
extraction by itself.

## Final Owner Map

The final architecture target is domain-owned and stays under `domain/ui/`:

- `ui_definition`: authored UI source, normalization, validation, source
  locations, templates, and retained compatibility inputs.
- `ui_schema`: UI-owned schema/value model, schema IDs, schema versions, event
  payload schemas, control property schemas, and binding value schemas.
- `ui_program`: durable `UiProgram`, graph families, stable IDs, source maps,
  diagnostics attachment, route/event contracts, and versioning.
- `ui_controls`: `ControlPackage` registry and package-owned control contracts,
  schemas, kernels, accessibility, fixtures, diagnostics, and migration hooks.
- `ui_compiler`: `UiProgram` to runtime artifact lowering, package resolution,
  capability checks, cache keys, and artifact building.
- `ui_artifacts`: `UiRuntimeArtifactManifest` and `UiRuntimeArtifactTables`.
- `ui_evaluator`: deterministic runtime evaluation of artifacts into `UiOutput`.
- `ui_state`: transient, preview, committed, focus, hover, drag, animation,
  host-fed, and package-owned state contracts.
- `ui_binding`: read/write bindings, snapshots, dirty propagation, collection
  diffs, host data contracts, diagnostics, and capability checks.
- `ui_hosts`: pure editor, game, world-space, and headless host contracts.
- `ui_render_data`: renderer-facing UI output only, including `UiFrame` and
  draw-neutral visual packets.
- `ui_theme`: theme tokens, style values, variants, and style slots.
- `ui_text`: font/style intent, shaping requests, layout metrics, glyph identity
  keys, atlas preparation keys, and fallback policy.
- `ui_input`: normalized pointer, keyboard, gamepad, and tablet input facts.
- `ui_accessibility`: roles, labels, descriptions, focus order, navigation, and
  semantic hints.
- `ui_geometry`: rects, transforms, constraints, hit regions, and layout units.
- `ui_surface`: surface identity, mounting, projection, visibility, lifetime,
  and presentation contracts.
- `ui_testing`: headless fixtures, proof harnesses, structural assertions,
  diagnostics, source maps, and readiness evidence.

## Current-To-Final Reconciliation

Current retained UI remains valid during migration:

```text
ui_definition -> ui_tree / ui_widgets / ui_runtime -> ui_render_data
```

The architecture implementation path is:

```text
ui_definition
-> ui_program
-> ui_compiler
-> ui_artifacts
-> ui_evaluator
-> ui_render_data
-> host
```

`ui_layout` remains the current retained layout owner until `ui_program`,
`ui_compiler`, and `ui_evaluator` prove the layout graph and artifact path.
`ui_math` remains compatible until geometry primitives move into `ui_geometry`
through an exact WR. `ui_tree`, `ui_widgets`, and `ui_runtime` remain retained
compatibility infrastructure until PM-UI-PROGRAM-ARCH-007 proves bridge or
adapter behavior.

## Crate Creation Gate

Later milestones may create only the exact UI owner crates listed in their
locked WR and `plan.contract.yaml` scopes. Crate creation is not a blanket
permission, and each new crate requires:

- exact `new:` crate `Cargo.toml` and source-file outputs;
- root workspace `Cargo.toml` edits in scope;
- executable cargo validations for the new package;
- rollback rules for the exact crate boundary.

No placeholder folders are authorized.

## Downstream Contract

PM-003 through PM-008 may implement architecture contracts only under their
active WR, accepted `plan.contract.yaml`, execution lock, validation commands,
and closeout evidence. PM-009 may satisfy architecture truth only after concrete
code, tests, fixtures, diagnostics, source maps, migration evidence, retained UI
compatibility, and visual/render boundary evidence exist.

MaterialProgram implementation and shared `foundation/meta` extraction remain
blocked until their own accepted gates authorize them.

<!-- BEGIN RUNENWERK:UI_COMPONENT_PLATFORM:uiprogram-alignment -->
## Relationship to UI Component Platform

`PT-UI-COMPONENT-PLATFORM` aligns with the UiProgram owner map. `ui_controls`, `ui_program`, `ui_artifacts`, `ui_evaluator`, `ui_binding`, `ui_state`, `ui_hosts`, `ui_accessibility`, `ui_geometry`, `ui_text`, `ui_theme`, `ui_input`, `ui_layout`, `ui_render_data`, and `ui_story` remain bounded owners. Component Platform docs define reusable package contracts and proof requirements; UiProgram proof slices remain bounded and must not become product-specific bypasses.
<!-- END RUNENWERK:UI_COMPONENT_PLATFORM:uiprogram-alignment -->
