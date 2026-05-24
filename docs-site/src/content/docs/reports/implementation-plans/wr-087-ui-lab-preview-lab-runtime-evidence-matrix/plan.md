---
title: WR-087 UI Lab Preview Lab Runtime Evidence Matrix Contract
description: Promotion and implementation-readiness contract for PM-UI-LAB-006 app-owned Editor Lab preview scenarios, visual or equivalent runtime evidence, diagnostics snapshots, accessibility checks, performance evidence, and degraded-provider proof.
status: active
owner: editor
layer: app/runtime-evidence
canonical: true
last_reviewed: 2026-05-24
related:
  - ../../../design/accepted/ui-lab-preview-lab-runtime-evidence-design.md
  - ../../../design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md
  - ../../../design/active/ui-lab-productization-design.md
  - ../../../reports/closeouts/pm-ui-lab-005-persistence-project-io-diff-apply-and-rollback/closeout.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-087 UI Lab Preview Lab Runtime Evidence Matrix Contract

## Goal

Implement `PM-UI-LAB-006` by turning the existing narrow Editor Lab runtime
proofs into a governed Preview Lab evidence matrix. The slice must prove real
app-hosted Editor Lab behavior for success, warning, error, reload, apply,
rollback, degraded-provider, accessibility, performance, and unsupported-check
states.

This contract covers promotion readiness and the future bounded implementation
slice only. It must not implement product code until WR-087 is promoted by the
roadmap workflow and `task ai:goal -- --track PT-UI-LAB --scope non-deferred`
selects a legal implementation action.

## Source Of Truth

- Production milestone: `PM-UI-LAB-006` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-087` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted PM006 design:
  `docs-site/src/content/docs/design/accepted/ui-lab-preview-lab-runtime-evidence-design.md`.
- Active productization design:
  `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`.
- Completed PM005 closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-005-persistence-project-io-diff-apply-and-rollback/closeout.md`.

Current implementation sources to inspect before product code changes:

- `apps/runenwerk_editor/src/shell/tests.rs`
- `apps/runenwerk_editor/src/shell/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs`
- `apps/runenwerk_editor/src/shell/editor_lab_project/mod.rs`
- `apps/runenwerk_editor/src/runtime/resources.rs`
- `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`
- `domain/editor/editor_shell/src/surfaces/editor_definition.rs`

## Promotion Readiness

`task production:plan -- --milestone PM-UI-LAB-006 --roadmap WR-087` reports
WR-087 as `write_promotion_contract`, with promotion preflight status
`promotable`.

Promotion is honest because:

- `PM-UI-LAB-005` is completed with runtime-proven project IO, diff/apply,
  activation report, failed activation preservation, reload, and rollback
  evidence.
- `WR-086` is completed and archived as the direct implementation prerequisite.
- `PM-UI-LAB-006` has an accepted Preview Lab runtime evidence design.
- `WR-087` has bounded write scopes and explicit PM007/game-runtime
  non-goals.
- No product code is required before promotion.

Use this evidence string for promotion:

```text
Accepted PM-UI-LAB-006 Preview Lab runtime evidence design plus completed PM-UI-LAB-005 runtime-proven project IO, diff/apply, activation report, reload, and rollback closeout clear WR-087 for current-candidate implementation planning; evidence capture remains app-owned and ui_definition remains behavior-free.
```

Suggested command after this contract validates:

```text
task roadmap:promote -- --id WR-087 --state current_candidate --evidence "Accepted PM-UI-LAB-006 Preview Lab runtime evidence design plus completed PM-UI-LAB-005 runtime-proven project IO, diff/apply, activation report, reload, and rollback closeout clear WR-087 for current-candidate implementation planning; evidence capture remains app-owned and ui_definition remains behavior-free."
```

Do not run product-code implementation before promotion and a subsequent
`task ai:goal` rerun select a legal implementation action.

## Architecture Decisions

Source-truth decisions:

- `apps/runenwerk_editor` owns Preview Lab scenario execution, provider
  mounting, retained surface formation, native screenshot capture if supported,
  artifact writing, accessibility checks, performance measurements,
  unsupported-check diagnostics, and evidence manifests.
