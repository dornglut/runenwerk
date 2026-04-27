---
title: Commit Splitting Routine
description: Bounded routine for organizing mixed Runenwerk working-tree changes into coherent commits.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../prompt-templates/commit-organization.md
---

# Commit Splitting Routine

## Purpose

Use this routine when the working tree contains mixed docs, code, crate, manifest, or tooling changes.

## Preconditions

Before staging:

1. Do not commit blindly.
2. Preserve all unrelated working-tree changes.
3. Inspect status and diffs.
4. Identify shared manifest files.
5. Identify generated files that must not be staged.

## Routine

1. Capture:
   - `git status --short`
   - `git diff --find-renames --stat`
   - `git diff --find-renames --name-status`
   - `git diff --summary`
2. Group files by architectural ownership.
3. Identify files that require patch staging.
4. Stage one coherent commit at a time.
5. Verify staged scope:
   - `git diff --cached --find-renames --stat`
   - `git diff --cached --find-renames --name-status`
6. Run validation for that commit.
7. Commit.
8. Repeat until the tree is clean or intentionally left with known work.

## Safety Rules

- `git restore --staged .` is allowed to clear the index.
- Do not use destructive git commands.
- Do not stage generated context exports.
- Do not combine unrelated domains for convenience.
- Do not hide failing validation.

## Final Report

Include:

- commit list;
- files included per commit;
- validation per commit;
- remaining working-tree state.
