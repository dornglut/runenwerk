# Testing Strategy

## Purpose

Tests are executable architecture documentation. They protect long-term refactors and AI-assisted changes.

## Test Naming

Prefer behavior names:

```text
mounting_unknown_surface_definition_is_rejected
workspace_projection_preserves_selected_surface
scene_ratifier_reports_missing_required_transform
render_graph_rejects_unbound_storage_resource
command_rejection_emits_diagnostic
```

Avoid vague names like `works`, `test_1`, or `surface_test`.

## Test Tiers

- Unit tests
- Domain invariant tests
- Ratification tests
- Command behavior tests
- Projection golden tests
- Schema compatibility tests
- Smoke tests

## Projection Golden Tests

Projection-heavy flows should have golden tests, especially:

```text
WorkspaceState -> EditorShellProjection -> UiSurfaceMountPlan -> InteractionRoutes
```

Golden tests should catch route drift, structural-context loss, backend leakage, and stale projection epoch behavior.

## Canonical Commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Add project-specific commands as they stabilize:

```text
cargo check -p runenwerk_editor
cargo test -p runenwerk_editor --test startup_render_smoke -- --nocapture
```

Documentation validation:

```text
python3 tools/docs/validate_docs.py
```

## Minimum Rule

Every important invariant should have at least one test or ratification case.
