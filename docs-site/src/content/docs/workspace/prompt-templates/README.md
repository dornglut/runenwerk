---
title: Prompt Templates
description: Reusable Codex and AI-agent prompt templates for Runenwerk repository work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-08
---

# Prompt Templates

This folder contains reusable prompts for Codex and AI-assisted repository work.

Prompt templates are documentation artifacts. They do not define runtime behavior, domain invariants, foundation APIs, or AI integration code.

Use these templates when a task benefits from a repeatable instruction shape but still needs repository inspection before editing.

## Available Templates

- [Architecture Audit](./architecture-audit.md)
- [Architecture Governance Review](./architecture-governance-review.md)
- [Code Review](./code-review.md)
- [Commit Organization](./commit-organization.md)
- [Crate Design](./crate-design.md)
- [Documentation Refactor](./docs-refactor.md)
- [Goal Execution](./goal-execution.md)
- [Implementation Batch](./implementation-batch.md)
- [New Design Intake](./new-design-intake.md)
- [Parallel Roadmap Batch](./parallel-roadmap-batch.md)
- [Phase Completion Drift Check](./phase-completion-drift-check.md)
- [Production Implementation Contract](./production-implementation-contract.md)
- [Roadmap Milestone Kickoff](./roadmap-milestone-kickoff.md)

## Rules

- Treat templates as starting points, not automatic authority.
- Inspect relevant files before changing code.
- Name exact files and functions/modules for requested changes.
- Run the smallest relevant validation commands.
- Stop when validation fails and report the concrete failure.
- Do not use templates to bypass domain ownership, ratification, diagnostics, or dependency rules.
- Use architecture audit for findings-only review and architecture governance review for pre-implementation decision gates.
- Use parallel roadmap batch for approved fan-out work across independent roadmap slices.

## Related Docs

- [`../planning-and-implementation-workflow.md`](../planning-and-implementation-workflow.md)
- [`../agents.md`](../agents.md)
- [`../routines/README.md`](../routines/README.md)
- [`../documentation-structure.md`](../documentation-structure.md)
