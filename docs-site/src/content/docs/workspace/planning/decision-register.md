---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ./ecs-backed-counter-ui-story-proof-planning.md
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
  - ../../design/active/runenwerk-typed-app-composition-plugin-framework-design.md
---

# Decision Register

Use this file to explain planning priority changes and lifecycle state transitions. Detailed historical evidence belongs in closeouts, investigation reports, and owning design files; this register records the decision and the evidence pointer.

## Phase 13 closeout decision

Date: 2026-07-02

Decision: Mark `PT-UI-COMPONENT-PLATFORM-013` Overlay / Popup / Layering full implementation completed through merged PR #44.

State transition: `review -> completed`

Evidence: PR #44 merged into `main` at `6f2d3827f315191d7aeaf68a64f523627197cad8`; local validation passed on 2026-07-02; package-backed overlay declarations, runtime overlay proof, proof-frame projection, static mount proof, and route-guard evidence are present.

Follow-up: Keep Phase 13 as completed dependency.

## Phase 14 text editing planning and completion decisions

Date: 2026-07-02

Decision: Start, implement, review, and mark `PT-UI-COMPONENT-PLATFORM-014` Text Editing / Editable Text Behavior completed through merged PR #46.

State transition: `production-track -> active-planning -> active-implementation -> review -> completed`

Evidence: Implementation covered editable-text vocabulary, descriptor wiring, package validation, InspectorField lowering, catalog and inspection projection, normalized edit/composition/selection facts, runtime proof-frame evidence, static mount validation, route-guard evidence, focused tests, and local validation on 2026-07-02. PR #46 merged into `main` at `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`.

Follow-up: Keep Phase 14 as completed dependency.

## Phase 15 generic text decisions

Date: 2026-07-02

Decision: Start, implement, harden, and mark `PT-UI-COMPONENT-PLATFORM-015` Generic Text completed through PR #48 baseline and PR #49 hardening.

State transition: `production-track -> active-planning -> active-implementation -> review -> completed -> completed-hardening`

Evidence: PR #48 merged into `main` at `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`; PR #49 merged into `main` at `338a8092d534dbb412da89363d50a46cd5efeae9`; final validation passed with the recorded package/workspace/docs/diff gate.

Follow-up: Use PR #48 plus PR #49 as authoritative Phase 15 completion evidence.

## Phase 16 Surface2D planning and completion decisions

Date: 2026-07-02 to 2026-07-03

Decision: Start `PT-UI-COMPONENT-PLATFORM-016` as Surface2D planning, then mark it completed through docs-hardening PR #62 and implementation PR #61.

State transition: `production-track -> active-planning -> review -> completed`

Evidence: PR #62 merged workflow/principle/decomposition/merge-readiness hardening at `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`; PR #61 squash-merged Surface2D at `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`; post-merge validation from `main` passed.

Follow-up: Keep Phase 16 as completed dependency and keep Phase 17 SpatialCanvas downstream-only until runtime platform work is settled.

## Phase 16 leftover branch decision

Date: 2026-07-03

Decision: Keep remote branch `surface2d-phase-16` for manual review instead of folding it into Phase 16 closeout.

State transition: none

Evidence: Branch comparison showed three extra commits and one large design-document diff with potentially useful downstream-use-case pressure mixed with stale assumptions.

Follow-up: Review manually outside Phase 16 closeout and extract only focused useful docs material if needed.

## UI framework app integration direction decision

Date: 2026-07-05

Decision: Replace the active next-work focus from `PT-UI-APP-PROGRAM-001` manual Typed App Program headless counter implementation to `PT-UI-FRAMEWORK-APP-INTEGRATION-001` UI Framework App Integration Direction Review.

State transition: `PT-UI-APP-PROGRAM-001 active-planning/implementation-spike -> superseded`; `PT-UI-FRAMEWORK-APP-INTEGRATION-001 -> active-planning`.

Reason: The best long-term direction is App/ECS-hosted app authoring that lowers UI through `ui_definition`, FormedInteractionModel, `UiProgram`, runtime artifacts, and `UiStory` reports while keeping app mutation in host/app owners.

Follow-up: Keep `ui-framework-app-integration-direction-review.md` as accepted direction authority.

## ECS-backed Counter UI Story Proof implementation-planning decision

Date: 2026-07-05

