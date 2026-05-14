---
title: Architecture Governance Review Prompt
description: Prompt template for pre-implementation architecture governance reviews.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related_docs:
  - ../agents.md
  - ../architecture-governance-review.md
  - ../planning-and-implementation-workflow.md
  - ../routines/architecture-governance-review-routine.md
  - ../../guidelines/domain-map.md
  - ../../guidelines/module-structure-guidelines.md
  - ../../adr/README.md
---

# Architecture Governance Review Prompt

Use this template before implementation when a task may affect dependency
direction, domain ownership, an ADR-worthy decision, migration strategy,
tradeoffs, enforcement, or ownership mode.

This template is a decision gate. It does not authorize edits by itself.

## Template

```text
Run an architecture governance review for this Runenwerk change.

Task:
- <task>

Scope:
- <crate/files/subsystem/design/roadmap>

Before recommendations:
1. Read AGENTS.md and AI_GUIDE.md.
2. Read ARCHITECTURE.md, DEPENDENCY_RULES.md, DOMAIN_MAP.md, GLOSSARY.md, and TESTING.md.
3. Read docs-site/src/content/docs/workspace/architecture-governance-review.md.
4. Read docs-site/src/content/docs/workspace/planning-methods.md.
5. Read the owning roadmap, design, ADR, or domain docs for the scope.
6. Inspect current code, tests, docs, and git state before judging.
7. Do not edit files unless explicitly asked after the review.

Output:
1. Governance recommendation: implement, prototype, write/update ADR, update design, defer, or reject.
2. DDD bounded context owner: owning domain, crate, subsystem, vocabulary, invariants, and translation boundaries.
3. Clean Architecture dependency check: allowed direction, forbidden dependencies, and required boundary contracts.
4. ADR need: whether the decision is durable enough to record, with the proposed ADR subject.
5. ATAM-lite summary: quality attributes in tension, options, sensitivity points, risks, non-risks, and evidence needed.
6. Strangler Fig migration shape: old path, coexistence boundary, first routed slice, parity guard, switch point, and deletion guard.
7. Fitness functions: tests, docs validation, metadata checks, architecture guards, or CI gates needed to enforce the boundary.
8. Team Topologies ownership label: stream-aligned, platform, complicated subsystem, or enabling.
9. Validation commands and stop conditions.

Stop instead of recommending implementation when:
- the owner or dependency direction is unclear;
- implementation would require a forbidden dependency;
- a durable architecture decision has no accepted ADR or design path;
- the migration cannot safely coexist with the old path;
- there is not enough evidence to promote the item beyond discovery.
```

## Expected Agent Behavior

The agent should inspect repository truth first, then make a narrow governance
recommendation. Prefer cleanup and explicit boundary contracts before redesign.

Use [Architecture Audit](./architecture-audit.md) when the task is findings-only.
Use this prompt when the task needs a pre-implementation decision gate.
