---
title: Code Review Prompt
description: Prompt template for focused Runenwerk code review.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
  - ../routines/public-api-review-routine.md
---

# Code Review Prompt

Use this template for focused code review.

## Template

```text
Review the current changes for:

Scope:
- <files/crates/subsystems>

Review requirements:
1. Inspect the actual changed files.
2. Prioritize correctness, regressions, ownership boundaries, API friction, and missing tests.
3. Do not rewrite unless explicitly asked.
4. Do not guess about intent; infer only from code, docs, and tests.
5. Mention if no issues are found.

Output:
1. Findings first, ordered by severity.
2. Exact file path and function/module location for each finding.
3. Why it matters.
4. Minimal recommended fix.
5. Tests or validation missing.
6. Safe commit split if relevant.
```

## Severity Guide

- Critical: broken invariant, data loss, unsoundness, wrong dependency direction.
- High: likely runtime bug, broken public API, missing validation on authoritative state.
- Medium: unclear ownership, duplicated logic, brittle design, insufficient tests.
- Low: naming, discoverability, docs drift, minor ergonomics.
