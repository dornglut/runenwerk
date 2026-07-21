---
title: Start Here
description: Canonical workspace router for Runenwerk engineering work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ./engineering-workflow.md
  - ./authority-model.md
  - ./documentation-structure.md
---

# Start Here

Use this page for non-trivial Runenwerk work.

## Primary entrypoints

```text
Human repository overview: README.md
AI agent entrypoint: AGENTS.md
Engineering process: docs-site/src/content/docs/workspace/engineering-workflow.md
Testing and validation: TESTING.md
```

## Read by task

### Code or refactor

```text
AGENTS.md
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
CRATES.md
TESTING.md
owning code, tests, ADRs, and designs
```

### Documentation

```text
AGENTS.md
docs-site/src/content/docs/workspace/engineering-workflow.md
docs-site/src/content/docs/workspace/documentation-structure.md
owning architecture, design, planning, or history document
```

### Architecture or extraction

```text
AGENTS.md
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
docs-site/src/content/docs/workspace/engineering-workflow.md
relevant accepted ADRs and designs
current code and conformance evidence
active GitHub issue and roadmap entry
```

### Pull-request review

Inspect:

```text
issue or accepted design
actual diff
owning tests and public surfaces
focused validation evidence
cargo validate and exact-head CI
known risks and post-merge truth
```

## Work classification

Classify the task using [Engineering Workflow](engineering-workflow.md):

```text
routine
significant
architectural or extraction
```

The classification controls how much investigation and durable design is required. It does not create a separate workflow state machine.

## Required baseline

Before merge:

```text
cargo validate
git diff --check
```

GitHub Actions runs the same `cargo validate` implementation.

## Legacy workflow tooling

Production-track, execution-lock, truth-certificate, batch, generated-prompt, routine, and task-card machinery is deprecated under issue `#122`.

It remains temporarily available only for active work that has not migrated. Do not use it for new work and do not treat it as workflow authority.

Historical links to superseded gate documents remain valid so old reports stay readable.