- `domain/editor/editor_shell` owns retained surface composition and typed
  Editor Lab view-model projection. It does not own app sessions, provider
  execution, artifact storage, screenshot capture, or performance runners.
- `domain/editor/editor_definition` may own runtime-neutral descriptors only
  when they contain no concrete file paths, app sessions, provider handles,
  screenshots, artifact paths, or runtime resources.
- `domain/ui/ui_definition` remains behavior-free. It may provide generic UI
  diagnostics and retained structures, but it must not execute scenarios,
  own screenshot truth, run accessibility tooling, write artifacts, or measure
  runtime performance.
- The evidence manifest is the source truth for closeout artifacts. Individual
  retained UI dumps, screenshots, diagnostics files, and performance reports
  are referenced artifacts, not independent completion claims.

Forbidden shortcuts:

- claiming descriptor-only, docs-only, status-panel-only, raw console-only, or
  static fixture-only evidence as runtime proof;
- treating unsupported screenshot, accessibility, or performance checks as
  passing checks;
- writing broad screenshot or artifact logic in `domain/ui/ui_definition`;
- hiding failed scenario inputs, diagnostics, or artifact paths;
- closing PM006 with only the PM002-PM005 narrow artifacts;
- claiming PM007 public API ergonomics, usage docs, examples, final track
  closeout, or no-gap certification under WR-087.

## Implementation Scope

### Preview Scenario Catalog

Add an app-owned scenario catalog for Editor Lab V1. The catalog should define
stable scenario ids, labels, target profile, setup path, expected state family,
provider requirements, expected diagnostics, capture requirements,
accessibility requirements, performance expectations, and known unsupported
checks.

Suggested app module shape, subject to nearby code inspection:

