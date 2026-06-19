---
title: UI Lab Final No Gap Certification Closeout Design
description: Accepted design for PM-UI-LAB-PERF-006 final Editor Lab V1 no-gap certification, drift audit, evidence reconciliation, and perfectionist_verified claim rules.
status: accepted
owner: editor
layer: app/domain/docs
canonical: true
last_reviewed: 2026-05-25
related_adrs:
  - ../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/superseded/0006-editor-surface-provider-plugin-seam.md
  - ../../adr/accepted/0010-graph-substrate-canvas-boundary.md
  - ../../adr/superseded/0012-capability-workbench-clean-break.md
related_designs:
  - ./ui-lab-perfectionist-audit-design.md
  - ./ui-lab-runtime-evidence-platform-closure-design.md
  - ./ui-lab-command-surface-source-truth-closure-design.md
  - ./ui-lab-direct-manipulation-ux-closure-design.md
  - ./ui-lab-persistence-api-examples-ergonomics-closure-design.md
related_reports:
  - ../../reports/closeouts/pm-ui-lab-perf-001-governance-audit-doctrine-and-code-truth-matrix/closeout.md
  - ../../reports/closeouts/pm-ui-lab-perf-002-runtime-evidence-platform-closure/closeout.md
  - ../../reports/closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md
  - ../../reports/closeouts/pm-ui-lab-perf-004-direct-manipulation-ux-closure/closeout.md
  - ../../reports/closeouts/pm-ui-lab-perf-005-persistence-diff-apply-api-and-examples-ergonomics/closeout.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# UI Lab Final No Gap Certification Closeout Design

## Status

Accepted for `PM-UI-LAB-PERF-006`.

This design clears only the PM006 final-certification design gate. It does not
authorize product code, mark the milestone complete, or assign
`perfectionist_verified`. A final claim still requires a legal roadmap row or
workflow-approved closeout path, `task production:plan` when a WR row is linked,
validated closeout evidence, updated production and roadmap metadata, and a
rerun of `task ai:goal -- --track PT-UI-LAB-PERFECTION`.

## Goal

PM006 is the final audit for the completed Editor Lab V1 perfectionist track.
It must answer one question: do code truth, runtime artifacts, public APIs,
usage docs, examples, generated roadmap state, production state, and previous
milestone closeouts all agree with zero known quality gaps?

The certification chain is:

```text
PM001 governance doctrine
  -> PM002 runtime evidence platform closure
  -> PM003 command and surface source-truth closure
  -> PM004 direct-manipulation UX closure
  -> PM005 persistence, diff/apply, public API, and examples closure
  -> PM006 final audit, drift check, metadata reconciliation, and no-gap claim
```

The completed `PT-UI-LAB` track remains `runtime_proven` input. PM006 must not
reopen it to justify stronger claims and must not expand into game-runtime UI
projection.

## Current Evidence Inputs

PM006 may rely on earlier milestones only after directly verifying their
completed closeouts and quality claims:

| Milestone | Closeout | Quality input | PM006 verification duty |
|---|---|---|---|
| `PM-UI-LAB-PERF-001` | `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-001-governance-audit-doctrine-and-code-truth-matrix/closeout.md` | `bounded_contract` | Confirm governance doctrine, code-truth matrix, follow-on scope, and explicit non-claim of runtime or no-gap quality. |
| `PM-UI-LAB-PERF-002` | `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-002-runtime-evidence-platform-closure/closeout.md` | `runtime_proven` | Confirm native or typed platform-impossible evidence artifacts cover screenshots, visual diffs, focus, contrast, timing, diagnostics, reload, apply, rollback, and degraded-provider states. |
| `PM-UI-LAB-PERF-003` | `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md` | `runtime_proven` | Confirm command catalog, toolbar/menu/keybinding, disabled reason, surface registry, routing, and compatibility edges have one normal authority. |
| `PM-UI-LAB-PERF-004` | `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-004-direct-manipulation-ux-closure/closeout.md` | `runtime_proven` | Confirm hierarchy, palette, canvas, inspector, diagnostics, operation diff, preview, undo, and redo evidence proves direct-manipulation workflows. |
| `PM-UI-LAB-PERF-005` | `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-005-persistence-diff-apply-api-and-examples-ergonomics/closeout.md` | `runtime_proven` | Confirm persistence, structural diff/apply, activation, rollback, public API, guides, examples, and product-surface evidence agree. |

Any missing artifact, stale validation, ambiguous quality tier, or unclosed
known gap blocks PM006 completion.

## Architecture Governance

Architecture governance for this design-only action:

```text
task ai:architecture-governance -- --task "PM-UI-LAB-PERF-006 final no-gap certification closeout design" --scope "Editor Lab final no-gap certification, completed milestone evidence audit, runtime artifacts, API/docs/examples agreement, roadmap and production state agreement, phase drift-check evidence, known quality gaps, and perfectionist_verified claim rules; design-only action, no product code"
```

Governance decisions:

- DDD bounded context owner: `editor`.
- App owner: `apps/runenwerk_editor` owns concrete Editor Lab runtime evidence,
  provider sessions, app-hosted project IO, artifact generation, and product
  surface snapshots.
- Domain owners: `domain/ui/ui_definition` owns behavior-free UI definition
  contracts; `domain/editor/editor_definition` owns reusable editor definition,
  validation, operation report, package, and review DTO contracts;
  `domain/editor/editor_shell` owns editor shell projection contracts.
- Docs and workflow owners: docs under `docs-site/src/content/docs`, roadmap
  YAML, production YAML, and generated planning docs own certification evidence
  traceability. They do not create runtime truth.
