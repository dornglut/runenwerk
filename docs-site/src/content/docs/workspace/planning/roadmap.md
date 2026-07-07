---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../workflow-lifecycle.md
  - ../../architecture/ui-framework-architecture.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
---

# Roadmap

This is the Markdown-first roadmap record for scriptless workflow.

## Current entries

### PT-UI-FRAMEWORK-APP-INTEGRATION-001

ID: `PT-UI-FRAMEWORK-APP-INTEGRATION-001`

Title: UI Framework App Integration Direction Review

State: accepted direction; no longer the current implementation focus

Lifecycle state: `accepted-direction`

Authority: `ui-framework-app-integration-direction-review.md`.

Evidence: PR #70 accepted the direction that App/Plugin/ECS-hosted UI must lower through `ui_definition`, `UiProgram`, `UiStory`, runtime/evaluator artifacts, and host-owned mutation instead of continuing the manual `app_program` proof or promoting SpatialCanvas as the app-framework answer.

Next action: Keep as accepted direction authority. The first proof slice, `PT-UI-FRAMEWORK-APP-INTEGRATION-002`, is completed and now serves as evidence for runtime-platform planning.

### PT-UI-FRAMEWORK-APP-INTEGRATION-002

ID: `PT-UI-FRAMEWORK-APP-INTEGRATION-002`

Title: ECS-backed Counter UI Story Proof

State: completed through PR #72 and closeout truth

Lifecycle state: `completed`

Authority: `ecs-backed-counter-ui-story-proof-planning.md`.

Evidence: PR #72 merged the `ui_app_integration` proof into `main` at `e093eb1affdc465b96430200960f8e3cdca0d26b`. Closeout evidence records code-authored Counter and Win source records, lowering through `ui_definition` and `ui_program`, route/event evidence, route-missing diagnostics, route-resolved host mutation, ECS-backed Counter mutation, positive proof flow, fail-closed cases, no public `AppUiExt`, no engine `UiPlugin`, no render adapter/runtime-visible render proof, no SDF/SpatialCanvas world-space implementation, no `foundation/meta`, no `domain/app_program`, and no generic plugin framework.

Next action: Keep as completed proof evidence. Do not reopen this slice unless future inspection finds the recorded boundary was violated.

### PT-UI-RUNTIME-PLATFORM-001

ID: `PT-UI-RUNTIME-PLATFORM-001`

Title: Live UiPlugin Runtime and Generic Surface-Frame Rendering

State: completed design-gate hardening through merged PR #74; implementation not authorized by this entry

Lifecycle state: `active-planning` design-gate complete / superseded by full cutover-plan focus

Authority:

```text
../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
../../architecture/ui-framework-architecture.md
../../design/active/ui-framework-app-integration-direction-review.md
../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
```

Evidence: PR #74 merged the docs-only investigation/design-gate hardening. The accepted target remains `RenderPlugin + UiPlugin + AppPlugin`, `app.mount_ui(Screen)`, typed `UiScreen`, typed `IntoUi`, typed `UiActionHandler` / `TryUiActionHandler`, host-owned mutation, reuse of existing `ui_surface` and `ui_hosts` owners, and staged generic surface-frame render publication. Runtime Rust implementation remains outside PR #74.

Gate status:

```text
Complete investigation gate: complete for PR #74 design-gate hardening.
Complete design gate: complete for opening implementation planning.
Implementation authorization: still forbidden by this entry.
```

Next action: Use as authority for `PT-UI-RUNTIME-PLATFORM-002`, the full platform cutover plan.

### PT-UI-RUNTIME-PLATFORM-002

ID: `PT-UI-RUNTIME-PLATFORM-002`

Title: Live UiPlugin Runtime Full Platform Cutover Plan

State: draft docs-only implementation-planning PR in progress

