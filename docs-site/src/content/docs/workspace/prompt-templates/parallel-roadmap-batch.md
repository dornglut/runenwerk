---
title: Parallel Roadmap Batch Prompt
description: Prompt template for coordinating approved parallel roadmap implementation batches.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related_docs:
  - ../parallel-roadmap-batch-automation.md
  - ../roadmap-decision-register.md
  - ../design-implementation-triage.md
  - ../repo-execution-priority-checklist.md
  - ../routines/parallel-roadmap-batch-routine.md
---

# Parallel Roadmap Batch Prompt

Use this template when asking Codex to coordinate a parallel roadmap batch.

The coordinator must first propose the batch and wait for explicit user
approval. After approval, it may spawn or coordinate workers when the current
environment provides subagents or worktrees.

Generated coordinator and worker prompts include the canonical quality doctrine
marker `runenwerk-quality-doctrine-v1`; keep that marker in worker prompts so
the quality bar stays centralized.

## Template

```text
Coordinate a Runenwerk parallel roadmap batch.

Batch goal:
- <goal or current roadmap focus>

Scope:
- <workspace rows, domains, roadmaps, or design proposals>

Phase 1: propose, do not implement.
1. Read AGENTS.md, AI_GUIDE.md, and the root architecture docs.
2. Read workspace/parallel-roadmap-batch-automation.md.
3. Read workspace/roadmap-items.yaml as the active execution source of truth.
4. Run `task roadmap:validate`.
5. Read workspace/roadmap-decision-register.md.
6. Read workspace/design-implementation-triage.md.
7. Read workspace/repo-execution-priority-checklist.md.
8. Read workspace/roadmap-archive.yaml and workspace/roadmap-deferred.yaml only for dependency, evidence, and policy context.
9. Inspect current git state and relevant owning roadmaps.
10. Identify work that is parallelizable at the same dependency level.
11. Exclude blocked B5 work and designs without accepted owners.
12. Produce a batch proposal with worker prompts, write scopes, validations, stop conditions, and coordinator-level docs updates.
13. Stop for explicit user approval.

Phase 2: execute after approval.
1. Create or name an integration branch.
2. Prefer isolated git worktrees or worker branches when available.
3. Start one worker per disjoint roadmap slice.
4. Require every worker to inspect, implement, validate, and report exact files/functions/modules.
5. Review the combined diff before accepting the batch.
6. Run focused and broad validation.
7. Update the owning roadmap YAML source first, then run `task roadmap:render`.
8. Run `task roadmap:check` and `task puml:validate`.
9. Update owning roadmaps, lifecycle links, and closeout evidence.
10. Report completed slices, blockers, and the next recommended batch.

Stop instead of executing when:
- the user has not approved the batch proposal;
- write scopes overlap in a way that would create merge risk;
- a candidate requires an unaccepted ADR/design;
- a candidate crosses dependency levels;
- workers cannot be isolated and the dirty worktree is too risky to integrate.
```

## Expected Agent Behavior

The coordinator owns sequencing and integration. Workers own implementation.

If subagents are not available, output worker prompts and stop. If subagents are
available, use them only after the approval gate.
