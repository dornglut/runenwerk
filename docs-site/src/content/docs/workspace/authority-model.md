---
title: Authority Model
description: Authority model for resolving Runenwerk code, docs, planning, report, generated-view, and tooling conflicts.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ./start-here.md
  - ./operating-model.md
  - ./documentation-structure.md
  - ./workflow-lifecycle.md
  - ./complete-investigation-gate.md
  - ./complete-design-gate.md
  - ./evidence-quality-taxonomy.md
  - ./complete-merge-readiness-gate.md
  - ../guidelines/programming-principles.md
---

# Authority Model

Every repository artifact has one job.

## Authority order

1. Code, tests, fixtures, captures, and runtime evidence own current behavior.
2. Accepted ADRs, accepted designs, guidelines, and root architecture docs own durable architecture direction.
3. Workspace authority docs own repository process.
4. Complete investigation gate docs own mandatory investigation evidence requirements.
5. Complete design gate docs own mandatory design/planning readiness requirements.
6. Evidence quality taxonomy owns evidence classes, confidence, freshness, and validation wording.
7. Complete merge readiness gate owns merge, branch cleanup, and post-merge truth requirements.
8. Planning Markdown owns active, deferred, completed, and strategic planning state.
9. Routines own repeatable work procedure.
10. Task cards own reusable task wording.
11. Reports and closeouts own historical evidence.
12. Generated views and local helpers are convenience only unless a machine contract explicitly says otherwise.

## Lifecycle authority

Use [`workflow-lifecycle.md`](workflow-lifecycle.md) when resolving whether a work item is an idea, investigation, proposed design, accepted direction, production track, active planning, active implementation, review, completed work, deferred work, rejected work, superseded work, or archive material.

Accepted architecture direction does not authorize implementation by itself. Active implementation requires a planning entry with owner, complete implementation contract, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, stop conditions, and complete investigation/design gate evidence where applicable.

## Complete investigation gate authority

Use [`complete-investigation-gate.md`](complete-investigation-gate.md) when a work item needs evidence about current reality, authority, ownership, vocabulary, capability inventory, alternatives, confidence, or missing evidence before design/planning/implementation decisions.

The gate owns the required investigation dossier shape and investigation matrix templates. It does not own track-specific architecture decisions, current planning state, or historical closeout evidence.

## Complete design gate authority

Use [`complete-design-gate.md`](complete-design-gate.md) when a work item touches architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary concerns.

The gate owns the required checklist shape and matrix templates. It does not own track-specific architecture decisions, current planning state, or historical closeout evidence.

## Evidence quality authority

Use [`evidence-quality-taxonomy.md`](evidence-quality-taxonomy.md) when a claim depends on validation, current behavior, freshness, confidence, connector inspection, local commands, CI, generated proof artifacts, user-reported validation, or inference.

The taxonomy owns evidence classes and reporting wording. It does not own the underlying code behavior, policy decision, or planning state.

## Merge readiness authority

Use [`complete-merge-readiness-gate.md`](complete-merge-readiness-gate.md) before recommending merge, branch deletion, phase merge, or post-merge cleanup.

The merge gate owns merge readiness shape. It does not replace PR review, CI, implementation evidence, or phase closeout truth.

## Generated file classes

Generated or machine-readable files may have one of three roles:

```text
mirror
  convenience copy of Markdown authority

evidence
  generated proof/report from code, tests, validators, captures, or local tools

contract
  machine-readable authority only when an accepted design explicitly grants that status for a scope
```

When a generated file and Markdown planning disagree, use the Markdown planning record unless the generated file is an explicitly accepted contract for that exact claim.

## Programming-principle lens

Use `docs-site/src/content/docs/guidelines/programming-principles.md` when authority conflicts are caused by over-complexity, duplication, speculation, blurred responsibility, premature optimization, or cross-boundary coupling.

In practice:

- KISS: prefer the simplest authority path that protects the invariant.
- DRY: do not keep the same durable claim authoritative in two places.
- YAGNI: do not preserve legacy workflow surfaces only because they might be useful someday.
- SOLID: keep responsibilities and dependencies honest.
- Separation of Concerns: separate entrypoints, authority docs, lifecycle, complete investigation gates, complete design gates, evidence quality, merge readiness, routines, planning, reports, and tooling.
- Avoid Premature Optimization: do not add generated views or scripts before there is evidence they solve a real problem.
- Law of Demeter: route through direct owners and explicit contracts.

## Planning files

Use these Markdown planning files from this cutover onward:

```text
docs-site/src/content/docs/workspace/planning/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/deferred-work.md
docs-site/src/content/docs/workspace/planning/completed-work.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Legacy structured files and generated Markdown may remain as migration context or optional mirrors. They are not required to understand active workflow.

## Conflict rule

When two files disagree, update the file that owns the disputed claim type. Do not update the convenient duplicate first.

Examples:

- Root summary conflicts with workspace process: update the workspace authority doc, then align the root summary.
- Code conflicts with accepted design: decide whether code drifted or the design needs intentional revision.
- Planning Markdown conflicts with a generated view: use Planning Markdown and report the generated view as stale.
- Task card conflicts with a routine: use the routine.
- Local helper conflicts with a routine: use the routine.
- Proposed design conflicts with accepted design: use the accepted design until a decision record supersedes it.
- Implementation authorization lacks complete investigation or design gate evidence where required: update investigation, design, or planning authority before coding.
- Merge recommendation lacks evidence quality or merge readiness evidence: continue review before recommending merge.
