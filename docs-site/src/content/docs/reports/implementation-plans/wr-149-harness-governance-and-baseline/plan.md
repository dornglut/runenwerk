---
title: WR-149 Harness Governance And Baseline Implementation Contract
description: Governance-only contract for activating PT-TRACK-EXECUTION-HARNESS before workflow implementation milestones begin.
status: active
owner: workspace
layer: workspace / production workflow
canonical: false
last_reviewed: 2026-06-01
related_designs:
  - ../../../workspace/track-execution-manifest.md
  - ../../../workspace/planning-and-implementation-workflow.md
  - ../../../workspace/design-track-roadmap-governance.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# WR-149 Harness Governance And Baseline Implementation Contract

## Goal

Activate `PT-TRACK-EXECUTION-HARNESS` as the dedicated production track for
proving the generic locked-track execution harness before it is trusted to run
`PT-UI-PROGRAM-ARCHITECTURE`.

This is a governance-only milestone. It proves that the harness has its own
track, manifest, WR authority, truth claim, generated registers, and next-action
gate. It does not implement workflow code, UI product code, crates,
MaterialProgram work, placeholder folders, or `foundation/meta`.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml`: structured
  production-track metadata for `PT-TRACK-EXECUTION-HARNESS`.
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml`: deferred WR
  authority for `WR-149`.
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-track-execution-harness.yaml`:
  machine-readable manifest and blocked harness truth claim.
- `docs-site/src/content/docs/workspace/track-execution-manifest.md`:
  repository workflow contract for manifest-backed track execution.
- `docs-site/src/content/docs/workspace/design-track-roadmap-governance.md`:
  authority ladder for designs, tracks, manifests, WRs, plans, closeouts, and
  truth claims.

## Readiness

`task production:plan -- --milestone PM-TRACK-HARNESS-001 --roadmap WR-149`
reported:

- production milestone state: `active`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- roadmap dependencies: none;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-149-harness-governance-and-baseline/plan.md`.

This contract clears the first design-first gap by making the activation scope,
non-goals, validation, and closeout requirements explicit.

## Authority

`WR-149` may update only governance and planning artifacts:

- `docs-site/src/content/docs/workspace/production-tracks.yaml`;
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml`;
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`;
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-track-execution-harness.yaml`;
- generated production and roadmap registers;
- this implementation plan;
- the PM-001 governance closeout report.

`WR-149` must not authorize workflow implementation, product code, runtime
behavior changes, crate creation, placeholder future folders, MaterialProgram
implementation, or `foundation/meta` extraction.

## Required Decisions

- The harness completion work has its own production track:
  `PT-TRACK-EXECUTION-HARNESS`.
- Harness truth starts blocked. Passing tests alone is not enough; the track
  must prove source-model closure, Contract Pack authority, locks,
  transactional executors, resolver-backed evidence, structured closeouts,
  agent-track, public command cutover, and legacy retirement.
- `PT-UI-PROGRAM-ARCHITECTURE` remains blocked behind the harness truth claim
  for full-track execution.
- MaterialProgram planning remains blocked until the relevant truth gates are
  explicitly satisfied or a later accepted decision changes the gate.
- PM-001 closeout may claim only `bounded_contract`.

## Non-Goals

PM-001 does not:

- implement or refactor `tools/workflow`;
- delete or retire the legacy manifest runner;
- create an Execution Lock or claim full automation readiness;
- execute `PT-UI-PROGRAM-ARCHITECTURE`;
- start MaterialProgram;
- create crates or placeholder folders;
- extract `foundation/meta`.

## Acceptance Criteria

- `PT-TRACK-EXECUTION-HARNESS` exists in production metadata.
- `WR-149` exists as the owning governance WR for
  `PM-TRACK-HARNESS-001`.
- The machine-readable harness manifest exists and names all PM-001 through
  PM-011 milestones.
- The harness truth claim remains blocked until runtime and architecture proof
  evidence exists.
- Generated production and roadmap docs are rendered and in sync.
- `/goal` and `production:next` report PM-001 as the next legal action and do
  not authorize implementation.

## Validation

Run:

```text
uv run pytest tools/workflow/test_workflow.py -q
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task docs:validate
task planning:validate
git diff --check
```

## Closeout Requirements

Close PM-001 only with a closeout report at:

```text
docs-site/src/content/docs/reports/closeouts/pm-track-harness-001-harness-governance-and-baseline/closeout.md
```

The closeout must record:

- files changed;
- validation results;
- confirmation that no product code, crates, placeholder folders,
  MaterialProgram work, or `foundation/meta` extraction occurred;
- the remaining blocked harness truth claim;
- next legal action: PM-TRACK-HARNESS-002 planning, not direct implementation
  without WR and plan authority.
