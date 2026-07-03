# AGENTS.md

This is the single root entrypoint for AI coding agents working in Runenwerk.

Use it when working through a local checkout, GitHub connector, ChatGPT context tooling, Codex-style patching, or manual file browsing.

## Start here

For any non-trivial task, read:

```text
docs-site/src/content/docs/workspace/start-here.md
```

Then follow the matching routine under:

```text
docs-site/src/content/docs/workspace/routines/
```

For reusable task handoffs, use:

```text
docs-site/src/content/docs/workspace/task-cards/
```

For architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work, use the complete investigation gate and complete design gate before implementation:

```text
docs-site/src/content/docs/workspace/complete-investigation-gate.md
docs-site/src/content/docs/workspace/complete-design-gate.md
```

## Operating rule

Runenwerk workflows are repository-readable first.

Do not assume a full repo export, a clean worktree, command execution, generated files, rendered planning views, or local scripts. Commands may provide optional validation evidence, but they are not workflow authority.

Default workflow:

```text
Read authority files -> inspect working files -> verify complete investigation gate when required -> verify complete design gate when required -> patch exact files -> manually validate evidence -> report changed files and gaps
```

## Authority files

Before code changes, read the relevant root summaries:

```text
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
CRATES.md
TESTING.md
GLOSSARY.md
```

For detailed workflow authority, use:

```text
docs-site/src/content/docs/workspace/operating-model.md
docs-site/src/content/docs/workspace/authority-model.md
docs-site/src/content/docs/workspace/documentation-structure.md
docs-site/src/content/docs/workspace/workflow-lifecycle.md
docs-site/src/content/docs/workspace/complete-investigation-gate.md
docs-site/src/content/docs/workspace/complete-design-gate.md
```

For engineering principles, use:

```text
docs-site/src/content/docs/guidelines/programming-principles.md
```

## Seven programming principles

Use these principles as a review lens:

1. KISS: keep the owned path simple.
2. DRY: remove duplicate authority.
3. YAGNI: do not build speculative surfaces.
4. SOLID: keep responsibility and dependency boundaries honest.
5. Separation of Concerns: organize by responsibility.
6. Avoid Premature Optimization: prove the bottleneck first.
7. Law of Demeter: depend on direct contracts.

These principles do not override accepted ADRs, dependency rules, domain ownership, complete investigation gates, complete design gates, or tests. They help identify when a proposed change is overbuilt, duplicated, misplaced, or too coupled.

## Repository conventions

- Code is organized by domain, not by technical layer.
- Each domain owns its models, services, rules, and vocabulary.
- Changes must respect domain boundaries and dependency direction.
- Prefer long-term, architecture-correct solutions over local patches.
- Prefer explicit types, clear interfaces, and discoverable public APIs.
- Avoid hidden global mutable state unless it is already an intentional repository pattern.
- Follow nearby module and crate conventions before adding abstractions.

## Code discovery

Before editing:

- inspect existing implementations, helpers, tests, examples, and docs;
- reuse local patterns before adding new abstractions;
- inspect public exports when public API changes;
- identify the owner, invariant, and boundary before patching;
- verify complete investigation gate evidence when the task requires it;
- verify complete design gate evidence when the task requires it.

## Documentation rules

Root Markdown files are short entrypoints and summaries. They must not become full workflow manuals, generated status views, design documents, roadmaps, or historical reports.

Canonical long-form documentation lives under:

```text
docs-site/src/content/docs
```

When root docs and docs-site docs overlap, update the docs-site authority first, then align the root summary.

## Validation and reporting

If command execution is unavailable, say so directly and use the routine's manual validation checklist.

Every final report must include:

```text
Files changed:
Exact functions/modules/sections changed:
Authority files inspected:
Complete investigation gate status:
Complete design gate status:
Manual validation performed:
Command validation run or unavailable:
Remaining risks or blockers:
Next recommended step:
```

Do not claim validation that was not run.
