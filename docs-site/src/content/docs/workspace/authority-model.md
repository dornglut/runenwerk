---
title: Authority Model
description: Authority model for resolving Runenwerk code, docs, planning, report, generated-view, and tooling conflicts.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-27
related_docs:
  - ./start-here.md
  - ./operating-model.md
  - ./documentation-structure.md
  - ./workflow-lifecycle.md
  - ../guidelines/programming-principles.md
---

# Authority Model

Every repository artifact has one job.

## Authority order

1. Code, tests, fixtures, captures, and runtime evidence own current behavior.
2. Accepted ADRs, accepted designs, guidelines, and root architecture docs own durable architecture direction.
3. Workspace authority docs own repository process.
4. Planning Markdown owns active, deferred, completed, and strategic planning state.
5. Routines own repeatable work procedure.
6. Task cards own reusable task wording.
7. Reports and closeouts own historical evidence.
8. Generated views and local helpers are convenience only unless a narrow machine contract explicitly says otherwise.

## Lifecycle authority

Use [`workflow-lifecycle.md`](workflow-lifecycle.md) when resolving whether a work item is an idea, investigation, proposed design, accepted direction, production track, active planning, active implementation, review, completed work, deferred work, rejected work, superseded work, or archive material.

Accepted architecture direction does not authorize implementation by itself. Active implementation requires a planning entry with owner, scope, validation envelope, evidence expectation, and stop conditions.

## Generated file classes

Generated or machine-readable files may have one of three roles:

```text
mirror
  convenience copy of Markdown authority

evidence
  generated proof/report from code, tests, validators, captures, or local tools

contract
  machine-readable authority only when an accepted design explicitly grants that status for a narrow scope
```

When a generated file and Markdown planning disagree, use the Markdown planning record unless the generated file is an explicitly accepted contract for that exact claim.

## Programming-principle lens

Use `docs-site/src/content/docs/guidelines/programming-principles.md` when authority conflicts are caused by over-complexity, duplication, speculation, blurred responsibility, premature optimization, or cross-boundary coupling.

In practice:

- KISS: prefer the simplest authority path that protects the invariant.
- DRY: do not keep the same durable claim authoritative in two places.
- YAGNI: do not preserve legacy workflow surfaces only because they might be useful someday.
- SOLID: keep responsibilities and dependencies honest.
- Separation of Concerns: separate entrypoints, authority docs, lifecycle, routines, planning, reports, and tooling.
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
