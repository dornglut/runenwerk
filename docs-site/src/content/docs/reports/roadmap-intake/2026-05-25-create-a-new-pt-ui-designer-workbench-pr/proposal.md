---
title: Roadmap Intake WR-120
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-25
---

# Roadmap Intake WR-120

Idea: Create `PT-UI-DESIGNER-WORKBENCH` as a governed product track for the real standalone and embedded UI Designer workbench.

Suggested title: UI Designer Workbench Product Track Governance
Initial planning state: `blocked_deferred`

## Proposed Scope

`WR-120` is governance and planning only. It should:

- reconcile code truth and planning truth against archived `WR-114` / `PM-EDITOR-UX-004`;
- install or propose the new `PT-UI-DESIGNER-WORKBENCH` production track;
- define follow-on implementation rows with disjoint write scopes;
- keep generic UI truth in `domain/ui`;
- keep editor workbench adapters in `domain/editor`;
- keep app execution and evidence in `apps/runenwerk_editor`;
- keep game-runtime UI implementation downstream of a compatibility seam.

No product code should be implemented under `WR-120`.

## Product Milestone Summary

- Governance, code-truth reconciliation, and production-track activation.
- V1 package/document/session/evidence model.
- Standalone UI Designer app shell and binary.
- Catalog, hierarchy, canvas, inspector, diagnostics, and diff/review V1.
- Operation-driven visual authoring over Canonical UI IR diffs.
- Scenario matrix, source-versioned evidence, and performance/resize baselines.
- Embedded `Editor Design` parity using the same product contract.
- `game.runtime` compatibility seam proof without HUD implementation.

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- Which V1 package persistence format is allowed for the first implementation row?

## Track Decision

Use a new production track:

```text
PT-UI-DESIGNER-WORKBENCH
```

This track does not reopen `PT-EDITOR-UX`. It treats archived `WR-114` /
`PM-EDITOR-UX-004` as bounded workbench-route evidence, then creates a stricter
productization repair path for the full V1 workbench contract.

`WR-120` may reconcile planning interpretation and cross-links, but it must not
rewrite the historical `WR-114` closeout unless a separate docs-refactor or
closeout-correction row explicitly authorizes that edit.

## Proposed Production Milestone Skeleton

| Milestone | Title | Kind | Roadmap |
|---|---|---|---|
| `PM-UI-DESIGNER-WB-001` | Governance Code Truth And Track Activation | design | `WR-120` |
| `PM-UI-DESIGNER-WB-002` | V1 Package Document Session And Evidence Model | design | TBD |
| `PM-UI-DESIGNER-WB-003` | Standalone App Shell And Embedded Host Parity | implementation | TBD |
| `PM-UI-DESIGNER-WB-004` | Catalog Hierarchy Canvas Inspector V1 | implementation | TBD |
| `PM-UI-DESIGNER-WB-005` | Operation Diff Apply And Rollback | implementation | TBD |
| `PM-UI-DESIGNER-WB-006` | Scenario Evidence And Performance Baselines | hardening | TBD |
| `PM-UI-DESIGNER-WB-007` | Game Runtime Compatibility Seam | hardening | TBD |
| `PM-UI-DESIGNER-WB-008` | Runtime Proven Closeout And Handoff | release | TBD |

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
