---
title: Documentation Structure
description: Canonical placement and authority rules for Runenwerk documentation.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-22
related_docs:
  - ./start-here.md
  - ./engineering-workflow.md
  - ./authority-model.md
---

# Documentation Structure

Runenwerk uses two documentation levels:

```text
repository root
  concise public and contributor entrypoints

docs-site/src/content/docs
  canonical long-form architecture, design, planning, and history
```

When they overlap, the docs-site document owns the detail. Root files summarize and link; they do not duplicate full policy or design.

## Root entrypoints

The root keeps:

```text
README.md
AGENTS.md
ARCHITECTURE.md
TESTING.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
CRATES.md
GLOSSARY.md
```

`README.md`, `AGENTS.md`, `ARCHITECTURE.md`, and `TESTING.md` are concise operational summaries. The other root documents are compatibility entrypoints to canonical docs-site owners. Root documents must not become roadmaps, design dossiers, execution ledgers, or duplicated reference manuals.

## Canonical tree

```text
docs-site/src/content/docs/
  workspace/     process, planning, repository inventories, glossary
  guidelines/    stable engineering and dependency rules
  architecture/  current and target system structure
  adr/           durable decisions and rejected alternatives
  design/        target contracts and migration plans
  foundation/    foundation-specific documentation
  domain/        domain-specific documentation
  apps/          application documentation
  net/           networking documentation
  adapters/      integration and host adapters
  reports/       investigations, proofs, closeouts, benchmarks
  archive/       non-authoritative historical material
```

## Document responsibilities

- **Guideline:** stable engineering rule or doctrine.
- **ADR:** durable decision, alternatives, and consequences.
- **Architecture:** repository or subsystem ownership and dependency structure.
- **Design:** target behavior, public vocabulary, boundaries, and migration.
- **Investigation report:** source-grounded current reality and unresolved findings.
- **Roadmap:** high-level sequence and dependencies; never an execution ledger.
- **Active work:** concise cross-project summary; GitHub issues remain authoritative.
- **Closeout report:** historical completion evidence when a PR and issue are not enough.
- **Archive:** superseded or historical context that does not authorize new work.

## Placement rules

1. Put one durable decision in one ADR or accepted design.
2. Put one active task in one GitHub issue.
3. Put high-level sequence in the maintained roadmap.
4. Put current behavior in code and tests.
5. Put delivery evidence in the pull request.
6. Cross-link instead of copying full state.
7. Move obsolete active documentation to reports or archive; do not preserve it as a parallel workflow.

## Naming

- Use kebab-case for docs-site Markdown files.
- Use `README.md` for section landing pages.
- Keep names literal and searchable.
- Prefer ownership-oriented names for architecture and task-oriented names for procedures.

When pruning or moving documents, update inbound links and record any compatibility entrypoint that remains.
