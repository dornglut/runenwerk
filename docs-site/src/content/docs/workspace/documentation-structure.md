---
title: Documentation Structure
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ./start-here.md
  - ./operating-model.md
  - ./authority-model.md
  - ../guidelines/programming-principles.md
---

# Documentation Structure

This document defines how Runenwerk documentation is organized and where docs belong.

## Core model

Runenwerk uses a two-level documentation model:

```text
repository root Markdown files
  short entrypoints and summaries

docs-site/src/content/docs
  canonical long-form documentation tree
```

When a root document and a docs-site document overlap, update the docs-site document first, then align the root summary.

## Scriptless workflow requirement

Workflow documentation must be usable by reading files. Do not require a command, local checkout, generated prompt, rendered planning view, or full repository export to understand the next action.

The active workspace workflow starts at:

```text
docs-site/src/content/docs/workspace/start-here.md
```

## Root documents

The following root files are intentionally kept at repository root:

```text
README.md
AGENTS.md
ARCHITECTURE.md
CRATES.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
GLOSSARY.md
TESTING.md
```

Root documents should be concise, stable, operationally useful, and linked to canonical docs-site pages where detail exists.

Root documents must not become full design documents, detailed roadmaps, generated status views, long historical records, duplicated docs-site pages, or dumping grounds for incomplete notes.

## Canonical tree

```text
docs-site/src/content/docs/
  workspace/
  guidelines/
  software-development/
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

## Active workspace structure

```text
workspace/
  start-here.md
  operating-model.md
  authority-model.md
  ai-agent-boundaries.md
  documentation-structure.md

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
    docs-cleanup-task.md
    review-task.md

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

- `workspace/`: repository process, structure, planning, status, and maintenance docs.
- `workspace/routines/`: repeatable human/agent procedures.
- `workspace/task-cards/`: short reusable task instructions that point to routines.
- `workspace/planning/`: Markdown-first planning records.
- `guidelines/`: stable doctrine, including programming principles, architecture, dependency, module, and validation rules.
- `adr/`: durable decisions and rejected alternatives.
- `design/`: target architecture and tradeoffs.
- `reports/`: historical evidence, audits, migrations, closeouts, and benchmarks.
- `archive/`: non-authoritative historical material.

## Routine shape

Every active routine should use this structure:

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

## Naming rules

- Use kebab-case for docs-site Markdown files.
- Use `README.md` for section landing pages.
- Do not add docs-site `readme.md` files.
- Keep names boring and searchable.
- Prefer task-oriented names for workflow docs.
- Prefer authority-oriented names for doctrine docs.

## Pruning rules

Use the programming principles when pruning docs:

- KISS: keep navigation short.
- DRY: keep one authority for each durable claim.
- YAGNI: remove unused workflow surfaces.
- Separation of Concerns: separate entrypoints, routines, planning, reports, and tooling.

When moving, merging, or pruning docs, report old path to new path mapping.

## Authority conflicts

Use:

```text
docs-site/src/content/docs/workspace/authority-model.md
```

Do not resolve authority conflicts by editing generated views first or by treating local helper output as policy.
