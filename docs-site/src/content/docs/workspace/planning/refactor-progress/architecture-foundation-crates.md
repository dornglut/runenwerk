---
title: Architecture Foundation Crates Progress
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-26
---

# Architecture Foundation Crates Progress

Step 3 updates `ARCHITECTURE.md` so the root architecture summary lists the current foundation crates from the canonical crate inventory.

Patched root summary:

```text
foundation/id
foundation/id_macros
foundation/diagnostics
foundation/ratification
foundation/schema
foundation/commands
foundation/resource_ref
```

Validation needed:

```text
task docs:validate
git diff --check
```
