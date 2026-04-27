---
title: Architecture Audit Prompt
description: Prompt template for auditing Runenwerk architecture, boundaries, and refactor priorities.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
  - ../../guidelines/architecture.md
  - ../../guidelines/runenwerk-architecture.md
---

# Architecture Audit Prompt

Use this template when asking Codex or another AI agent to audit architecture.

## Template

```text
Audit the current Runenwerk repository architecture for the following scope:

Scope:
- <crate/domain/subsystem/files>

Before giving recommendations:
1. Inspect the relevant files first.
2. Do not treat docs as truth when code contradicts docs.
3. Identify current ownership boundaries.
4. Identify dependency direction and layering.
5. Identify public API and usage ergonomics issues.
6. Identify documentation drift caused by the current code.
7. Do not guess. If evidence is missing, say exactly what is missing.

Output:
1. Findings ordered by severity.
2. Exact file paths and function/module locations.
3. Boundary or doctrine violated, if any.
4. Recommended fixes in priority order.
5. Validation commands to run.
6. Suggested commit split.

Do not implement changes unless explicitly asked.
```

## Expected Agent Behavior

The agent should produce findings first, not a speculative redesign.

Recommend cleanup before redesign unless there is clear semantic inconsistency, duplicated architectural logic, repeated public API friction, or unclear ownership that cannot be solved locally.
