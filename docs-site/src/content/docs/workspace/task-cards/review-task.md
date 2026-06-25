---
title: Review Task
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
---

# Review Task

Use this card for architecture review, pull request review, phase closeout, or docs review.

Repository: `Crystonix/Runenwerk`

Routine:

```text
Choose the matching routine from docs-site/src/content/docs/workspace/start-here.md.
```

Authority files:

```text
AGENTS.md
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
TESTING.md
docs-site/src/content/docs/guidelines/programming-principles.md
owning design, ADR, routine, roadmap, or docs file
```

Review lens:

- KISS: is the path understandable without ceremony?
- DRY: is there one owner for each durable claim?
- YAGNI: is every new surface needed now?
- SOLID: are responsibilities and dependencies legal?
- Separation of Concerns: are code, docs, planning, reports, and tooling separated by purpose?
- Avoid Premature Optimization: is complexity evidence-driven?
- Law of Demeter: does the change use direct contracts instead of internals?

Final report:

```text
Recommendation:
Files inspected:
Findings:
Validation evidence:
Risks:
Next action:
```
