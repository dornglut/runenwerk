---
title: Documentation Structure
description: Canonical placement and authority rules for Runenwerk documentation.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-09-25
publication: repository-current
draft: true
pagefind: false
sidebar:
  hidden: true
related_docs:
  - ./start-here.md
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

## Publication classes

Every documentation source has exactly one `publication` class. This field is the
semantic publication authority. Directory placement, lifecycle names such as
`accepted` or `active`, and words such as `reference` in a path do not determine
the class by themselves.

Publication controls production exposure. For routed classes, the current Starlight
configuration groups pages in the public sidebar by source directory; the publication
class does not independently choose a sidebar section. The production-build validator
still requires every routed publication page to appear in the generated sidebar.

| Class | Use it for | Production projection |
|---|---|---|
| `primary` | Maintained product/developer documentation intended for normal public discovery. | Production route, sitemap, default Pagefind search, and generated public sidebar. |
| `reference` | Maintained deeper reference, design, architecture, guideline, or evidence material that should remain publicly routable without entering default search. | Production route, sitemap, generated public sidebar, and `pagefind: false`. |
| `repository-current` | Maintained repository/contributor/process material that is current authority for repository work but is not public site content. | Source retained only; no production route, Pagefind, sitemap, or public sidebar entry. Requires `draft: true`, `pagefind: false`, and `sidebar.hidden: true`. |
| `history` | Retained historical evidence or superseded context that remains useful for provenance but does not authorize current work. | Source retained only; no production route, Pagefind, sitemap, or public sidebar entry. Requires `draft: true`, `pagefind: false`, and `sidebar.hidden: true`. |

The current sidebar configuration presents owner/developer directories under
**Primary documentation** and architecture, ADR, design, guideline, and report
directories under **Reference**. That grouping is navigation presentation, not a
second publication authority.

`draft`, `pagefind`, and `sidebar.hidden` are projection controls derived from the
publication role. They must not contradict `publication`. In particular:

- a `primary` page must remain routed, searchable, and present in the generated sidebar;
- a `reference` page must remain routed and present in the generated sidebar, with
  default Pagefind disabled;
- `repository-current` and `history` pages must carry all three suppression controls
  shown above.

Typical frontmatter shapes are:

```yaml
# Normal maintained owner/developer page.
publication: primary
```

```yaml
# Accepted design or other deeper public reference.
publication: reference
pagefind: false
```

```yaml
# Current repository-only workspace/process authority.
publication: repository-current
draft: true
pagefind: false
sidebar:
  hidden: true
```

```yaml
# Retained historical evidence.
publication: history
draft: true
pagefind: false
sidebar:
  hidden: true
```

For a new document, choose the class explicitly from its semantic role. For a moved
or re-homed document, preserve the existing class when its role is unchanged and
re-evaluate it only when the owning work explicitly changes that role. Never allow a
move to drop publication metadata or infer a new class merely from the destination
path.

## Relation metadata

Relations on maintained documents describe current repository navigation and must
resolve. Historical report and archive relations preserve point-in-time provenance
and may retain truthful paths that no longer resolve. Report `README.md` and other
maintained index pages remain current navigation surfaces. `superseded_by` and
`replaced_by` always point to a current successor, including when their source is
historical. Nested closeout evidence is provenance rather than live navigation; do
not add compatibility stubs solely to preserve stale historical paths.

## Root entrypoints

The root keeps:

```text
README.md
AGENTS.md
ARCHITECTURE.md
TESTING.md
```

These are the primary public and contributor entrypoints. Detailed dependency guidance
lives directly in the canonical docs-site owner rather than through a root forwarding
summary.

Crate inventory, code-placement guidance, dependency rules, and shared vocabulary live
directly in their canonical docs-site owners:

- [`crate-inventory.md`](./crate-inventory.md) for active local workspace membership;
- [`../guidelines/architecture.md`](../guidelines/architecture.md) for Runenwerk placement and boundary guidance;
- [`../guidelines/dependency-rules.md`](../guidelines/dependency-rules.md) for Runenwerk-local dependency direction, adapter boundaries, and framework-consumer rules;
- [`glossary.md`](./glossary.md) for shared vocabulary.

Root documents must not become roadmaps, design dossiers, execution ledgers, or duplicated reference manuals.

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
- **Investigation report:** source-grounded point-in-time reality and unresolved findings.
- **Roadmap:** durable high-level sequence and dependencies; never an execution or live-state ledger.
- **GitHub issue / Engineering Portfolio:** proposed, active, blocked, deferred, completed, and prioritized work state.
- **Closeout report:** historical completion evidence when a PR and issue are not enough.
- **Archive:** superseded or historical context that does not authorize new work.

## Placement rules

1. Put one durable decision in one ADR or accepted design.
2. Put one active or deferred task in one GitHub issue and use the Engineering Portfolio for live priority/status.
3. Put durable high-level sequence and dependencies in the maintained roadmap.
4. Put current behavior in code and tests.
5. Put delivery evidence in the pull request.
6. Cross-link instead of copying full state.
7. Move obsolete active documentation to reports or archive; do not preserve it as a parallel workflow.

## Naming

- Use kebab-case for docs-site Markdown files.
- Use `README.md` for section landing pages.
- Keep names literal and searchable.
- Prefer ownership-oriented names for architecture and task-oriented names for procedures.

When pruning or moving documents, update current inbound links. Historical reports may preserve point-in-time provenance, but they do not authorize current work.