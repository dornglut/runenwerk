---
title: Roadmap Intake WR-104
description: Governance-only roadmap intake proposal for the game-runtime UI projection and HUD production track.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-24
related:
  - ../../closeouts/pm-ui-design-010-production-readiness-and-evidence/closeout.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# Roadmap Intake WR-104

Idea: Game Runtime UI Projection And HUD Platform: governance-only intake for PT-GAME-RUNTIME-UI, starting with a later SDF screen-HUD runtime proof; no implementation in the first row.

Suggested title: Game Runtime UI Projection Governance And Track Activation

Initial planning state: `blocked_deferred`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.
- First row is governance/design only; no app, domain, engine, renderer, or SDF runtime implementation.
- Do not create `domain/game_ui`, `domain/game/interface`, or another game UI owner before an accepted design or ADR authorizes it.
- Use completed `PT-UI-DESIGN` and `WR-054` as the design-contract input; cite `PT-UI-LAB` only as an evidence-pattern reference.

## Boundaries

- `domain/ui/ui_definition` remains owner of generic UI definition, target-profile, binding, intent-reference, and diagnostics contracts.
- A future accepted game UI owner owns game-specific HUD vocabulary, view-model packages, and game intent adapters.
- `engine` owns runtime composition and UI expression submission only.
- The SDF example owns proof-specific state, view-model projection, and application of accepted SDF intents.
- Runtime UI composition must flow through scene output copied to `surface.color` followed by built-in UI composite, not `WindowState` title updates or debug overlay reuse.

## Open Questions

- Which follow-on WR IDs will `PM-GAME-RUNTIME-UI-002` through `PM-GAME-RUNTIME-UI-007` use after `WR-104`?
- Which game-runtime UI owner boundary is accepted: future `domain/game_ui`, `domain/game/interface`, or an extension under current UI definition contracts?
- Which runtime evidence checks are natively supportable, and which must remain explicit platform gaps?

## Apply Command

```text
task roadmap:apply-intake -- --proposal docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-game-runtime-ui-projection-and-hud-platf/proposal.yaml
```

## First Move After Acceptance

```text
task production:plan -- --milestone PM-GAME-RUNTIME-UI-001 --roadmap WR-104
```
