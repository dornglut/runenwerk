---
title: PM-UI-DESIGNER-WB-003 Standalone App Shell And Embedded Host Parity Closeout
description: Runtime-proven closeout for WR-122 standalone UI Designer launch wiring and embedded Editor Design host parity.
status: completed
owner: editor
layer: app / domain/editor
canonical: false
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
related_reports:
  - ../../implementation-plans/wr-122-standalone-app-shell-and-embedded-host-parity/plan.md
  - ../pm-ui-designer-wb-001-governance-code-truth-and-track-activation/closeout.md
  - ../pm-ui-designer-wb-002-v1-package-document-session-and-evidence-model/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-archive.yaml
---

# PM-UI-DESIGNER-WB-003 Standalone App Shell And Embedded Host Parity Closeout

## Summary

`PM-UI-DESIGNER-WB-003` / `WR-122` completed the bounded host-parity slice for
the UI Designer Workbench. The implementation adds a standalone UI Designer
runtime selection path and proves that the embedded `Editor Design` workspace
and standalone host resolve the same app-neutral UI Designer canvas contract.

This slice does not implement the V1 catalog, hierarchy authoring, canvas
editing, inspector editing, operation diff/apply/rollback, scenario evidence,
performance baselines, game-runtime HUD behavior, or final handoff docs. Those
remain owned by `PM-UI-DESIGNER-WB-004` through `PM-UI-DESIGNER-WB-008`.

## Implementation Evidence

Code changes are limited to the WR-122 host-parity write scope:

- `apps/runenwerk_editor/src/runtime/app.rs` module `app`: added
  `RunenwerkRuntimeWorkbench::UiDesigner`,
  `build_ui_designer_workbench_headless_app`, `run_ui_designer_workbench`, and
  focused headless runtime-selection coverage.
- `apps/runenwerk_editor/src/runtime/mod.rs` module `runtime`: re-exported the
  UI Designer headless builder and runner for the standalone binary.
- `apps/runenwerk_editor/src/bin/runenwerk_ui_designer.rs` module
  `runenwerk_ui_designer`: added the standalone UI Designer entry point.
- `apps/runenwerk_editor/src/editor_app/state.rs` module `state`: added
  `RunenwerkEditorApp::new_ui_designer_workbench` and
  `RunenwerkEditorApp::try_new_ui_designer_workbench`.
- `apps/runenwerk_editor/src/runtime/resources.rs` module `resources`: added
  `EditorHostResource::ui_designer_workbench` and shell bootstrap coverage for
  the `Editor Design` workspace profile.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` module
  `workbench_host`: added the `UiDesigner` composition, UI Designer workbench
  profile, editor-design tool-suite selection, and embedded/standalone canvas
  parity tests.
- `apps/runenwerk_editor/src/shell/providers/mod.rs` module `providers`: added
  `EditorSurfaceProviderRegistry::runenwerk_ui_designer_workbench`.

The implementation keeps generic UI package, document, operation, token,
recipe, and evidence truth outside `apps/runenwerk_editor`. App code owns only
runtime selection, executable shell state, and host composition wiring.

## Gate Classification

`task production:plan -- --milestone PM-UI-DESIGNER-WB-003 --roadmap WR-122`
passed with `PM-UI-DESIGNER-WB-003` in `ready_next`, `WR-122` in
`current_candidate`, and dependencies `WR-004:support_only`,
`WR-046:support_only`, `WR-114:completed`, and `WR-120:completed`.

`task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH` selected
`PM-UI-DESIGNER-WB-003` with next legal action
`execute_next_wr_implementation_contract` before closeout metadata was updated.

## Validation Results

Focused validation run on 2026-05-26:

```text
cargo fmt --package runenwerk_editor passed.
cargo test -p runenwerk_editor runtime passed.
cargo test -p runenwerk_editor workbench_host passed.
cargo test -p runenwerk_editor ui_designer passed.
cargo test -p editor_shell ui_designer passed.
task production:plan -- --milestone PM-UI-DESIGNER-WB-003 --roadmap WR-122 passed.
task ai:goal -- --track PT-UI-DESIGNER-WORKBENCH selected PM-UI-DESIGNER-WB-003 execute_next_wr_implementation_contract.
```

Closeout metadata validation run on 2026-05-26:

```text
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
slice proves host selection and embedded parity with focused runtime and
domain/editor tests. Full product readiness remains owned by later milestones.

## Completion Quality

Completion quality is `runtime_proven`.

The runtime proof is headless rather than native-window screenshot evidence:
`build_ui_designer_workbench_headless_app` installs the real UI Designer
workbench host resource, active/open shell profile, and workbench composition
used by the standalone runner. Embedded and standalone canvas requests both
resolve through the same `runenwerk.editor_design` provider-family contract.

Known quality gaps remain by design:

- `PM-UI-DESIGNER-WB-004` still owns catalog, hierarchy, canvas, inspector,
  diagnostics, and normal authoring surfaces.
- `PM-UI-DESIGNER-WB-005` still owns operation diff, apply, rollback,
  undo/redo, and deterministic textual patch flow.
- `PM-UI-DESIGNER-WB-006` still owns scenario evidence and performance
  baselines.
- `PM-UI-DESIGNER-WB-007` still owns game-runtime compatibility seam proof.
- `PM-UI-DESIGNER-WB-008` still owns final runtime-proven track closeout,
  usage docs, examples, and handoff notes.

## Drift Check

The closeout satisfies the PM-003 host-parity acceptance criteria and does not
claim later product behavior. Full editor and Material Lab behavior remain
covered by the existing workbench host tests, runtime tests, and unchanged
composition paths.