- Vocabulary: final no-gap certification, drift check, completed closeout,
  evidence gate, runtime artifact, platform-impossible result, source truth,
  structural apply review, public API ergonomics, known quality gap, and
  `perfectionist_verified`.
- Clean Architecture direction: final audit may read and reconcile contracts
  across app, domain, docs, roadmap, and production state, but it must not move
  app execution into domain crates or make docs/generated state authoritative
  over code or runtime evidence.
- ADR need: no new ADR is required for a certification closeout that preserves
  the accepted ownership model. Add an ADR or accepted design update before
  changing source-truth authority, persisted public formats, dependency
  direction, or runtime evidence ownership.
- ATAM-lite priority order: truthfulness first, reproducibility second,
  source-truth consistency third, public usability fourth, metadata freshness
  fifth, compatibility sixth.
- Team Topologies ownership: stream-aligned editor product certification with
  complicated-subsystem support from UI definition, editor definition, app
  runtime evidence, and workspace planning owners.
- Recommended next action after this design validates: rerun
  `task ai:goal -- --track PT-UI-LAB-PERFECTION` and follow only the next legal
  PM006 action it reports.

## Certification Contract

PM006 closeout may claim `perfectionist_verified` only when all of the
following are true:

- `PM-UI-LAB-PERF-001` through `PM-UI-LAB-PERF-005` are completed in production
  metadata with existing evidence gates, completed closeouts, and honest
  completion-quality tiers.
- Every linked WR row for the completed milestones is completed or archived
  with matching closeout evidence and no stronger quality claim than its
  evidence supports.
- Runtime artifact paths named by the closeouts exist and the artifact contents
  match the claimed coverage.
- Public APIs, focused preludes, usage guides, examples, and product-surface
  evidence describe the same normal Editor Lab workflow.
- Command, surface, operation, persistence, diff/apply, rollback, and public API
  paths have one normal source of truth at their owning boundaries.
- Generic `ui_definition` logic remains behavior-free, and app execution,
  provider sessions, project IO, rollback, activation, and artifact writing stay
  app-owned.
- `task docs:validate`, `task puml:validate`, `task roadmap:render`,
  `task roadmap:validate`, `task roadmap:check`, `task production:render`,
  `task production:validate`, `task production:check`,
  `task planning:validate`, `git diff --check`, and the focused runtime and
  example commands selected by the closeout all pass.
- The phase completion drift-check routine is completed after the last
  implementation slice and before final certification is claimed.
- `known_quality_gaps` is empty for PM006 and for the completed production track
  claim.

If any item is false, PM006 must close as blocked or remain incomplete with the
exact gap recorded. It must not downgrade a real gap into wording that looks
like success.

## Required Audit Artifacts

The final closeout should create or refresh a bounded evidence pack under:

```text
docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-006-final-no-gap-certification-closeout/
```

Required contents:

- `closeout.md` with final status, scope, exact validation commands, results,
  completion-quality reasoning, and stop-condition review.
- A prerequisite milestone evidence matrix covering PM001 through PM005.
- A runtime artifact inventory that names each artifact path, owning milestone,
  evidence target, and verification result.
- A public API, guide, and example agreement review.
- A source-truth review for commands, surfaces, operations, persistence,
  diff/apply, rollback, and API entry points.
- Phase completion drift-check evidence.
- Final generated roadmap and production state evidence.
- The final `task ai:goal -- --track PT-UI-LAB-PERFECTION` result showing the
  track has no remaining legal incomplete milestone action.

## Required Fitness Functions

The final audit implementation contract or closeout must justify the exact
focused runtime commands. At minimum, it must include:

```text
cargo fmt
cargo test -p ui_definition
cargo test -p editor_definition
cargo test -p runenwerk_editor editor_lab_project
cargo test -p runenwerk_editor editor_lab
cargo test -p runenwerk_editor editor_definition_activation
cargo run -p ui_definition --example ui_definition_workflow
cargo run -p editor_definition --example editor_definition_workflow
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

Add PM-specific evidence-generation test commands when they are used as final
proof, especially the PM002 and PM005 artifact-writing tests.

## Roadmap And Production Handling

PM006 currently has no implementation authority from this design alone. If the
workflow requires a WR row for final closeout, create a bounded roadmap intake
for the audit and metadata work before changing product, production, or roadmap
completion state. If a linked WR row becomes active or `ready_next`, run:

```text
task production:plan -- --milestone "PM-UI-LAB-PERF-006" --roadmap "<WR-ID>"
```

The implementation contract must keep the write scope to final audit artifacts,
planning metadata, and validation evidence unless it discovers a real product
gap. A discovered product gap stops PM006 completion and must become a separate
legal follow-up scope; it must not be repaired inside the final certification
closeout unless workflow, WR scope, design gates, and validation explicitly
authorize that repair.

## Non-Goals

PM006 does not:

- implement new Editor Lab product behavior;
- reopen completed `PT-UI-LAB` milestones;
- expand into game-runtime UI projection;
- replace PM002 platform-impossible evidence with native screenshot or GPU
  visual-diff work unless a legal follow-up scope first accepts that platform
  decision;
- move app-owned project IO, activation, rollback, provider sessions, or
  artifact generation into `ui_definition` or `editor_definition`;
- claim `perfectionist_verified` when any known quality gap remains.

## Stop Conditions

Stop before completing PM006 if:

- ownership of any audited source of truth is unclear;
- a prerequisite closeout or artifact is missing, stale, or unverifiable;
- a completed WR row or production milestone claims a stronger quality tier
  than its evidence supports;
- final validation fails;
- roadmap or production render/check output is stale;
- a source file changes enough that `task ai:goal` must be rerun before
  continuing;
- any known quality gap remains.
