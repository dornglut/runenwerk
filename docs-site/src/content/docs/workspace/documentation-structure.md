---
title: Documentation Structure
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ./start-here.md
  - ./operating-model.md
  - ./authority-model.md
  - ./workflow-lifecycle.md
  - ./complete-design-gate.md
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

The lifecycle model is:

```text
docs-site/src/content/docs/workspace/workflow-lifecycle.md
```

The complete design gate is:

```text
docs-site/src/content/docs/workspace/complete-design-gate.md
```

Use the complete design gate before implementation authorization for architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work.

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

## Workspace structure

`workspace/` owns repository process, lifecycle, complete design gate, structure, planning, status, and maintenance docs.

```text
workspace/
  start-here.md
  operating-model.md
  authority-model.md
  ai-agent-boundaries.md
  documentation-structure.md
  workflow-lifecycle.md
  complete-design-gate.md
  routines/
  task-cards/
  planning/
```

## Folder responsibilities

- `workspace/`: repository process, lifecycle, complete design gate, structure, planning, status, and maintenance docs.
- `workspace/routines/`: repeatable human/agent procedures.
- `workspace/task-cards/`: short reusable task instructions that point to routines.
- `workspace/planning/`: Markdown-first planning records.
- `guidelines/`: stable doctrine, including programming principles, architecture, dependency, module, and validation rules.
- `adr/`: durable decisions and rejected alternatives.
- `design/`: target architecture and tradeoffs.
- `domain/`: domain-specific current-state and target documentation.
- `reports/`: historical evidence, audits, migrations, closeouts, and benchmarks.
- `reports/closeouts/`: detailed completion evidence for completed phases or slices.
- `archive/`: non-authoritative historical material.

## Document-type rules

Use [`workflow-lifecycle.md`](workflow-lifecycle.md) for state transitions and promotion rules.

```text
Guideline
  stable doctrine and engineering rules

Design
  target architecture, vocabulary, owner boundaries, tradeoffs, non-owned responsibilities

Complete design gate
  mandatory readiness checklist and matrix templates before implementation authorization

Roadmap / production track
  strategic sequence and current planning state

Active work
  one current focus

Completed work
  short completion index

Closeout report
  detailed historical evidence

Generated file
  mirror, evidence, or contract only
```

## Routine shape

Every active routine should use this structure:

```text
Use when
Authority files to read
Working files to inspect
What to decide before editing
State transitions produced
Patch rules
Manual validation checklist
Stop conditions
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
- Separation of Concerns: separate entrypoints, lifecycle, complete design gates, routines, planning, reports, and tooling.

When moving, merging, or pruning docs, report old path to new path mapping.

## Authority conflicts

Use:

```text
docs-site/src/content/docs/workspace/authority-model.md
```

Do not resolve authority conflicts by editing generated views first or by treating local helper output as policy.
