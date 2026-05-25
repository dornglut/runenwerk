---
title: WR-105 UI Lab Runtime Evidence Platform Closure Contract
description: Current-candidate implementation contract for PM-UI-LAB-PERF-002 native-or-typed-impossible Editor Lab runtime evidence closure.
status: active
owner: editor
layer: app/runtime-evidence
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/accepted/ui-lab-runtime-evidence-platform-closure-design.md
  - ../../../design/accepted/ui-lab-perfectionist-audit-design.md
  - ../../../design/accepted/ui-lab-preview-lab-runtime-evidence-design.md
related_reports:
  - ../../closeouts/pm-ui-lab-perf-001-governance-audit-doctrine-and-code-truth-matrix/closeout.md
  - ../../closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md
  - ../../roadmap-intake/2026-05-25-pm-ui-lab-perf-002-runtime-evidence-plat/proposal.yaml
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-105 UI Lab Runtime Evidence Platform Closure Contract

## Goal

Implement `PM-UI-LAB-PERF-002` by replacing PM006 free-form unsupported
evidence gaps with typed captured-or-platform-impossible evidence results.

WR-105 is now a `current_candidate` row selected by
`task ai:goal -- --track PT-UI-LAB-PERFECTION` for the bounded implementation
contract. Product code changes are allowed only inside the implementation scope
below, with PM003 through PM006 left untouched except for explicit known-gap
references in closeout evidence.

## Source Of Truth

- Production milestone: `PM-UI-LAB-PERF-002` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-105` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted PM002 design:
  `docs-site/src/content/docs/design/accepted/ui-lab-runtime-evidence-platform-closure-design.md`.
- Accepted no-gap doctrine:
  `docs-site/src/content/docs/design/accepted/ui-lab-perfectionist-audit-design.md`.
- Completed PM001 closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-001-governance-audit-doctrine-and-code-truth-matrix/closeout.md`.
- PM006 runtime-proven input:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md`.

Current implementation sources to inspect before code changes:

- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs`
- `apps/runenwerk_editor/src/shell/tests.rs`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/artifacts/unsupported-checks.ron`

## Current-Candidate Readiness

`task production:plan -- --milestone PM-UI-LAB-PERF-002 --roadmap WR-105`
reported the promotion preflight as promotable before WR-105 was promoted:

- action: `write_promotion_contract`;
- promotion preflight: `promotable`;
- dependency state: `WR-100:completed`.

The current implementation action is honest because:

- PM001 is completed with accepted no-gap doctrine and completed closeout
  evidence.
- PM002 has an accepted runtime evidence platform closure design.
- WR-105 has bounded app-owned evidence write scopes and explicit non-goals.
- The row was promoted to `current_candidate` with the evidence below, so this
  contract may implement only the PM002 runtime evidence platform closure.

Recorded promotion evidence:

```text
Accepted PM-UI-LAB-PERF-002 runtime evidence platform closure design plus completed WR-100 no-gap governance closeout clear WR-105 for current-candidate implementation planning; evidence execution remains app-owned, ui_definition remains behavior-free, and unsupported checks must become captured or typed platform-impossible results.
```

## Architecture Decisions

Source-truth decisions:

- `apps/runenwerk_editor` owns evidence execution, capability probes, artifact
  writing, native screenshot adapters, focus traversal inspection, contrast
  sampling, timing capture, GPU visual-diff integration if available, and
  platform-impossible diagnostics.
- `domain/editor/editor_shell` owns retained UI composition and app-neutral
  view models only.
- `domain/editor/editor_definition` owns runtime-neutral document and
  operation report data consumed by evidence, but not runtime evidence
  execution.
- `domain/ui/ui_definition` remains behavior-free and owns diagnostics,
  retained structures, and validation contracts only.
- PM006 evidence artifacts are input evidence, not no-gap proof for PM002.

Forbidden shortcuts:

- moving screenshot, focus, contrast, timing, provider sessions, or artifact
  writing into `ui_definition`;
- treating a free-form unsupported string as no-gap evidence;
- accepting retained-only artifacts when a native capability probe says the
  native path is available;
- claiming PM003 command/surface closure, PM004 UX closure, PM005 API
  ergonomics, or PM006 final no-gap certification from this row.

## Implementation Scope

Expected implementation files:

- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs`
- `apps/runenwerk_editor/src/shell/tests.rs`

Expected evidence and docs:

- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-002-runtime-evidence-platform-closure/closeout.md`
- PM002 runtime artifacts under that closeout directory.
- Roadmap and production metadata/rendered docs after closeout.

Expected contract changes:

- Add typed capability, probe, result, and platform-impossible result models.
- Extend artifact kinds for native screenshot, GPU visual diff, focus
  traversal, contrast sample, timing, retained UI, diagnostics, activation,
  rollback, and platform-impossible reports.
- Add manifest validation requiring every no-gap target to have captured,
  platform-impossible, or failed status.
- Convert PM006 unsupported checks into typed PM002 no-gap evidence results.
- Add artifact-writing PM002 evidence test and closeout artifacts.

## Acceptance Criteria

- Descriptor-only, docs-only, status-panel-only, and free-form unsupported-only
  evidence is rejected.
- Every no-gap evidence capability has a captured or typed
  platform-impossible result, or the scenario fails.
- Platform-impossible results include capability, backend/environment,
  support status, reason, and reproduction command.
- PM002 artifacts cover visual truth, focus, contrast, timing, diagnostics,
  degraded-provider behavior, reload, apply, rollback, and failure
  preservation.
- App-owned evidence remains in `apps/runenwerk_editor`.
- `ui_definition` remains behavior-free and `editor_definition` remains
  runtime-neutral.

## Validation

Implementation validation:

```text
cargo fmt
cargo test -p runenwerk_editor editor_lab_evidence
cargo test -p runenwerk_editor pm_ui_lab_perf_002
$env:RUNENWERK_WRITE_PM_UI_LAB_PERF_002_EVIDENCE='1'; cargo test -p runenwerk_editor pm_ui_lab_perf_002_runtime_evidence_platform_closure -- --nocapture
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
task ai:goal -- --track PT-UI-LAB-PERFECTION
```

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-002-runtime-evidence-platform-closure/closeout.md`
only after focused tests and artifact generation pass.

The closeout must include:

- reproduction commands;
- artifact manifest paths;
- native captured evidence or typed platform-impossible diagnostics for every
  no-gap target;
- validation output;
- remaining known quality gaps that belong to PM003 through PM006;
- roadmap archive and production milestone updates only after validation.

## Perfectionist Closeout Audit

WR-105 may close PM002 at `runtime_proven` only if the runtime evidence
platform proves captured or typed platform-impossible evidence for every PM002
target. It must not claim `perfectionist_verified`.

Remaining gaps after PM002 are expected:

- command/surface source-of-truth closure remains PM003;
- direct-manipulation UX closure remains PM004;
- persistence/API/examples ergonomics closure remains PM005;
- final no-gap certification remains PM006.

## Stop Conditions

Stop implementation if:

- promotion preflight is no longer promotable;
- native evidence ownership cannot stay app-owned;
- platform-impossible results cannot be backed by typed probe metadata;
- the evidence path would be descriptor-only, status-panel-only, or console-only;
- implementation requires a reusable cross-domain evidence platform without an
  accepted ADR or design update;
- the row starts PM003, PM004, PM005, or PM006 scope.
