---
title: Typed App Program UI Proof 001 Planning
description: Superseded implementation-planning contract for the first Typed App Program proof. Preserved as historical pressure evidence; no longer the active implementation foundation.
status: superseded
owner: app-program
layer: workspace
canonical: false
last_reviewed: 2026-07-05
related_docs:
  - ./active-work.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
  - ../../reports/investigations/typed-app-program-current-state-investigation.md
  - ../../reports/investigations/typed-app-program-engine-pressure-and-design-review.md
  - ../../reports/investigations/typed-app-program-multiplayer-concurrency-design-review.md
  - ../../reports/investigations/typed-app-program-cross-cutting-design-review.md
  - ../../reports/investigations/typed-app-program-ui-proof-001-superseded-planning-archive.md
---

# Typed App Program UI Proof 001 Planning

## Status

Lifecycle state: `superseded`.

This planning contract is no longer the active implementation foundation.

Superseded by:

```text
PT-UI-FRAMEWORK-APP-INTEGRATION-001 — UI Framework App Integration Direction Review
```

Authority:

```text
docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Detailed historical archive:

```text
docs-site/src/content/docs/reports/investigations/typed-app-program-ui-proof-001-superseded-planning-archive.md
```

## Supersession reason

The original plan selected a dedicated `domain/app_program` crate plus a headless counter proof as the first implementation path.

That was a valid planning slice for the older question:

```text
Can Runenwerk define local app-program proof vocabulary around model/action/route/reducer/effect/projection/replay/report?
```

The current strategic question is different:

```text
How should real app/plugin authors use Runenwerk's UI framework correctly while preserving ui_definition source, UiProgram semantic contracts, UiStory proof, runtime output, host policy, and app-owned mutation?
```

The better long-term decision is now:

```text
App / Plugin / ECS-hosted app authoring
  + ui_definition-backed UI source
  + FormedInteractionModel / UiProgram semantic contracts
  + ui_runtime / ui_evaluator runtime output
  + UiStory proof and mount eligibility
  + host/app-owned mutation
```

The manual `app_program` proof-first path risks making normal app authors work directly with:

```text
AppModelSnapshot
RouteActionMap
AppAction decoder
AppReducer
AppViewProjection
AppReplayTrace
AppProgramReport
```

Those concepts may remain useful as later proof/report vocabulary, but they are not the selected public framework foundation.

## Preserved pressure from this plan

Future framework integration work should preserve these requirements from the superseded plan:

```text
fail-closed route/action resolution
distinct diagnostics for unknown route, schema mismatch, bad payload, missing capability, reducer failure, projection failure
safe bounded payload reports
stable action and route identity
no visible localized label as durable identity
rejected actions must not mutate app/domain state
UI emits proposals/events; host/app owners decide mutation
headless deterministic proof before production claims
no callback-first generic UI behavior
no renderer-owned product truth
no editor/game/engine mutation in generic UI
no shared plugin framework or foundation/meta extraction from the first proof
```

## Explicit non-authorizations

This superseded planning document no longer authorizes or requests:

```text
creating domain/app_program as the next active foundation
adding root workspace membership for app_program
implementing domain/app_program/src/*
adding domain/app_program/examples/headless_counter_ui.rs
adding domain/app_program/tests/headless_counter_replay.rs
merging PR #69 as the framework foundation
continuing manual AppModelSnapshot / RouteActionMap / reducer / projection implementation as the next framework step
```

## PR #69 status expectation

PR #69 may be closed or retained only as a superseded implementation spike. It should not be merged into `main` as the foundation for the Runenwerk UI framework unless a future accepted design explicitly reactivates this plan.

## Reactivation condition

Reactivate this plan only if a later accepted design concludes all of the following:

```text
1. A dedicated app_program crate is required before an ECS-hosted UI integration proof.
2. app_program is proof/report vocabulary, not the public framework authoring path.
3. The implementation does not bypass ui_definition, UiProgram, UiStory, or host/app mutation owners.
4. Exact owner files, allowed files, validation commands, module decomposition, and stop conditions are rewritten from current repository truth.
```

Until then, use `ui-framework-app-integration-direction-review.md` as the active direction authority.
