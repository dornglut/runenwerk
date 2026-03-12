# Data-Driven Flow Fragments

## Purpose

Provide a narrow foundation for asset-authored render graph fragments that compile into `RenderFlowContribution`.

## APIs

- `parse_fragment_ron(...)`
- `RenderFlowFragmentSpec::validate()`
- `RenderFlowFragmentSpec::to_contribution()`
- `RenderFlowFragmentHotReloadState::apply_source(...)`

## Supported Fragment Scope

- texture/import resources (`sampled`, `storage`, `color`, `depth`, `history`, `imported`)
- pass declarations (`compute`, `fullscreen`, `graphics`, `copy`, `present`, `builtin_ui_composite`)
- namespaced IDs with contribution namespace ownership for pass IDs and non-import resources

## Variant Support

`RenderFlowVariant` supports:

- main view fragments
- editor viewport variants (`EditorViewport`)
- named variants (`Named`)

Variant-specific contribution IDs remain namespace-safe.

## Hot Reload State

`RenderFlowFragmentHotReloadState` tracks:

- source hash
- source revision
- last parse/compile error

and reports transitions:

- `Unchanged`
- `Updated { contribution, revision }`
- `Failed { error, revision }`

## Current Constraint

`RenderFlow`/`RenderFlowContribution` currently take `&'static str` IDs, so fragment compilation bridges dynamic asset strings through internal string interning. Owned-ID flow APIs remain a future extension.
