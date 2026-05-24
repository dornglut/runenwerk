---
title: PM-UI-LAB-006 Preview Lab And Runtime Evidence Closeout
description: Runtime-proven closeout evidence for WR-087 Editor Lab preview scenarios, retained visual artifacts, diagnostics snapshots, accessibility checks, performance evidence, unsupported-check diagnostics, and degraded-provider proof.
status: completed
owner: editor
layer: app/runtime-evidence
canonical: true
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/accepted/ui-lab-preview-lab-runtime-evidence-design.md
  - ../../../design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md
  - ../../../design/active/ui-lab-productization-design.md
related_reports:
  - ../../implementation-plans/wr-087-ui-lab-preview-lab-runtime-evidence-matrix/plan.md
  - ./artifacts/runtime-proof.txt
  - ./artifacts/evidence-manifest.ron
  - ./artifacts/provider-snapshot.ron
  - ./artifacts/diagnostics-snapshot.ron
  - ./artifacts/accessibility-snapshot.ron
  - ./artifacts/performance-snapshot.ron
  - ./artifacts/unsupported-checks.ron
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# PM-UI-LAB-006 Preview Lab And Runtime Evidence Closeout

## Scope

`WR-087` completes the bounded `PM-UI-LAB-006` implementation slice: Editor Lab
now has an app-owned Preview Lab evidence harness with a typed scenario catalog,
runtime evidence manifest, retained visual artifacts, diagnostics snapshots,
accessibility snapshots, performance snapshots, unsupported-check diagnostics,
and degraded-provider proof.

This slice does not implement PM-007 public API ergonomics, usage docs,
examples, final PT-UI-LAB closeout, game-runtime UI projection execution,
native-window screenshot infrastructure, GPU visual diff infrastructure, or
no-gap perfectionist certification.

## Implementation Summary

- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs` module
  `editor_lab_evidence` defines app-owned `EditorLabPreviewScenario`,
  `EditorLabEvidenceRun`, `EditorLabEvidenceArtifact`,
  `EditorLabAccessibilitySnapshot`, `EditorLabPerformanceSnapshot`,
  `EditorLabEvidenceManifest`, unsupported-check diagnostics, the required
  Preview Lab scenario catalog, and manifest validation.
- `apps/runenwerk_editor/src/shell/tests.rs` test
  `pm_ui_lab_006_runtime_evidence_reports_preview_lab` executes the real
  app-hosted Editor Lab shell/provider/project/apply/rollback path and writes
  PM006 artifacts when `RUNENWERK_WRITE_PM_UI_LAB_006_EVIDENCE=1`.
- `apps/runenwerk_editor/src/shell/mod.rs` module `shell` exports the
  app-owned evidence contracts for normal editor app usage.

The implementation keeps execution, artifact writing, accessibility snapshots,
performance measurements, provider state, and unsupported-check diagnostics in
`apps/runenwerk_editor`. `domain/editor/editor_shell` still owns retained
surface composition, and `domain/ui/ui_definition` remains behavior-free.

## Runtime Evidence

Runtime proof artifacts:

- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/runtime-proof.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/evidence-manifest.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/provider-snapshot.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/success-retained-surface-debug.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/warning-retained-surface-debug.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/error-diagnostics.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/project-package.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/reload-retained-surface-debug.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/activation-reports.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/apply-retained-surface-debug.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/rollback-reports.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/rollback-retained-surface-debug.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/degraded-provider-retained-surface-debug.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/accessibility-snapshot.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/accessibility-retained-surface-debug.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/performance-snapshot.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/performance-retained-surface-debug.txt`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/diagnostics-snapshot.ron`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/unsupported-checks.ron`

The artifacts are generated by:

```text
$env:RUNENWERK_WRITE_PM_UI_LAB_006_EVIDENCE='1'; cargo test -p runenwerk_editor pm_ui_lab_006_runtime_evidence_reports_preview_lab -- --nocapture
```

