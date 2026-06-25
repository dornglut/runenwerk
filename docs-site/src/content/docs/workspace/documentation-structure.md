---
title: Documentation Structure
description: Source-of-truth rules, document lifecycles, placement policy, naming rules, and maintenance expectations for Runenwerk documentation.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ./start-here.md
  - ./operating-model.md
  - ./authority-model.md
---

# Documentation Structure

This document defines how Runenwerk documentation is organized and how to place new or changed docs.

## Core model

Runenwerk uses a two-level documentation model:

```text
repository root Markdown files
  short operational summaries for humans and AI agents working from the root

docs-site/src/content/docs
  canonical long-form documentation tree
```

Root Markdown files are entrypoints and summaries. The docs-site tree is the canonical location for detailed documentation.

When a root document and a docs-site document overlap, update the docs-site document first, then align the root summary.

## Scriptless workflow requirement

Workflow documentation must be usable by reading files. Do not require a command, local checkout, generated prompt, rendered planning view, or full repository export to understand the next action.

Local helpers may be documented in an `Optional local helpers` section only.

The active workspace workflow starts at:

```text
docs-site/src/content/docs/workspace/start-here.md
```

## Root documents

The following root files are intentionally kept at repository root:

```text
README.md
AGENTS.md
AI_GUIDE.md
ARCHITECTURE.md
CRATES.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
GLOSSARY.md
TESTING.md
```

Root documents should be:

- concise;
- stable;
- operationally useful;
- safe to read before making changes;
- linked to canonical docs-site pages where detail exists.

Root documents must not become:

- full design documents;
- detailed implementation roadmaps;
- generated status views;
- long historical records;
- duplicated copies of docs-site pages;
- dumping grounds for incomplete notes.

## Canonical tree

Recommended high-level structure:

```text
docs-site/src/content/docs/
  workspace/
  software-development/
  guidelines/
  foundation/
  adr/
  design/
  domain/
  apps/
  net/
  adapters/
  reports/
  archive/
```

## Workspace structure

The active workspace workflow structure is:

```text
workspace/
  start-here.md
  operating-model.md
  authority-model.md
  documentation-structure.md
  planning-and-implementation-workflow.md

  routines/
    README.md
    investigation-routine.md
    implementation-routine.md
    architecture-governance-review-routine.md
    code-refactor-routine.md
    docs-refactor-routine.md
    roadmap-update-routine.md
    phase-completion-drift-check-routine.md
    pr-review-routine.md

  task-cards/
    README.md
    github-connector-task.md
    codex-task.md
    implementation-task.md
    architecture-review-task.md
    docs-cleanup-task.md
    roadmap-update-task.md
    phase-closeout-task.md

  planning/
    README.md
    active-work.md
    roadmap.md
    deferred-work.md
    completed-work.md
    production-tracks.md
    decision-register.md
```

## Folder responsibilities

### `workspace/`

Repository-wide process, structure, planning, status, and maintenance documentation.

Workspace docs must not contain crate-specific implementation detail unless the document is an index or status tracker.

### `workspace/routines/`

Repeatable human/agent procedures. Routines own process.

Every active routine should use this shape:

```text
Use when
Authority files to read
Working files to inspect
What to decide before editing
Patch rules
Manual validation checklist
Evidence to report
Optional local helpers
```

### `workspace/task-cards/`

Copy-pasteable task prompts. Task cards wrap routines. They do not own process.

### `workspace/planning/`

Markdown-first planning records. These files own active planning state from the scriptless workflow cutover onward.

Legacy YAML and generated Markdown files may remain as optional mirrors or migration context, but they are not required for normal workflow comprehension.

### `guidelines/`

Stable repository doctrine such as dependency direction, architecture principles, module structure, validation policy, and contribution rules.

### `adr/`

Durable architecture decisions and rejected alternatives.

### `design/`

Active, accepted, implemented, deferred, superseded, or rejected design documents.

### `reports/`

Historical audits, closeouts, migrations, benchmarks, and evidence records. Reports may prove what happened; they do not authorize future work by themselves.

### `archive/`

Historical non-authoritative material that should not remain in active workflow navigation.

## Naming rules

- Use kebab-case for docs-site Markdown files.
- Use `README.md` for section landing pages.
- Do not add docs-site `readme.md` files.
- Keep names boring and searchable.
- Prefer task-oriented names for workflow docs.
- Prefer authority-oriented names for doctrine docs.

## Moving or pruning docs

When moving, merging, or pruning docs:

1. Classify each affected file by responsibility.
2. Identify the owning canonical file.
3. Move useful material into the owner.
4. Replace stale active docs with a short redirect or archive note only when needed.
5. Update internal links.
6. Update root summaries last.
7. Report old path to new path mapping.

## Authority conflicts

Use:

```text
docs-site/src/content/docs/workspace/authority-model.md
```

Do not resolve authority conflicts by editing generated views first or by treating local helper output as policy.
