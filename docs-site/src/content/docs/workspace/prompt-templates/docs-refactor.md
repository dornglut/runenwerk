---
title: Documentation Refactor Prompt
description: Prompt template for documentation restructuring work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../documentation-structure.md
  - ../routines/docs-refactor-routine.md
---

# Documentation Refactor Prompt

Use this template when moving, renaming, pruning, or reorganizing documentation.

## Template

```text
Refactor the Runenwerk documentation for this scope:

Scope:
- <docs area/files>

Requirements:
1. Inspect current docs structure and links first.
2. Follow docs-site frontmatter, status, lifecycle, and filename conventions.
3. Do not treat outdated docs as truth when code contradicts docs.
4. Do not rename or move files only for aesthetics.
5. Preserve source-of-truth clarity.
6. Update indexes and internal links.
7. Run documentation validation.

Output:
1. Current structure findings.
2. Proposed moves/renames/prunes.
3. Exact files to change.
4. Link updates needed.
5. Validation commands.
6. Suggested commit split.
```
