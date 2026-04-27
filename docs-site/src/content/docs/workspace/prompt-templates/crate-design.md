---
title: Crate Design Prompt
description: Prompt template for designing or revising Runenwerk crate architecture.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../../guidelines/architecture.md
  - ../../guidelines/runenwerk-architecture.md
---

# Crate Design Prompt

Use this template before adding a new crate or redesigning an existing crate boundary.

## Template

```text
Design or revise the Runenwerk crate:

Crate:
- <crate path/name>

Goal:
- <goal>

Constraints:
- Respect foundation -> domain -> engine/runtime -> apps/adapters/tools.
- Do not put runtime/app/editor/backend concerns into foundation.
- Do not put concrete runtime glue into pure domain crates.
- Prefer small, reusable contracts over god abstractions.
- Public APIs should be discoverable and easy to use correctly.

Required process:
1. Inspect existing crates/docs that may already own the concept.
2. Challenge whether a new crate is needed.
3. Compare alternatives:
   - no new crate
   - smaller module
   - new crate
   - split crate
   - move to engine/runtime
4. Pick the best long-term boundary.
5. Define purpose, scope, non-scope, dependencies, invariants, public API shape, validation, and docs impact.

Output:
1. Boundary decision.
2. Alternatives rejected and why.
3. Proposed module/file structure.
4. Public API shape.
5. Dependency rules.
6. Test plan.
7. Documentation updates.
8. Phased implementation plan.

Do not implement unless explicitly asked.
```
