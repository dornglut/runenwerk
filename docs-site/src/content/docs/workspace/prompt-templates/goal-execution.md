---
title: Goal Execution
description: Prompt template for production-track scoped Codex /goal coordination.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-20
related_docs:
  - ../planning-and-implementation-workflow.md
  - ../production-track-planning-model.md
  - ../production-tracks.yaml
  - ../roadmap-items.yaml
  - ../roadmap-archive.yaml
  - ../roadmap-deferred.yaml
---

# Goal Execution

Use this template when a Codex `/goal` should coordinate a full production
track, or the non-deferred portion of a production track, instead of a single
roadmap row.

Start with the read-only generator:

```text
task ai:goal -- --track <PT-ID>
task ai:goal -- --track <PT-ID> --scope non-deferred
```

Then paste the generated prompt into `/goal`.

## Template

```text
Complete production track <PT-ID>.

For bounded work that must preserve blocked or deferred milestones, generate
the prompt with `--scope non-deferred` instead. That prompt completes only
milestones that are not blocked or deferred and records preserved milestones as
explicit deferred gaps.

Use production-tracks.yaml as the production sequencing source, active
roadmap-items.yaml rows as the WR execution authority, and the archive/deferred
roadmap sources for historical links, dependencies, and evidence gates.

Work through the finite milestone list in dependency order.
For each milestone, perform exactly one legal next action, validate it, close it
out, then rerun task ai:goal before continuing.

Do not bypass design gates, ADR gates, WR roadmap state, write scopes,
validation, closeout evidence, or completion-quality rules.

The production track is complete only when every milestone is completed with
valid evidence gates, linked WR rows satisfy roadmap completion-quality rules,
and production plus roadmap render/validate/check gates pass.
```

## Stop Conditions

Stop and report instead of continuing when:

- a milestone dependency is incomplete;
- a design, ADR, WR, validation, or closeout gate is missing;
- a linked WR row is not ready for its required action;
- implementation would exceed the current milestone;
- validation or closeout evidence is missing;
- source files changed enough that the generated plan must be refreshed.