Lifecycle state: `active-planning` full-platform cutover contract draft

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
../../architecture/ui-framework-architecture.md
```

Evidence: This entry corrects the previous first-runtime-slice framing. The platform is planned as a full cutover, then implemented through bounded phase PRs: UiPlugin foundation, app mounting API, typed screen/source/action contracts, mounted surface/session runtime, host action dispatch with UI-runtime trace, runtime evaluation with state snapshot and invalidation, producer-generic surface-frame boundary, UiPlugin render publication, scene/debug overlay producer migration/retirement, runnable human/agent Counter app product, source reload and persistence contract, and closeout/adoption lock. SDF UI backend work is assigned to downstream `PT-UI-RENDER-BACKEND-SDF-001`; phase implementation specs are downstream workflow hardening, not part of this planning PR.

Gate status:

```text
Complete investigation gate: inherited from PT-UI-RUNTIME-PLATFORM-001 and extended with render/app-engine feature mapping.
Complete design gate: in progress for full cutover contract.
Implementation authorization: forbidden until this planning PR is accepted and the next phase PR opens.
```

Next action: Review the full cutover plan. If accepted, merge it and open `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` as the first bounded implementation phase.

### PT-UI-COMPONENT-PLATFORM-013

ID: `PT-UI-COMPONENT-PLATFORM-013`

Title: Overlay / Popup / Layering full implementation

State: completed through merged PR #44

Lifecycle state: `completed`

Evidence: PR #44 merged into `main` at `6f2d3827f315191d7aeaf68a64f523627197cad8`. Evidence covers package-backed overlay declarations, base-control overlay lowering, main-path package validation, catalog projection, inspection projection, normalized input fact consumption, runtime overlay proof, proof-frame projection, static mount proof, route-guard evidence, and full local validation gate passed on 2026-07-02.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-014

ID: `PT-UI-COMPONENT-PLATFORM-014`

Title: Text Editing / Editable Text Behavior

State: completed through merged PR #46

Lifecycle state: `completed`

Authority: `ui-component-platform-text-editing-design.md`.

Evidence: PR #46 merged into `main` at `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`. Main contains package-backed editable-text declarations, InspectorField text-editing lowering, package descriptor wiring, package validation, catalog projection, inspection projection, normalized text edit/composition/selection facts, runtime text-editing proof, proof-frame projection, static mount validation, focused tests, final proof-frame cleanup, and full local validation gate passed on 2026-07-02.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-015

ID: `PT-UI-COMPONENT-PLATFORM-015`

Title: Generic Text

State: completed through baseline PR #48 and hardening PR #49

Lifecycle state: `completed`

Authority: `ui-component-platform-generic-text-design.md`.

Evidence: PR #48 merged into `main` at `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`. PR #49 merged into `main` at `338a8092d534dbb412da89363d50a46cd5efeae9` and completed the hardening pass. Final validation passed with the recorded package/workspace/docs/diff gate.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-016

ID: `PT-UI-COMPONENT-PLATFORM-016`

Title: Surface2D

State: completed through docs-hardening PR #62 and implementation PR #61

Lifecycle state: `completed`

Authority: `ui-component-platform-surface2d-design.md`.

Evidence: PR #62 merged docs-only workflow, principle, decomposition, and merge-readiness hardening at `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`. PR #61 squash-merged the Phase 16 Surface2D implementation at `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. Post-merge validation from `main` passed with the recorded Surface2D focused commands, workspace tests, docs validation, and diff check.

Next action: Keep as completed dependency. Keep Phase 17 SpatialCanvas as downstream planning only until runtime platform planning settles its accepted implementation path.

## Future app-framework follow-ups

These are downstream planning candidates only. They are not implementation work and must wait for their own accepted planning/design contracts.

- `PT-UI-FRAMEWORK-APP-INTEGRATION-003 - Public AppUiExt Ergonomics`
- `PT-UI-FRAMEWORK-APP-INTEGRATION-004 - Authoring Frontends and Execution Strategy Model`

`PT-UI-FRAMEWORK-APP-INTEGRATION-003` is superseded/absorbed by `PT-UI-RUNTIME-PLATFORM-001` and `PT-UI-RUNTIME-PLATFORM-002` as the broader live runtime platform track. Do not treat standalone public `AppUiExt` ergonomics as the immediate next implementation target.

`PT-UI-FRAMEWORK-APP-INTEGRATION-004` should define how Rust builders, templates, visual designer output, compiler DSLs, immediate-mode adapters, reactive adapters, retained execution, ECS-driven execution, and SDF/world-space targets share source/program/event/story contracts through the accepted route and proof model.

## Rules

- Markdown must be enough to understand the current state.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
- Use `../workflow-lifecycle.md` before changing lifecycle state.
- Accepted direction does not authorize implementation.
