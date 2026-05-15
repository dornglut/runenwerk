---
title: New Design Intake
description: One-line Codex prompt for turning a new design or change idea into a roadmap review proposal.
status: active
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-15
---

# New Design Intake

Use this when the user wants to discuss a new design, feature, architecture
change, or implementation track before it is eligible for batch work.

## One-Line Prompt

```text
Run task roadmap:intake -- --idea "<design/change idea>" and prepare it for roadmap review.
```

## Workflow

1. Inspect the relevant docs and code ownership before changing roadmap state.
2. Run architecture governance review for dependency direction, DDD owner, ADR
   need, migration shape, tradeoffs, and fitness functions.
3. Generate an intake proposal with `task roadmap:intake`.
4. Discuss and edit the proposal until the user approves it.
5. Apply with `task roadmap:apply-intake -- --proposal <proposal.yaml>`.
6. Promote only with evidence:

```text
task roadmap:promote -- --id WR-XXX --state current_candidate --evidence "<accepted evidence>"
```

## Stop Conditions

- Ownership or dependency direction is unclear.
- No accepted design, ADR, closeout, or roadmap evidence exists.
- The item would need `current_candidate` before blocker and dependency gates are clear.
- The proposal needs write scopes or validations that cannot be named yet.
