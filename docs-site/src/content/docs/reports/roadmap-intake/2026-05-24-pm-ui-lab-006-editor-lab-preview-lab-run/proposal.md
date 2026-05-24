---
title: Roadmap Intake WR-087
description: Roadmap intake proposal for PM-UI-LAB-006 Preview Lab runtime evidence.
status: completed
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-24
---

# Roadmap Intake WR-087

Idea: PM-UI-LAB-006 Editor Lab preview lab runtime evidence matrix
Suggested title: UI Lab preview lab runtime evidence matrix
Planning state: `completed`

## Scope

WR-087 completes the PM-UI-LAB-006 implementation slice after the accepted Preview Lab runtime evidence design. It is archived with runtime-proven evidence for Editor Lab preview scenarios, retained visual artifacts, diagnostics snapshots, accessibility snapshots, performance snapshots, unsupported-check diagnostics, and degraded-provider proof.

The implementation boundary is app-owned preview and evidence harness work: preview scenarios, retained visual or screenshot-equivalent artifacts, diagnostics snapshots, accessibility unsupported-check diagnostics, performance snapshots, degraded-provider proof, and manifest-backed runtime evidence. `ui_definition` remains behavior-free.

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Decision Basis

- Accepted design: `docs-site/src/content/docs/design/accepted/ui-lab-preview-lab-runtime-evidence-design.md`
- Active productization design: `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`
- Completed dependency: WR-086 / PM-UI-LAB-005 project IO, diff/apply, activation, reload, and rollback runtime evidence
- Completed closeout: `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md`

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
