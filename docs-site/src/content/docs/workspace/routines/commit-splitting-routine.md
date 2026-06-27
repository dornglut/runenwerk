---
title: Commit Splitting Routine
description: Optional local-checkout routine for grouping mixed working-tree changes.
status: active
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-06-27
related_docs:
  - ../workflow-lifecycle.md
---

# Commit Splitting Routine

This is an optional local-checkout routine.

Use it only when a working tree contains mixed changes and local git commands are available.

For connector or context-tool work, do not use this as an active workflow. Instead report file groups and recommend commit boundaries.

Safety rules:

- Preserve unrelated work.
- Group files by domain and responsibility.
- Do not combine unrelated domains for convenience.
- Do not hide failed validation.

Final report: proposed commit groups, files per group, validation status, and remaining worktree risk.
