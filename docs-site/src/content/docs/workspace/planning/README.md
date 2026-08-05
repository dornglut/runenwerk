---
title: Planning Records
description: Concise Markdown planning summaries for Runenwerk.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-04
related_docs:
  - ../engineering-workflow.md
  - ../authority-model.md
  - ../documentation-structure.md
  - ./roadmap.md
---

# Planning Records

Runenwerk keeps planning readable without creating a second work-management system.

## Authority split

```text
GitHub issue
  live task state, scope, owner, blockers, and acceptance criteria

accepted ADR or design
  durable architecture, ownership, dependency direction, and migration

roadmap
  high-level sequence and dependencies

pull request
  proposed diff, review findings, validation, and merge evidence

code and tests
  current behavior

reports and archive
  historical evidence and context
```

Do not copy live issue or pull-request state into stable architecture documents. Do not use planning files as execution ledgers, branch trackers, CI dashboards, generated task databases, or implementation authorization certificates.

## Maintained planning files

- [Active Work](active-work.md) — concise cross-project summary of current focus and blockers.
- [Roadmap](roadmap.md) — durable high-level sequence and dependencies.
- [Deferred Work](deferred-work.md) — work intentionally postponed, with reason and reactivation condition.
- [Completed Work](completed-work.md) — short historical index linking accepted evidence.
- [Decision Register](decision-register.md) — durable planning decisions that materially change priority, sequence, ownership, or disposition.

GitHub issues remain authoritative when a summary disagrees with live work state. Correct the summary rather than creating another planning record.

## Planning rules

1. Give each live task one owning GitHub issue.
2. Keep the roadmap at milestone and dependency level; do not enumerate branch execution state.
3. Put architecture and public-contract decisions in an accepted ADR or design before implementation when the engineering workflow classifies the change as architectural or extraction work.
4. Record delivery evidence in the pull request, not in the roadmap.
5. Keep completed-work entries short; use a closeout report only when durable historical evidence would otherwise bloat the index.
6. Name why deferred work is deferred and what would reactivate it.
7. Cross-link owners instead of duplicating their content.
8. Remove obsolete planning authority after current consumers are migrated; do not preserve compatibility pages without a real consumer and removal condition.

## Work states

Use the operational states defined by [Engineering Workflow](../engineering-workflow.md):

```text
proposed
active
blocked
done
deferred
```

Durable documents may additionally be `accepted`, `superseded`, or `archived` when those words describe document authority rather than task execution.

Do not invent intermediate process states merely to record review ceremony. A pull request and its checks already own delivery and merge state.

## Updating planning truth

When work changes:

- update the owning issue first;
- update the roadmap only when high-level sequence or dependency truth changed;
- update active work only when the concise current-focus summary changed;
- update deferred or completed indexes only when disposition changed;
- add a decision-register entry only for a durable planning decision;
- keep exact branch heads, workflow run IDs, and transient diagnostics in the pull request.

A phase specification may provide a bounded implementation handoff when an accepted design calls for one. It derives from the owning issue and accepted architecture; it does not replace them or create a parallel lifecycle.

## Review and validation

Planning changes are reviewed against the owning issue, accepted architecture, current code where behavior claims are made, and the repository authority model.

Run the documentation checks required by [Engineering Workflow](../engineering-workflow.md) and [TESTING.md](../../../../../../TESTING.md). Report inspection as inspection and executable validation as validation.
