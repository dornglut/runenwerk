---
title: Code Refactor Routine
description: Bounded routine for safe code refactors across Runenwerk crates and subsystems.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
  - ../../guidelines/architecture.md
---

# Code Refactor Routine

## Purpose

Use this routine for code refactors that preserve behavior while improving structure, names, boundaries, or public API ergonomics.

## Preconditions

Before editing:

1. Identify the owning crate and subsystem.
2. Inspect nearby modules and tests.
3. Identify public API impact.
4. Identify docs impact.
5. Identify validation commands.
6. Avoid broad speculative reshuffles.

## Routine

1. Capture current state:
   - `git status --short`
   - relevant `git diff -- <path>`
2. Inspect current implementation and call sites.
3. Define the smallest coherent refactor.
4. Apply the refactor.
5. Run formatting.
6. Run focused tests.
7. Run broader checks only when the refactor crosses crate boundaries.
8. Update docs when public behavior, ownership, or usage changes.
9. Report changed files, functions/modules, validation, and follow-up risks.

## Required Validation

Use the smallest relevant set:

```text
cargo fmt --all -- --check
cargo test -p <crate>
cargo check --workspace
```

## Stop Conditions

Stop and report when:

- ownership is unclear;
- a dependency direction violation would be required;
- behavior changes are needed but not requested;
- validation failure is outside the refactor scope;
- the refactor expands into unrelated domains.

## Final Report

Include:

- changed files;
- exact functions/modules changed;
- behavior preserved or intentionally changed;
- validation commands run;
- remaining risks.
