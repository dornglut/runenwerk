---
title: WR-134 Workbench Composition Manifest Compiler And Authored Custom Workbench Contract
description: Closeout evidence for the app-neutral workbench composition compiler, authored workbench composition documents, and atomic custom workbench activation path.
status: completed
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-29
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# WR-134 Workbench Composition Manifest Compiler And Authored Custom Workbench Contract

## Summary

`WR-134` completes the long-term workbench composition compiler batch at
`bounded_contract` quality. The implementation moves workbench declaration
authority into app-neutral editor-shell manifests, makes `ProfileRef` durable
profile identity, adds authored workbench composition documents, and activates
custom composition/profile/layout packages atomically.

The batch does not introduce a fluent builder, ECS rewrite, dynamic external
plugin system, or external provider loading. UI Designer visual layout editing
continues to use the existing layout controls; this batch adds the durable
composition/profile/layout document contract and activation path those controls
compile through.

## Implementation Summary

- `domain/editor/editor_definition/src/workbench.rs` adds
  `EditorWorkbenchCompositionDefinition` and
  `EditorWorkbenchHostPolicyDefinition`.
- `domain/editor/editor_definition/src/document.rs` and
  `domain/editor/editor_definition/src/validate.rs` add the
  `WorkbenchComposition` document kind/content and validate durable lowercase
  dotted composition ids, suite ids, profile refs, default profile refs, and
  host policy capability keys.
- `domain/editor/editor_shell/src/workbench/` owns the app-neutral manifest,
  compiler, compiled-parts, and diagnostic contracts for tool suites, workspace
  profiles, workbench compositions, and authored document conversion.
- `domain/editor/editor_shell/src/workspace/profile.rs` makes `ProfileRef`
  durable profile identity while keeping `WorkspaceProfileId` as a compatibility
  handle.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` builds built-in and
  authored workbenches through one manifest-first path and keeps concrete
  provider support validation app-owned.
- `apps/runenwerk_editor/src/runtime/resources.rs` activates authored
  workbench composition packages transactionally: manifest conversion, host
  build, provider support validation, and replacement workspace formation must
  all succeed before host/shell state is swapped.
- `apps/runenwerk_editor/src/shell/self_authoring/mod.rs`,
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`, and
  `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` add visible
  create, edit, review, apply, activate, and rollback workflows for custom
  workbench packages.
- `ToolSuiteProfileDefinition` and the old suite-owned profile conversion path
  were removed from the public editor-shell architecture.

## Validation

Focused validation run on 2026-05-29:

```text
task ai:architecture-governance -- --task "WR-134 completion: authored workbench composition documents, atomic package activation, and ProfileRef-authoritative custom workbench UX" --scope "domain/editor/editor_definition/src, domain/editor/editor_shell/src/workbench, domain/editor/editor_shell/src/workspace, domain/editor/editor_shell/src/tool_suite, apps/runenwerk_editor/src/shell, apps/runenwerk_editor/src/editor_app/state.rs, apps/runenwerk_editor/src/runtime/resources.rs, docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md"
cargo test -p editor_definition workbench --quiet
cargo test -p editor_shell tool_suite --quiet
cargo test -p editor_shell workspace_profile --quiet
cargo test -p editor_shell workbench --quiet
cargo test -p runenwerk_editor workbench_host --quiet
cargo test -p runenwerk_editor ui_designer --quiet
cargo test -p runenwerk_editor material_lab --quiet
cargo test -p runenwerk_editor workbench_composition --quiet
cargo test -p runenwerk_editor toolbar_custom_workbench_package_activates_atomically --quiet
cargo test -p runenwerk_editor invalid_custom_workbench_activation_preserves_previous_host_and_shell_state --quiet
cargo test -p runenwerk_editor editor_lab_workbench_composition_fields_edit_suite_and_profile_lists --quiet
```

Results:

- Architecture governance produced the WR-134 checklist and did not require a
  split roadmap row.
- `editor_definition` workbench validation tests passed.
- `editor_shell` tool-suite, workspace-profile, and workbench compiler slices
  passed.
- `runenwerk_editor` workbench-host, UI Designer, Material Lab, custom package
  activation, failed activation preservation, and composition field-edit tests
  passed.

Final metadata and full-gate validation:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
./quiet_full_gate.sh
```

Results:

- Docs, roadmap, production, and planning validation passed on 2026-05-29.
- `./quiet_full_gate.sh` passed on 2026-05-29 with fmt, clippy, and full test
  phases green.

## Completion Quality

Completion quality is `bounded_contract`.

The proof covers the WR-134 contract boundary:

- authored workbench composition/profile/layout documents compile through the
  same path as built-in manifests;
- `ProfileRef` and composition ids are durable identity;
- unknown suite ids, duplicate refs, missing package documents, unknown layout
  surfaces, default profile drift, and provider support failures reject;
- custom package activation preserves previous host and shell state on failure;
- apply review, applied snapshots, last-applied snapshots, and rollback are
  package-aware for workbench compositions;
- built-in Full Editor, Material Lab, UI Designer, and host tests still compile
  through the manifest-first path.

Known quality gaps:

- This is not a `perfectionist_verified` claim.
- External dynamic plugin loading remains explicitly out of scope.
- The UI Designer visual workbench-authoring product can still be expanded with
  richer picker widgets for suite/profile selection; the durable text-field
  document contract and compiler path are complete for this batch.
- `WorkspaceProfileId` compatibility call sites remain until a later cleanup can
  remove profile-id-first APIs.

## Closeout Decision

`WR-134` is closed as completed at `bounded_contract` after roadmap,
production, planning, docs, and full-gate validation passed. Future work should
build on the authored composition document contract instead of reintroducing
suite-owned profile builders or legacy `ToolSurfaceKind` authority.
