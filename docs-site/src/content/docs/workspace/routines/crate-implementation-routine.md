---
title: Crate Implementation Routine
description: Bounded routine for implementing new Runenwerk crates or major crate phases.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../../guidelines/architecture.md
  - ../prompt-templates/crate-design.md
---

# Crate Implementation Routine

## Purpose

Use this routine when implementing a new crate or a major crate phase from an accepted design.

## Preconditions

Before implementation:

1. Confirm the crate boundary is already designed.
2. Confirm the crate belongs in the selected layer.
3. Confirm dependencies obey repository direction.
4. Confirm workspace membership and crate inventory impact.
5. Identify tests and docs required for the public surface.

## Routine

1. Read the crate design or current crate docs.
2. Inspect adjacent crates for naming, module, and API patterns.
3. Create or update `Cargo.toml`.
4. Add module skeletons by responsibility, not technical layer.
5. Implement public vocabulary first.
6. Implement validation, ratification, or reporting logic second when relevant.
7. Add tests for invariants and serialization when relevant.
8. Update root and docs-site crate maps when workspace membership changes.
9. Run focused tests.
10. Run workspace checks when dependencies changed.

## Required Validation

```text
cargo fmt --all -- --check
cargo test -p <crate>
cargo check --workspace
task docs:validate
```

## Stop Conditions

Stop and report when:

- the crate boundary is not justified;
- implementation requires a forbidden dependency;
- the design contradicts current code;
- the crate becomes a god abstraction;
- public API is technically correct but awkward to discover.

## Final Report

Include:

- crate path;
- public API surface;
- dependency decisions;
- tests added;
- docs updated;
- remaining phase work.
