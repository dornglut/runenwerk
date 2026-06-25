# Testing Strategy

## Purpose

Tests are executable architecture documentation. They protect long-term refactors and assisted changes.

This file supports two operating modes:

- **connector/context mode**, where the agent cannot run commands and must report manual evidence honestly;
- **local checkout mode**, where commands can be run as optional validation helpers.

No workflow is invalid merely because command execution is unavailable. The final report must be precise about what was and was not validated.

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

## Connector/Context Manual Validation

When commands cannot be run, validate by inspection and report the limitation.

Manual validation should include the smallest relevant subset of:

- authority files read;
- owning crate, domain, and subsystem identified;
- dependency direction checked against `DEPENDENCY_RULES.md`;
- public API impact checked against `lib.rs`, README files, examples, and tests when available;
- docs impact checked against `docs-site/src/content/docs/workspace/documentation-structure.md`;
- changed files and exact functions/modules/sections listed;
- tests that should be run locally named explicitly;
- risks or unverified behavior stated without completion overclaiming.

Use this wording when command validation was unavailable:

```text
Command validation was not run because this work was performed through connector/context mode. Manual validation covered: <files and checks>. Recommended local validation: <commands or tests>.
```

## UI Story V2 Gate

UI Story V2 is a proof/orchestration contract. Its focused local validation gate is:

```text
cargo test -p ui_story
cargo test -p runenwerk_editor --bin runenwerk_ui_gallery
cargo test -p runenwerk_editor --bin runenwerk_ui_designer
cargo fmt --all --check
```

This gate covers Manifest V2, Registry V2, workflow graphs, app-owned evidence recording, workflow reports, mount decisions, CLI summaries, and the editor gallery integration.

`cargo test -p runenwerk_editor` is a broader editor package gate. Failures in editor-shell workspace facade methods such as `workspace_state`, `replace_workspace_state`, or `apply_workspace_mutation` are not UI Story proof failures unless the failing code directly touches the UI Story V2 path.

## Optional Local Commands

Use local commands only when a checkout is available.

Broad local gates:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Focused/project-specific helpers, when relevant and available:

```text
./quiet_editor_gate.sh
./quiet_full_gate.sh
cargo check -p runenwerk_editor
cargo test -p runenwerk_editor --test startup_render_smoke -- --nocapture
```

Documentation validation helper, when available:

```text
python3 tools/docs/validate_docs.py
```

These helpers do not define workflow authority. They are evidence-producing tools. If they cannot be run, state that clearly and use the manual validation checklist.

## Minimum Rule

Every important invariant should have at least one test or ratification case. Every final report should distinguish implemented behavior, inspected evidence, local command validation, and unverified risk.

<!-- BEGIN RUNENWERK:UI_COMPONENT_PLATFORM:root-note -->
## UI Component Platform note

The current UI Component Platform activation is `PT-UI-COMPONENT-PLATFORM`, starting after `PM-UI-STORY-004`. It defines reusable, story-proven `ControlPackage` and surface kernels before product-specific Gallery, Workbench, Designer, game HUD, or world-space UI behavior. See `docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md` and the `ui-component-platform-*-design.md` active design docs.
<!-- END RUNENWERK:UI_COMPONENT_PLATFORM:root-note -->