Decision: Convert `PT-UI-FRAMEWORK-APP-INTEGRATION-002` from broad proof-planning intake into a conditional implementation contract for a small UI-owned `ui_app_integration` crate and an ECS-backed Counter UiStory-compatible proof.

State transition: `active-planning intake -> active-planning implementation contract`; implementation branch authorized only after the docs PR was reviewed/merged and the implementation stayed within the accepted contract.

Reason: Internal proof first was better than immediate public AppUiExt API: create a small UI-owned integration bridge, prove the Counter loop through internal/proof-local APIs, use `ui_story`/`ui_testing` only as proof harnesses, and defer public engine ergonomics until dependency direction and owner boundaries were validated.

Evidence: `Cargo.toml` had no `ui_app_integration` member yet; inspected UI crates provided the required source/program/route/control/story/proof contracts; `engine::App` world state was private.

Follow-up: After docs acceptance, implement only the allowed `ui_app_integration` proof scope.

## ECS-backed Counter UI Story Proof closeout decision

Date: 2026-07-06

Decision: Mark `PT-UI-FRAMEWORK-APP-INTEGRATION-002` ECS-backed Counter UI Story Proof completed through merged PR #72 and the closeout report.

State transition: `review -> completed`

Reason: The delivered PR #72 scope matched the accepted planning contract for the bounded proof and preserved recorded non-goals: no public `AppUiExt`, engine `UiPlugin`, render adapter, SDF/SpatialCanvas/world-space implementation, `foundation/meta`, `domain/app_program`, generic plugin framework, callback/direct mutation shortcut, or ECS-owned durable UI semantic model.

Evidence: PR #72 merged on 2026-07-06 at `e093eb1affdc465b96430200960f8e3cdca0d26b`; source inspection covered the new crate, source modules, tests, and docs index link.

Follow-up: Use PR #72 only as proof-local evidence for runtime-platform planning.

## Live UiPlugin runtime and surface-frame rendering intake decision

Date: 2026-07-06

Decision: Create/review `PT-UI-RUNTIME-PLATFORM-001` as the proposed-design / active-planning intake for the live public UI runtime platform instead of continuing with a narrow public `AppUiExt` ergonomics slice or a broad `domain/ui/ui_runtime_platform` crate.

State transition: `idea/investigating -> proposed-design / active-planning intake review`; implementation remains not authorized.

Reason: The best long-term path is an engine-owned `UiPlugin` runtime layer using existing domain UI contracts and a producer-agnostic render surface-frame boundary. A public AppUiExt-only slice would freeze ergonomics before the runtime path is proven; render-adapter-only work would not solve mounted sessions, typed actions, or host mutation.

Follow-up: Harden PR #74 / `PT-UI-RUNTIME-PLATFORM-001` intake before implementation.

## Live UiPlugin runtime design-gate hardening decision

Date: 2026-07-06

Decision: Mark `PT-UI-RUNTIME-PLATFORM-001` complete investigation/design gate hardening complete for PR #74, while keeping runtime implementation blocked until a separate implementation-planning PR records the exact implementation contract.

State transition: `active-planning intake review -> active-planning design-gate complete / implementation-planning required`.

Evidence: `E2` connector metadata/file inspection, `E3` source/test inspection by path, and `E8` accepted architecture/workflow/planning authority. Local command validation was unavailable in the connector-only session.

Follow-up: Open implementation planning after PR #74 review/merge.

## Live UiPlugin runtime full cutover-planning decision

Date: 2026-07-07

Decision: Start and harden `PT-UI-RUNTIME-PLATFORM-002` as a full-platform cutover-planning PR instead of a narrow first-slice planning PR.

State transition: `active-planning design-gate complete / implementation-planning required -> active-planning full-platform cutover contract draft`.

Context: Critical review found that the initial PR #76 framing was directionally correct but not handoff-complete. It used transitional-producer language, did not require a runnable product-grade Counter app, did not make the app agent-controllable, lacked a generic UI-runtime trace/history plan, lacked source reload and persistence decisions, kept architecture/PlantUML material inside the planning document instead of architecture docs, and scheduled render-boundary genericization too late.

Options considered: keep the first-slice-only plan; make one giant implementation PR; create a full cutover plan and execute it through gated phase PRs.