```text
apps/runenwerk_editor/src/shell/editor_lab_evidence/
|-- mod.rs
|-- scenario.rs
|-- manifest.rs
|-- accessibility.rs
|-- performance.rs
|-- artifact.rs
`-- runner.rs
```

If the implementation keeps the module under `apps/runenwerk_editor/src/shell`
but chooses a different subdomain name, the closeout must name the final
module and why it is the owning app boundary.

### Scenario Matrix

The first PM006 implementation must cover these state families:

- success: Editor Lab opens, provider surfaces mount, and retained visual or
  screenshot-equivalent artifact is non-empty;
- warning: non-blocking diagnostics are visible in diagnostics or review
  output;
- error: invalid project package or invalid editor definition produces typed
  diagnostics and preserves input;
- reload: saved project package reloads without live activation;
- apply: accepted apply produces review, queued activation, runtime activation
  report, and provider-surface visibility;
- rollback: snapshot-backed rollback records a typed rollback report and
  provider-surface visibility;
- degraded-provider: missing or unsupported provider state renders a typed
  degraded surface with recovery actions or disabled reasons;
- accessibility: enabled and disabled controls expose labels and disabled
  reasons; unsupported accessibility checks are explicit diagnostics;
- performance: supported formation/capture timings and artifact sizes are
  recorded; unsupported GPU or native-window timing is explicit diagnostics.

### Evidence Manifest And Artifacts

Add an `EditorLabEvidenceManifest` path that records:

- scenario id and target profile;
- setup commands or operations;
- package/review/activation/rollback references where applicable;
- provider state summary;
- diagnostics snapshot path;
- retained visual/debug artifact path;
- native screenshot path when supported;
- accessibility snapshot path;
- performance snapshot path;
- unsupported-check diagnostics;
- result status and known gap classification.

Artifact paths should live under:

```text
docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/
```

Do not add closeout files to WR087 write scopes until the files exist.

### Visual Or Equivalent Proof

Use the PM006 design priority order:

1. Native screenshot capture when the current runtime and environment support
   it reliably.
2. Retained UI artifact plus provider view-model snapshot when native capture
   is unavailable in the current headless test environment.
3. Unsupported-check diagnostic when neither visual path is available.

The implementation must make the chosen path explicit per scenario. A retained
UI artifact is acceptable only when it is generated by the app-hosted Editor
Lab path and proves non-empty, scenario-specific surface output.

### Accessibility And Performance Snapshots

Accessibility snapshots must inspect runtime-formed Editor Lab controls where
available. At minimum they should prove labels and disabled reasons for normal
command controls and record unsupported checks as typed diagnostics.

Performance snapshots must record supported measurements such as scenario setup
duration, retained surface formation duration, artifact count, artifact bytes,
and optional native capture duration. Unsupported native-window or GPU timing
must be typed unsupported diagnostics, not omitted or marked pass.

### Provider And Runtime Boundaries

Provider degradation, activation, apply, rollback, and package behavior must
use the app/editor runtime path that PM002-PM005 established:

- `SelfAuthoringWorkspaceState` remains the app-owned Editor Lab state hub.
- `EditorLabDocumentStore` remains the project package and apply-review source
  boundary.
- `EditorHostResource::apply_pending_editor_definition_activations` remains
  the live activation owner.
- `build_editor_lab_surface` remains retained composition support, not
  evidence execution ownership.

## Implementation Steps

1. Re-inspect PM005 runtime tests and artifacts before editing.
2. Add an app-owned evidence module with scenario, manifest, artifact,
   accessibility, performance, and runner contracts.
3. Create deterministic scenario fixtures for success, warning, error, reload,
   apply, rollback, degraded-provider, accessibility, and performance states.
4. Execute scenarios through the app-hosted Editor Lab state and provider
   projection path, not isolated descriptors.
5. Generate retained visual/debug artifacts and provider snapshots for every
   scenario; add native screenshots only if deterministic support exists.
6. Add diagnostics, activation, package, review, rollback, accessibility, and
   performance artifact writers.
7. Add manifest validation so every required scenario has a result, artifact
   links, and explicit unsupported-check records.
8. Add focused tests and the artifact-writing runtime test.
9. Create PM006 closeout only after runtime artifacts exist and tests pass.
10. Update roadmap, production, and generated docs after the closeout evidence
    is truthful.

If native screenshot capture, accessibility checks, and performance snapshots
cannot stay bounded in one app-owned implementation, stop and split WR087
before product code continues.

## Runtime Evidence Contract

WR087 closeout must include a generated evidence manifest and artifacts proving:

- success, warning, error, reload, apply, rollback, degraded-provider,
  accessibility, performance, and unsupported-check scenario execution;
- non-empty retained visual or screenshot-equivalent artifacts for every
  scenario where native screenshots are unavailable;
- typed diagnostics snapshots for warning, error, degraded-provider, and
  unsupported-check paths;
- app-owned package/review/activation/rollback artifacts for reload, apply,
  and rollback scenarios;
- accessibility snapshots for labels, disabled reasons, focus/control support
  where available, and explicit unsupported checks;
- performance snapshots for supported timings and artifact sizes;
- manifest validation that fails if a required scenario is missing or only
  static/descriptive evidence exists.

## Acceptance Criteria

- `EditorLabPreviewScenario` and `EditorLabEvidenceManifest` are app-owned and
  typed.
- Evidence runs are generated from the app-hosted Editor Lab path.
- Every required state family has a scenario entry, result status, diagnostics
  snapshot, and visual or equivalent artifact.
- Unsupported screenshot, accessibility, and performance checks are typed
  diagnostics and listed as gaps or environment limits.
- Degraded-provider behavior renders a typed degraded state with recovery
  actions or disabled reasons.
- Runtime evidence is reproducible through a focused artifact-writing test.
- `ui_definition` remains behavior-free and does not gain app scenario,
  screenshot, accessibility, performance, artifact writing, or provider
  execution behavior.

## Validation

Minimum validation before WR087 closeout:

```text
cargo fmt
cargo test -p runenwerk_editor editor_lab
cargo test -p runenwerk_editor pm_ui_lab_006
$env:RUNENWERK_WRITE_PM_UI_LAB_006_EVIDENCE='1'; cargo test -p runenwerk_editor pm_ui_lab_006_runtime_evidence_reports_preview_lab -- --nocapture
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
```

Focused tests should prove:

- scenario ids and matrix completeness are deterministic;
- manifest validation fails when required scenarios, artifacts, diagnostics, or
  unsupported-check records are missing;
- retained visual or screenshot-equivalent artifacts are non-empty and
  scenario-specific;
- warning/error diagnostics preserve input and expected failure context;
- reload/apply/rollback scenarios use PM005 project/apply/activation/rollback
  paths;
- degraded-provider scenarios expose typed provider state and recovery actions
  or disabled reasons;
- accessibility snapshots include labels and disabled reasons where supported;
- performance snapshots include supported timings and artifact sizes;
- unsupported screenshot/accessibility/performance checks are explicit
  diagnostics.

## Closeout Requirements

- Create
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md`
  only after runtime-proof artifacts and focused tests pass.
