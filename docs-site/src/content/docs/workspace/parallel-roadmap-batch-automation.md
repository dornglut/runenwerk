---
title: Parallel Roadmap Batch Automation
description: Coordinator workflow for selecting, approving, executing, integrating, and documenting parallel roadmap batches.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related:
  - ./planning-and-implementation-workflow.md
  - ./roadmap-items.yaml
  - ./schemas/roadmap-items.schema.json
  - ./schemas/batch-manifest.schema.json
  - ./roadmap-decision-register.md
  - ./design-implementation-triage.md
  - ./repo-execution-priority-checklist.md
  - ./diagrams/codex-roadmap-orchestration-target.puml
  - ./diagrams/parallel-roadmap-batch-automation.puml
  - ./diagrams/design-intake-roadmap-automation.puml
  - ./prompt-templates/parallel-roadmap-batch.md
  - ./routines/parallel-roadmap-batch-routine.md
  - ./architecture-governance-review.md
---

# Parallel Roadmap Batch Automation

## Purpose

A parallel roadmap batch is the repeatable workflow for turning the current
value-weighted dependency roadmap into bounded concurrent Codex work.

The coordinator does three things:

1. reviews what can run in parallel now;
2. asks for an explicit approval gate;
3. fans out bounded implementation workers, then integrates and documents the
   batch as one workspace update.

This is the preferred long-term shape for automatic Codex execution. It keeps
parallel work fast without letting agents skip dependency gates, ownership
boundaries, design acceptance, or closeout documentation.

## Diagrams

The automation boundaries are captured as PlantUML swimlane diagrams:

- [codex-roadmap-orchestration-target.puml](diagrams/codex-roadmap-orchestration-target.puml)
- [parallel-roadmap-batch-automation.puml](diagrams/parallel-roadmap-batch-automation.puml)
- [design-intake-roadmap-automation.puml](diagrams/design-intake-roadmap-automation.puml)

The diagrams distinguish user approval gates from Codex coordinator work,
workflow automation, worker/subagent execution, validation, and closeout docs.

## Inputs

Use these sources in order:

1. [roadmap-items.yaml](./roadmap-items.yaml)
2. [roadmap-decision-register.md](./roadmap-decision-register.md)
3. [design-implementation-triage.md](./design-implementation-triage.md)
4. [repo-execution-priority-checklist.md](./repo-execution-priority-checklist.md)
5. owning app, domain, engine, or net roadmaps;
6. current code truth and validation state.

New designs enter the same system through architecture governance review first.
Until the design has an accepted owner, dependency level, gate, value/blocker
score, and roadmap row, it remains discovery work and must not be mixed into an
implementation batch.

## Approval Gate

Before implementation, the coordinator presents a batch proposal:

```text
Batch name:
Base branch or worktree state:
Candidate rows:
Parallel lanes:
Worker prompts:
Disjoint write scopes:
Expected validations:
Docs to update after integration:
Stop conditions:
```

The batch can start only after the user approves that proposal.

Use the structured proposer when the batch should be generated from current
roadmap rows:

```text
task batch:kickoff -- --next
task batch:propose -- --goal "<goal>" --scope L0 --out docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
task batch:approve -- --batch docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
task batch:prepare -- --batch docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
task batch:scope-check -- --batch docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
```

Use `batch:kickoff -- --next` for the normal one-line Codex start. It selects
only `planning_state=current_candidate` items, writes the proposed manifest, and
prints the exact follow-up commands. Use `batch:propose` directly only when the
coordinator needs a custom goal, scope, output path, or discovery flag.

## Execution Shape

Preferred execution uses real git worktrees or otherwise isolated worker
branches when the environment supports them. If workers share one dirty
workspace, the coordinator must treat the result as a single integration branch
and perform a combined diff review before validation.

Each worker must:

- read `AGENTS.md` and `AI_GUIDE.md`;
- inspect owning docs and code truth before editing;
- own a disjoint write scope;
- avoid later roadmap phases unless explicitly approved;
- run focused validation;
- report changed files, exact functions/modules, validation, blockers, and
  deferred work.

## Integration Closeout

After workers finish, the coordinator:

1. reviews each worker output and the combined `git diff`;
2. rejects or repairs conflicts, ownership leaks, or scope expansion;
3. runs focused validation and broader workspace checks where needed;
4. updates roadmap docs, score evidence, triage status, and lifecycle links;
5. reports remaining blockers and the next recommended batch.

Coordinator-level docs updates are mandatory. Worker-local docs updates are not
enough because the workspace roadmap needs to record what the batch changed.

## New Design Intake

When the user discusses or proposes a new design:

1. run architecture governance review;
2. identify owner, bounded context, dependency direction, ADR need, and
   migration shape;
3. run `task roadmap:intake -- --idea "<idea>"` to create a review proposal;
4. place the design in the correct lifecycle folder;
5. add or update roadmap decision-register scoring only when the design has a
   concrete implementation candidate;
6. update the dependency roadmap diagram if the design changes topology;
7. keep it behind `B5` or discovery gates until an accepted design, ADR, or
   owning roadmap promotes it.

Accepted intake proposals are applied with:

```text
task roadmap:apply-intake -- --proposal docs-site/src/content/docs/reports/roadmap-intake/<id>/proposal.yaml
task roadmap:promote -- --id WR-XXX --state current_candidate --evidence "<accepted evidence>"
```

## Automation Boundary

The repo workflow can automate prompt generation, batch proposals, validation
checklists, worktree preparation, generated roadmap docs, PlantUML validation,
and closeout requirements. Codex subagent spawning is a runtime capability of
the Codex environment, not a normal Rust or docs-site feature.

Do not add autonomous AI mutation paths to `foundation` or `domain` crates. AI
workflow automation belongs in workspace docs, `tools/`, apps, or adapters.