Reason: The correct path is to plan the full platform cutover now while still implementing it through bounded PRs. The accepted runtime direction must include prior UI path retirement, producer-generic surface-frame semantics before UiPlugin render publication, render/app-engine feature mapping, generic UI-runtime trace/history, agent-controllable operation, source reload and persistence boundaries, a runnable `apps/ui_counter_runtime` product, SDF UI backend assignment to `PT-UI-RENDER-BACKEND-SDF-001`, phase-spec workflow hardening as downstream work, architecture-owned PlantUML diagrams, and phase-by-phase gates that a simple implementation agent can follow without inventing architecture.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `design/active/README.md`, `../../design/active/live-uiplugin-runtime-full-cutover-plan.md`, `../../architecture/live-uiplugin-runtime-platform-architecture.md`, and its PlantUML diagram files.

Evidence: PR #74 merge metadata, accepted PR #74 design/investigation authority, PR #72 proof-local closeout, connector inspection of current engine app/render code, and connector inspection of PR #76.

Follow-up: Review PR #76 as docs-only full cutover planning. If accepted, merge it and open `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` as the first bounded implementation PR.

Reactivation condition: Reopen this decision if review finds a missing render/app-engine feature, incomplete agent/product acceptance criterion, a preserved prior path, incomplete trace/history model, missing source reload/persistence boundary, render-boundary genericization scheduled after code depends on non-target ownership names, or a phase that cannot be handed to a simple implementation agent.

Supersedes: the earlier “first runtime slice” framing after PR #74.

Superseded by: none.

## Track orchestration and phase spec workflow decision

Date: 2026-07-07

Decision: Add `PT-WORKFLOW-TRACK-ORCHESTRATION-001` as a docs-only workflow hardening step before `PT-UI-RUNTIME-PLATFORM-003` implementation begins.

State transition: `PT-UI-RUNTIME-PLATFORM-002 active-planning full-platform cutover contract draft -> PT-WORKFLOW-TRACK-ORCHESTRATION-001 active-planning workflow-hardening contract draft`.

Context: PR #76 established the full Live UiPlugin Runtime Platform cutover and recorded that implementation must proceed through bounded phase PRs. Review then identified one missing workflow layer: the repo had strong routines for implementation, PR review, roadmap updates, and phase completion, but no explicit manager/orchestrator routine for a long production-track goal executed through multiple phase PRs.

Options considered: start `PT-UI-RUNTIME-PLATFORM-003` immediately using the existing routines; add only a prompt/task card; add a docs-only track orchestration routine plus phase-spec handoff docs; add validator/tooling immediately.

Reason: The correct next step is to add the workflow layer before runtime implementation. A one-shot track goal is valid as manager intent because it can own the whole destination, phase order, planning truth, PR readiness, and next-phase activation. It is not valid as one implementation PR because active implementation still requires one bounded phase with exact owner, scope, validation, evidence, and stop conditions. Phase specs should be RON handoff contracts derived from accepted Markdown authority, not replacements for Markdown design/process/planning truth. JSONL is reserved for append-only event/log/trace streams such as runtime traces, agent output, validation/proof logs, and a possible future track-manager execution ledger. Validator/tooling is deferred because the spec shape should be reviewed and exercised before scripts harden it.

Affected planning files: `active-work.md`, `decision-register.md`, `../start-here.md`, `../authority-model.md`, `../planning/README.md`, `../routines/README.md`, `../task-cards/README.md`, `../routines/track-orchestration-routine.md`, `../task-cards/track-manager-task.md`, `../specs/README.md`, `../specs/phase-implementation-spec.md`, and `../specs/templates/phase-implementation-spec.ron`.

Evidence: `AGENTS.md`, root architecture/domain/testing summaries, workspace operating model, authority model, lifecycle, implementation routine, PR review routine, phase completion drift check, roadmap update routine, planning records, PR #76 merge state, and accepted runtime-platform cutover docs.

Follow-up: Review and merge this docs-only workflow PR. After it merges, open `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` as a separate active-implementation PR using the new track orchestration routine and phase spec handoff rules.

Reactivation condition: Reopen if the workflow creates a second authority source, encourages one broad implementation PR, makes phase specs replace Markdown authority, selects JSONL as the primary phase spec format, or starts validator/tooling before the spec shape is accepted.

Supersedes: none.

Superseded by: none.

## Lifecycle rule

Use `../workflow-lifecycle.md` for state transitions. New entries should include `State transition` when the decision changes lifecycle state.

## Decision shape

```text
Date:
Decision:
State transition:
Context:
Options considered:
Reason:
Affected planning files:
Evidence:
Follow-up:
Reactivation condition:
Supersedes:
Superseded by:
```
