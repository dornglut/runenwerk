---
title: Workspace Routines
description: Scriptless repeatable routines for Runenwerk repository work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../start-here.md
  - ../operating-model.md
  - ../authority-model.md
  - ../workflow-lifecycle.md
  - ../complete-design-gate.md
  - ../task-cards/README.md
---

# Workspace Routines

Routines are the active process layer for Runenwerk work.

They must be usable from GitHub connector, ChatGPT context tooling, Codex patching, manual repo browsing, or a local checkout. Commands belong only in optional local helper sections.

Use [`../complete-design-gate.md`](../complete-design-gate.md) before implementation is authorized for architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work.

## Active routines

- [Investigation Routine](investigation-routine.md)
- [Implementation Routine](implementation-routine.md)
- [Architecture Governance Review Routine](architecture-governance-review-routine.md)
- [Code Refactor Routine](code-refactor-routine.md)
- [Documentation Refactor Routine](docs-refactor-routine.md)
- [Roadmap Update Routine](roadmap-update-routine.md)
- [Phase Completion Drift Check Routine](phase-completion-drift-check-routine.md)
- [Pull Request Review Routine](pr-review-routine.md)

## Routine shape

Every active routine should use this structure:

```text
Use when
Authority files to read
Working files to inspect
What to decide before editing
State transitions produced
Patch rules
Manual validation checklist
Stop conditions
Evidence to report
Optional local helpers
```

Existing routines may be normalized as they are touched.

## Rules

- Routines are bounded.
- Routines must identify authority files.
- Routines must include manual validation.
- Routines must name lifecycle state when work changes planning or authority.
- Routines must check complete design gate evidence when the work requires it.
- Routines must not require command execution.
- Routines must preserve unrelated work.
- Routines must report what changed, what was skipped, what was blocked, and what remains risky.

## Legacy specialized routines

Some older specialized routine files remain for link compatibility, but active work should route through the list above unless a specialized page has been explicitly refreshed to this scriptless shape.
