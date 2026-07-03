---
title: Architecture Governance Review Routine
description: Scriptless routine for architecture-sensitive Runenwerk changes.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../authority-model.md
  - ../start-here.md
  - ../workflow-lifecycle.md
  - ../complete-design-gate.md
  - ../../guidelines/programming-principles.md
  - ../../guidelines/module-structure-guidelines.md
  - ../../adr/README.md
---

# Architecture Governance Review Routine

## Use when

Use this routine before implementation when a change may affect ownership, dependency direction, durable decision history, migration shape, tradeoffs, enforcement, ownership mode, public API ergonomics, feature support, hierarchy/composition, or reusable platform capability.

## Authority files to read

- `AGENTS.md`
- `ARCHITECTURE.md`
- `DEPENDENCY_RULES.md`
- `DOMAIN_MAP.md`
- `GLOSSARY.md`
- `docs-site/src/content/docs/workspace/authority-model.md`
- `docs-site/src/content/docs/workspace/workflow-lifecycle.md`
- `docs-site/src/content/docs/workspace/complete-design-gate.md`
- `docs-site/src/content/docs/guidelines/programming-principles.md`
- owning design, ADR, guideline, roadmap, or domain doc

## Working files to inspect

Inspect affected crates, modules, tests, docs, examples, public API entrypoints, existing user-facing paths, and prior phase closeout evidence.

## What to decide before editing

- Owning domain, crate, and subsystem.
- Whether an ADR or design update is needed.
- Whether the change is investigation, proposed design, accepted direction, implementation, documentation, defer, or reject.
- Which invariants and dependency directions must hold.
- Whether architecture acceptance is enough or whether implementation authorization is also needed.
- Whether `complete-design-gate.md` applies.
- Whether the design has a complete capability map.
- Whether the design has a feature support matrix.
- Whether the design has a future-use-case pressure matrix.
- Whether the design has a hierarchy/composition matrix when relevant.
- Whether the design has an ergonomics and usability contract.
- Whether every non-delivered capability has a named downstream owner, named contract, and activation condition.

## State transitions produced

This routine may produce:

```text
idea -> investigating
investigating -> proposed-design
proposed-design -> accepted-direction
proposed-design -> deferred
proposed-design -> rejected
accepted-direction -> superseded
```

It must not produce `active-implementation` unless a separate implementation scope is explicitly requested and planning authority already authorizes it with complete design gate evidence where required.

## Patch rules

Do not implement product code from this review unless the task explicitly includes implementation and active planning already authorizes the complete implementation contract. Prefer a recommendation, design patch, ADR patch, or scoped implementation path.

For architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work, include or require complete design gate evidence before recommending implementation.

## Manual validation checklist

- Domain owner named.
- Dependency direction checked.
- ADR/design need decided.
- Lifecycle state and intended transition named.
- Complete design gate applicability checked.
- Complete capability map checked where applicable.
- Feature support matrix checked where applicable.
- Future-use-case pressure matrix checked where applicable.
- Hierarchy/composition matrix checked where applicable.
- Ergonomics and usability contract checked where applicable.
- Seven programming principles applied.
- Public API and docs impact considered.
- Stop conditions named.

## Stop conditions

Stop and redesign if the review would require broad shared extraction without a proving domain, implementation without active scope, root docs as long-form authority, generated views as default authority, cross-domain ownership drift, or implementation authorization without complete design gate evidence where required.

## Evidence to report

Report recommendation, scope, owner, dependency direction, ADR/design need, complete design gate status where applicable, lifecycle transition, validation, stop conditions, and next action.

## Optional local helpers

Local tests, docs validators, and search commands may add evidence when available, but this routine must work without them.