The runtime proof opens the Editor Design profile through the app shell and
proves:

- success: Editor Lab surfaces mount and produce retained visual/provider
  artifacts;
- warning: preview-console warning state is visible in the Command Diff surface;
- error: invalid project package input is preserved with typed diagnostics;
- reload: saved project package reloads without live activation;
- apply: accepted apply produces review, activation queueing, runtime activation
  report, and retained command-surface evidence;
- rollback: snapshot-backed rollback records typed rollback reports and retained
  command-surface evidence;
- degraded-provider: non-previewable selection renders a typed degraded Editor
  Lab canvas with recovery controls;
- accessibility: retained controls expose labels/routes and disabled reasons;
- performance: supported scenario setup, retained-surface formation timing,
  artifact count, and artifact byte evidence are recorded;
- unsupported checks: native screenshot capture, native focus traversal, pixel
  contrast sampling, native screenshot timing, and GPU visual diff timing are
  typed unsupported diagnostics, not hidden passes.

## Acceptance Evidence

- The scenario catalog and manifest validation reject descriptor-only evidence.
- Every required PM006 state family has a manifest run, result status, artifact
  links, diagnostics or unsupported-check records where needed, and retained
  visual or equivalent artifacts.
- Accessibility and performance reports are real generated artifacts.
- Native screenshot/GPU visual-diff gaps are explicit unsupported-check records
  with retained visual equivalents.
- Degraded-provider behavior is app-hosted and uses the real Editor Lab canvas
  provider path.
- Reload, apply, activation, and rollback scenarios reuse PM005 project IO and
  activation paths instead of duplicate test-only state.
- `ui_definition` remains behavior-free and did not gain app scenario,
  screenshot, accessibility, performance, artifact-writing, or provider
  execution behavior.

## Validation

Focused validation completed during implementation and artifact generation:

```text
cargo fmt
cargo test -p runenwerk_editor pm_ui_lab_006
$env:RUNENWERK_WRITE_PM_UI_LAB_006_EVIDENCE='1'; cargo test -p runenwerk_editor pm_ui_lab_006_runtime_evidence_reports_preview_lab -- --nocapture
```

The artifact-writing test required escalated execution after sandboxed Cargo
target-lock access was denied. It completed successfully after escalation.

Final metadata validation must be run after writing this closeout, moving
`WR-087` to completed archive state, and marking `PM-UI-LAB-006` completed:

```text
cargo test -p runenwerk_editor editor_lab
task docs:validate
task puml:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
git diff --check
task ai:goal -- --track PT-UI-LAB --scope non-deferred
```

## Completion Quality

Completion quality is `runtime_proven`.

Known quality gaps:

- PM-006 proves the Editor Lab V1 evidence matrix through retained visual or
  equivalent artifacts in the current headless runtime test environment.
- Native window screenshots, GPU visual diffing, native focus traversal, pixel
  contrast sampling, native screenshot timing, and GPU visual-diff timing remain
  explicit unsupported-check diagnostics.
- PM-007 still owns public API ergonomics review, usage docs, examples, and the
  final runtime-proven PT-UI-LAB closeout.
- Game-runtime UI projection execution remains out of Editor Lab V1 scope.
- `perfectionist_verified` is intentionally not claimed. The no-gap audit
  remains later `PT-UI-LAB-PERFECTION` or equivalent scope.

## Drift Check

The phase completion drift check found the implementation aligned with the
accepted PM-006 Preview Lab runtime evidence design and WR-087 contract:

- App-owned Preview Lab evidence execution and artifact writing remain under
  `apps/runenwerk_editor`.
- Editor shell retained composition remains a projection contract and does not
  own scenario execution or artifacts.
- `domain/editor/editor_definition` remains runtime-neutral.
- Generic `domain/ui/ui_definition` still owns behavior-free UI definitions and
  diagnostics only.
- PM-007 remains incomplete and must not be inferred from PM-006 runtime
  evidence.