- Store PM006 runtime artifacts under that closeout directory, including the
  evidence manifest, retained visual or screenshot artifacts, diagnostics
  snapshots, accessibility snapshots, performance snapshots, activation/apply
  reports, rollback reports, and unsupported-check reports.
- Add the closeout path and artifact paths to WR087 write scopes only when the
  files exist and the validator accepts the scope.
- Move WR087 to `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
  and mark PM-UI-LAB-006 completed only after evidence supports
  `runtime_proven`.
- Record PM007 as the remaining known gap for public API ergonomics, usage
  docs, examples, and final track closeout.
- Run roadmap, production, docs, planning, and PUML gates after evidence edits
  and after moving WR087 out of active roadmap items.
- Rerun `task ai:goal -- --track PT-UI-LAB --scope non-deferred` after
  closeout and follow the next milestone action instead of starting PM007 from
  memory.

## Perfectionist Closeout Audit

WR087 may close PM006 as `runtime_proven` if the Preview Lab scenario matrix,
visual or equivalent artifacts, diagnostics snapshots, accessibility reports,
performance reports, degraded-provider proof, manifest validation, and runtime
artifact generation are proven end to end. It must not claim
`perfectionist_verified`; the final no-gap audit belongs to
`PT-UI-LAB-PERFECTION` or an equivalent later track after PM-UI-LAB-007.

Known quality gaps that must remain visible if WR087 closes successfully:

- PM007 still owns public API ergonomics review, usage docs, examples, and the
  final PT-UI-LAB runtime-proven closeout.
- Game-runtime UI projection execution remains out of Editor Lab V1 scope.
- Native screenshot capture may remain an environment-limited gap only if the
  implementation records retained visual equivalents and typed unsupported
  diagnostics.
- A later perfectionist audit must re-check module structure, UI ergonomics,
  evidence breadth, visual fidelity, accessibility depth, performance budgets,
  and no-gap completion claims.

## Non-Goals

- No PM007 public API review, usage guide, examples, or final track closeout.
- No game-runtime UI projection execution.
- No no-gap or `perfectionist_verified` claim.
- No moving app evidence execution, screenshot capture, accessibility tooling,
  performance measurement, provider sessions, or artifact writing into
  `domain/ui/ui_definition`.
- No treating unsupported screenshot/accessibility/performance checks as
  passing evidence.
- No broad renderer or GPU visual-diff platform work beyond the current Editor
  Lab runtime evidence needs.

## Stop Conditions

Stop implementation if:

- PM006 would require `ui_definition` to execute app/runtime/provider behavior;
- evidence would be descriptor-only, docs-only, static-fixture-only,
  status-panel-only, or raw console-only;
- failed scenarios would not preserve input, diagnostics, or artifact links;
- native screenshot capture is required but cannot be made deterministic and no
  equivalent retained visual artifact is accepted by this contract;
- accessibility or performance checks are claimed without actual reports or
  typed unsupported diagnostics;
- degraded-provider scenarios cannot be represented with typed state;
- the work cannot remain bounded without pulling in PM007 or game-runtime
  projection scope;
- the implementation needs a durable ownership or dependency-direction change
  not covered by the accepted PM006 design or existing ADRs.
