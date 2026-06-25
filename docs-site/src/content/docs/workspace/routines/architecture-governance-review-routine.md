---
title: Architecture Governance Review Routine
description: Scriptless routine for architecture-sensitive Runenwerk changes.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ../authority-model.md
  - ../start-here.md
  - ../../guidelines/module-structure-guidelines.md
  - ../../adr/README.md
---

# Architecture Governance Review Routine

## Use when

Use this routine before implementation when a change may affect ownership, dependency direction, durable decision history, migration shape, tradeoffs, enforcement, or ownership mode.

## Authority files to read

- `AGENTS.md`
- `AI_GUIDE.md`
- `ARCHITECTURE.md`
- `DEPENDENCY_RULES.md`
- `DOMAIN_MAP.md`
- `GLOSSARY.md`
- `docs-site/src/content/docs/workspace/authority-model.md`
- owning design, ADR, guideline, roadmap, or domain doc

## Working files to inspect

Inspect the affected crates, modules, tests, docs, examples, and public API entrypoints.

## What to decide before editing

- Owning domain, crate, and subsystem.
- Whether an ADR or design update is needed.
- Whether the change is implementation, prototype, documentation, defer, or reject.
- Which invariants and dependency directions must hold.

## Patch rules

Do not implement product code from this review unless the task explicitly includes implementation. Prefer a recommendation, design patch, ADR patch, or scoped implementation handoff.

## Manual validation checklist

- Domain owner named.
- Dependency direction checked.
- ADR/design need decided.
- Migration shape named when applicable.
- Public API and docs impact considered.
- Stop conditions named.

## Evidence to report

Report recommendation, scope, owner, dependency direction, ADR/design need, validation, stop conditions, and next action.

## Optional local helpers

Local tests, docs validators, and search commands may add evidence when available, but this routine must work without them.
