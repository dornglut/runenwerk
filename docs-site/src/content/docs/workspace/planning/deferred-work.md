---
title: Deferred Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-04
related_docs:
  - ../engineering-workflow.md
  - ./README.md
---

# Deferred Work

Use this file for intentionally postponed planning work.

GitHub issues own live deferred state. This page is a concise cross-project index and must not become a second task database.

## Entry shape

```text
ID:
Title:
State: deferred
Reason deferred:
Reactivation condition:
Owner:
Blocked by:
Owning issue:
Notes:
```

## Rules

- Deferred does not mean completed.
- Deferred work must name a reason and a concrete reactivation condition.
- Update the owning issue before changing this summary.
- Reactivation requires an explicit planning decision and moves the owning issue back to `proposed`, `active`, or `blocked` as appropriate.
- Do not preserve optional mirrors of live state.
- Follow [Engineering Workflow](../engineering-workflow.md) before implementation resumes.
