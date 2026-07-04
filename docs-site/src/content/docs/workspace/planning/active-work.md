---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-04
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
  - ./typed-app-program-ui-proof-001-planning.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
  - ../../reports/investigations/typed-app-program-current-state-investigation.md
  - ../../reports/investigations/typed-app-program-engine-pressure-and-design-review.md
  - ../../reports/investigations/typed-app-program-multiplayer-concurrency-design-review.md
  - ../../reports/investigations/typed-app-program-cross-cutting-design-review.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-APP-PROGRAM-001`

Title: `Typed App Program UI Proof 001 — Headless Counter App Proof`

State: implementation-planning intake selected; implementation is not authorized until the planning contract is reviewed and accepted.

Lifecycle state: `active-planning`

Owner: `domain/app_program` owns the first app-program crate and proof-local app-program contracts. UI is the first proving consumer through examples/tests, not the owner of the app-program architecture. Workspace planning owns this active-work record. Shared plugin/app composition ownership remains blocked until at least one non-UI proof validates repeated domain-neutral structure.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md`, `docs-site/src/content/docs/workspace/planning/typed-app-program-ui-proof-001-planning.md`, `docs-site/src/content/docs/design/active/typed-app-program-and-ui-proof-design.md`, `docs-site/src/content/docs/reports/investigations/typed-app-program-current-state-investigation.md`, `docs-site/src/content/docs/reports/investigations/typed-app-program-engine-pressure-and-design-review.md`, `docs-site/src/content/docs/reports/investigations/typed-app-program-multiplayer-concurrency-design-review.md`, and `docs-site/src/content/docs/reports/investigations/typed-app-program-cross-cutting-design-review.md`.

Evidence classes: `E3` local/source inspection from the current-state investigation, `E8` accepted investigation/design authority from merged PR #66 plus planning authority once this PR is merged, and future `E5` validation evidence required before implementation merge.

Complete investigation gate: satisfied by `typed-app-program-current-state-investigation.md` from merged PR #66. This planning PR builds on that merged authority and must remain planning-only.

Complete design gate: satisfied by `typed-app-program-and-ui-proof-design.md` plus companion engine/runtime, multiplayer/concurrency, and cross-cutting reviews from merged PR #66. This planning PR consumes those design-gate artifacts and does not expand scope.

Implementation contract: prepared by `typed-app-program-ui-proof-001-planning.md`; implementation is still not authorized by this active-work update alone. A future implementation branch must follow the exact allowed files, validation commands, evidence expectations, and stop conditions recorded in the planning contract.

Allowed files/crates: docs/planning files in this PR. For the future implementation proof only, the planning contract permits creating `domain/app_program` as the dedicated app-program crate, adding it to root `Cargo.toml`, implementing the crate modules under `domain/app_program/src/*`, adding `domain/app_program/examples/headless_counter_ui.rs`, and adding `domain/app_program/tests/headless_counter_replay.rs`. Only-if-required compiler/test wiring is limited to `domain/ui/ui_program/src/events/mod.rs` and `domain/ui/ui_hosts/src/lib.rs`.

Non-owned files/crates: all product/editor/game behavior, all engine scheduler/runtime/physics/asset-loading/streaming/LOD/render-resource/world-mutation behavior, networking/multiplayer/threading implementations, `foundation/meta`, command execution in `foundation/commands`, generic plugin framework/AppRecipe/PluginSuite behavior, Phase 17 SpatialCanvas implementation files, `ui_definition` callback behavior, `ui_controls` app mutation behavior, `ui_state` generic app model ownership, `ui_hosts` host effect execution, and production dependency from `app_program` into UI crates.

Principle compliance matrix: recorded in `typed-app-program-ui-proof-001-planning.md`. The first proof must remain one local app-program crate, one local headless counter proof, one demo/example, no actors/engine/multiplayer/editor/game/plugin work, separated IDs/model/action/route/reducer/effect/projection/replay/report modules, no runtime optimization, and communication only through explicit snapshots, packets, maps, and reports.

Module decomposition map: recorded in `typed-app-program-ui-proof-001-planning.md` as `domain/app_program/Cargo.toml`, `lib.rs`, `ids.rs`, `model.rs`, `action.rs`, `route_action.rs`, `reducer.rs`, `effect.rs`, `projection.rs`, `replay.rs`, `report.rs`, `counter.rs`, `examples/headless_counter_ui.rs`, and `tests/headless_counter_replay.rs`.

Maintainability review status: planning selected. Maintainability must be reviewed again in the implementation closeout against changed files, module separation, crate boundary, production dependency boundary, report shape, validation evidence, and non-owned boundary proof.

Feature support matrix: recorded in `typed-app-program-ui-proof-001-planning.md`; first proof requires model snapshot, action IDs/versions, route-action mapping, reducer trace, NoEffect effect plan, UI proof integration through example/test dev-dependencies, headless compatibility, deterministic replay, missing capability negative case, safe bounded payload summary, stable IDs, local source metadata, and no engine/multiplayer implementation.

Future-use-case pressure matrix: covered by the design and companion reviews for engine/runtime, multiplayer/concurrency, and cross-cutting concerns. The implementation proof must consume these as stop conditions, not implement those systems.

Hierarchy/composition matrix: first proof hierarchy is `AppModelSnapshot -> AppViewProjection -> UiProgram/UiOutput -> UiEventPacket -> RouteActionMap -> AppAction -> AppReducer -> AppEffectPlan -> AppReplayTrace`. The app-program crate may own local structural contracts, but AppRecipe/PluginSuite/shared framework extraction remains blocked.

Ergonomics/usability: first proof must be reviewable as a clear headless counter app replay in a crate example/test and must keep route/action/reducer/effect/report structures inspectable without hidden callbacks or whole-engine execution.

Validation expectation: this planning PR must pass `python tools/docs/validate_docs.py` and `git diff --check`. The future implementation PR must run `cargo test -p app_program`, `cargo test -p app_program --test headless_counter_replay`, `cargo test -p app_program --examples`, `cargo test -p ui_program event`, `cargo test -p ui_hosts route`, `cargo test -p ui_binding host_data`, `cargo test -p ui_evaluator`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`, or explain equivalent substitutions.

Known blockers: PR #66 is merged, so the remaining blocker is review and acceptance of this planning contract. No implementation should start until this planning PR is accepted.

Next action: validate this docs-only planning PR, review it as the implementation-planning contract, then open a separate implementation branch for `Typed App Program UI Proof 001 — Headless Counter App Proof` only after planning is accepted.

Evidence: this active-work update is based on the merged Typed App Program investigation/design/review authority and deliberately keeps implementation in `active-planning`, not `active-implementation`.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, stop conditions, principle compliance status, and module decomposition status are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Evidence classes:
Complete investigation gate:
Complete design gate:
Implementation contract:
Allowed files/crates:
Non-owned files/crates:
Principle compliance matrix:
Module decomposition map:
Maintainability review status:
Feature support matrix:
Future-use-case pressure matrix:
Hierarchy/composition matrix:
Ergonomics/usability:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
