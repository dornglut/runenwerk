---
title: Parallel Roadmap Batch Routine
description: Repeatable routine for approved parallel roadmap batch execution and closeout.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related_docs:
  - ../parallel-roadmap-batch-automation.md
  - ../prompt-templates/parallel-roadmap-batch.md
  - ../roadmap-items.yaml
  - ../roadmap-decision-register.md
  - ../design-implementation-triage.md
  - ../repo-execution-priority-checklist.md
---

# Parallel Roadmap Batch Routine

## Purpose

Use this routine to turn the value-weighted dependency roadmap into an approved
parallel implementation batch.

## Preconditions

Before execution:

1. The coordinator has inspected current roadmap docs and code truth.
2. Candidate work is in the same dependency level or otherwise independent.
3. Every candidate has an owning roadmap or accepted design path.
4. Write scopes are disjoint or intentionally sequenced.
5. The user approved the proposed batch.

## Routine

1. Capture current state:
   - `git status --short`
   - `git branch --show-current`
   - relevant `git diff --stat`
2. Read the workspace sources of truth:
   - `roadmap-items.yaml`
   - `roadmap-decision-register.md`
   - `design-implementation-triage.md`
   - `repo-execution-priority-checklist.md`
   - owning roadmaps for every candidate row
3. Run `task roadmap:validate`.
4. Select candidate rows by gate, dependency level, lane, and score.
   Use `task batch:propose -- --goal "<goal>" --scope L0
   --out docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml`
   when the batch should be recorded.
5. For each candidate, define:
   - task;
   - owner;
   - write scope;
   - stop conditions;
   - validation;
   - docs to update after completion.
6. Present the batch proposal and wait for user approval.
7. Start workers only after approval.
8. Prefer isolated git worktrees or worker branches. If workers share one dirty
   workspace, use one integration branch and review the combined diff.
9. Integrate worker outputs by ownership area.
10. Run focused validation, then broader validation when contracts cross domains.
11. Update `roadmap-items.yaml` first, then run `task roadmap:render`.
12. Run `task roadmap:check`, `task puml:validate`, and `task docs:validate`.
13. Update owning roadmap docs, lifecycle links, and closeout notes.
14. Report completed slices and the next recommended batch.

## Required Closeout Docs

Every completed batch should update at least one coordinator-level source of
truth:

- `repo-execution-priority-checklist.md` for operational status;
- `roadmap-items.yaml` for evidence, score, gate, and current decision;
- generated `roadmap-decision-register.md` for evidence and current decision;
- generated `design-implementation-triage.md` for implement-now/ready-next movement;
- owning roadmap docs when a phase checkbox or status changes.

## Stop Conditions

Stop and report when:

- approval is missing;
- a worker would need overlapping write ownership;
- a candidate requires an unaccepted design or ADR;
- current code truth contradicts the roadmap enough to invalidate the batch;
- validation fails for a reason that changes the batch decision;
- worker outputs cannot be integrated without reverting unrelated work.
