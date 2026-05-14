---
title: Architecture Governance Review Routine
description: Bounded routine for architecture-sensitive Runenwerk changes.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related_docs:
  - ../agents.md
  - ../architecture-governance-review.md
  - ../planning-and-implementation-workflow.md
  - ../prompt-templates/architecture-governance-review.md
  - ../../guidelines/domain-map.md
  - ../../guidelines/module-structure-guidelines.md
  - ../../adr/README.md
---

# Architecture Governance Review Routine

## Purpose

Use this routine before implementation when a change may affect architecture
ownership, dependency direction, durable decision history, migration shape,
tradeoff policy, enforcement, or ownership mode.

This is a review and decision routine. It should produce an implementation,
prototype, design, ADR, defer, or reject recommendation.

## Preconditions

Before reviewing:

1. Identify the requested task and scope.
2. Capture the current dirty worktree state.
3. Identify the likely owning domain, crate, and subsystem.
4. Find the owning roadmap, design, ADR, or domain doc.
5. Decide whether the task is architecture-sensitive enough to need this routine.

## Routine

1. Capture current state:
   - `git status --short`
   - relevant `git diff -- <path>`
2. Read the root AI and architecture entrypoints:
   - `AGENTS.md`
   - `AI_GUIDE.md`
   - `ARCHITECTURE.md`
   - `DEPENDENCY_RULES.md`
   - `DOMAIN_MAP.md`
   - `GLOSSARY.md`
   - `TESTING.md`
3. Inspect the owning docs and code before recommending a change.
4. Use DDD to name the bounded context owner, vocabulary, invariants, and translation boundaries.
5. Use Clean Architecture to verify that dependencies point toward stable policy and domain contracts.
6. Decide whether an ADR is required for a durable architecture choice.
7. Run ATAM-lite when quality attributes are in tension.
8. Use Strangler Fig only when an old path must coexist with a replacement path.
9. Name required fitness functions before treating a documented boundary as enforceable.
10. Assign a Team Topologies ownership label for collaboration expectations.
11. Recommend one next action: implement, prototype, write/update ADR, update design, defer, or reject.

## Findings Format

Use this format:

```text
Recommendation:
Scope:
Owner:
Dependency direction:
ADR need:
ATAM-lite:
Migration shape:
Fitness functions:
Ownership mode:
Validation:
Stop conditions:
Next action:
```

## Required Validation

For a review-only pass, validation is normally inspection-only. If documentation
changes are made after the review, run:

```text
task docs:validate
```

If code changes follow, run the focused tests or architecture guards named by
the review.

## Stop Conditions

Stop and report when:

- ownership is unclear;
- a dependency direction violation would be required;
- a durable architecture decision lacks an ADR or accepted design path;
- migration cannot safely route old and new paths side by side;
- the evidence is too weak to promote the work beyond discovery;
- validation needed for the recommendation is unavailable.

## Final Report

Include:

- the exact files and modules inspected;
- the governance recommendation and why;
- the owning domain, crate, subsystem, and boundary contracts;
- any required ADR, design update, migration guard, or fitness function;
- validation commands to run before implementation or closeout.
