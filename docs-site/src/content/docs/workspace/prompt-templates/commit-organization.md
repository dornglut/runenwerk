---
title: Commit Organization Prompt
description: Prompt template for splitting Runenwerk working-tree changes into coherent commits.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../routines/commit-splitting-routine.md
---

# Commit Organization Prompt

Use this template when the working tree contains mixed changes.

## Template

```text
Organize the current working tree into clean commits.

Before recommending commands:
1. Inspect git status, diff stat, name-status, and relevant manifest diffs.
2. Identify unrelated changes.
3. Group changes by domain and architectural ownership.
4. Do not stage or commit everything together unless the tree is truly one coherent change.
5. Protect unrelated work from being reverted or lost.

Output:
1. Proposed commit order.
2. Files included in each commit.
3. Files explicitly excluded from each commit.
4. Exact git add commands.
5. Validation commands before each commit.
6. Commit messages.
7. Final post-commit status check.

Never use destructive git commands.
```

## Minimum Evidence

The agent should inspect:

```text
git status --short
git diff --find-renames --stat
git diff --find-renames --name-status
git diff --summary
git diff --cached --find-renames --name-status
```
