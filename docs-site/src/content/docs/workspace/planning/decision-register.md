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
  - ../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-009-closeout.md
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

Evidence: Implementation covered editable-text vocabulary, descriptor wiring, package validation, InspectorField text-editing lowering, catalog and inspection projection, normalized edit/composition/selection facts, runtime proof-frame evidence, static mount validation, route-guard evidence, focused tests, and local validation on 2026-07-02. PR #46 merged into `main` at `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`.

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

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, `../start-here.md`, `../authority-model.md`, `../planning/README.md`, `../routines/README.md`, `../task-cards/README.md`, `../routines/track-orchestration-routine.md`, `../task-cards/track-manager-task.md`, `../specs/README.md`, `../specs/phase-implementation-spec.md`, and `../specs/templates/phase-implementation-spec.ron`.

Evidence: `AGENTS.md`, root architecture/domain/testing summaries, workspace operating model, authority model, lifecycle, implementation routine, PR review routine, phase completion drift check, roadmap update routine, planning records, PR #76 merge state, accepted runtime-platform cutover docs, and the completion/planning alignment updates in roadmap, production-tracks, completed-work, active-work, and decision-register.

Follow-up: Fulfilled by merged PR #77 and closeout truth. Open `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` as a separate active-implementation PR using the new track orchestration routine and phase spec handoff rules.

Reactivation condition: Reopen if the workflow creates a second authority source, encourages one broad implementation PR, makes phase specs replace Markdown authority, selects JSONL as the primary phase spec format, starts validator/tooling before the spec shape is accepted, or omits implementation-authorization fields needed for phase handoff.

Supersedes: none.

Superseded by: Track orchestration closeout and Phase 003 activation decision.

## Track orchestration closeout and Phase 003 activation decision

Date: 2026-07-07

Decision: Mark `PT-WORKFLOW-TRACK-ORCHESTRATION-001` completed through merged PR #77 and open `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` as the only active runtime-platform implementation phase.

State transition: `PT-WORKFLOW-TRACK-ORCHESTRATION-001 review -> completed`; `PT-UI-RUNTIME-PLATFORM-003 production-track -> active-implementation`.

Context: Current `main` contains merge commit `8b7a6b558bef79303e66d6a9f329dc71e00a0931` for PR #77, but planning still described the workflow gate as the active blocker before runtime implementation. The track orchestration routine and workflow lifecycle require completion truth before the next implementation phase starts.

Options considered: start Phase 003 runtime code immediately from stale planning; stop with the conflict only; record workflow-gate completion truth and activate Phase 003 with exact scope before implementation.

Reason: The correct long-term path is to fix planning truth first, then create one bounded Phase 003 implementation PR. Phase 003 is authorized only for the UiPlugin foundation shell: `engine::plugins::ui` module root, plugin install/build behavior, schedule labels, default resources, report shell, diagnostics shell, export wiring, and focused engine tests. Public mounting, typed screen/source/action contracts, sessions, host dispatch, trace, render publication, scene/debug migration, Counter product, reload/persistence, SDF/world-space work, `foundation/meta`, `domain/app_program`, generic plugin framework, validator tooling, and later phases remain blocked.

Affected files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, `../../reports/closeouts/pt-workflow-track-orchestration-001-closeout.md`, and `tools/docs/validate_docs.py`.

Evidence: `E3` source/design/planning/tooling inspection by path, `E5` local docs/diff/status validation for this planning-closeout PR when recorded by the PR report, and `E8` accepted architecture/workflow/planning authority. PR #77 merge evidence is commit `8b7a6b558bef79303e66d6a9f329dc71e00a0931`. The docs validator stale-pattern update aligns optional helper behavior with accepted runtime-platform docs; it does not create workflow authority.

Follow-up: Fulfilled by completed `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` through PR #79 and closeout truth. Use Phase 004 planning as the next runtime-platform state.

Reactivation condition: Reopen if Phase 003 needs scope outside the authorized foundation shell, if planning drifts from merged branch truth again, if the workflow gate closeout evidence is found inaccurate, or if Phase 003 implementation tries to include Phase 004 or later work.

Supersedes: none.

Superseded by: UiPlugin Foundation completion and Phase 004 planning decision.

## UiPlugin Foundation completion and Phase 004 planning decision

Date: 2026-07-07

Decision: Mark `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` completed through merged PR #79 and open `PT-UI-RUNTIME-PLATFORM-004 — App Mounting API` as active planning only.

State transition: `PT-UI-RUNTIME-PLATFORM-003 review -> completed`; `PT-UI-RUNTIME-PLATFORM-004 production-track -> active-planning`.

Context: PR #79 merged the bounded Phase 003 implementation into `main` at `0135850277e904b4be2c336e3ef6507b3fc88b72`. The track orchestration routine, workflow lifecycle, and phase completion drift check require completion truth before the next implementation phase starts.

Options considered: leave Phase 003 as active implementation after merge; start Phase 004 implementation immediately from the cutover plan; record Phase 003 completion truth and open Phase 004 as planning only.

Reason: Phase 003 delivered exactly the authorized UiPlugin foundation shell and preserved all later-phase boundaries. The correct next state is to record completion truth, then prepare Phase 004 separately. The phase completion drift check does not authorize Phase 004 implementation from the closeout patch.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, and `../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md`.

Evidence: `E3` source/test inspection by path, `E5` local command validation on PR #79 head, `E6` PR #79 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment. PR #79 had no unresolved comments, reviews, review requests, or hosted checks when inspected before merge.

Follow-up: Fulfilled by the Phase 004 App Mounting API activation decision below.

Reactivation condition: Reopen if PR #79 completion evidence is found inaccurate, if Phase 003 introduced forbidden Phase 004-014 scope, if Phase 004 planning needs authority beyond the accepted cutover plan, or if planning drifts from merged code again.

Supersedes: none.

Superseded by: Phase 004 App Mounting API activation decision.

## Phase 004 App Mounting API activation decision

Date: 2026-07-07

Decision: Authorize `PT-UI-RUNTIME-PLATFORM-004 — App Mounting API` as exactly one bounded active-implementation phase after Phase 003 completion truth merged.

State transition: `PT-UI-RUNTIME-PLATFORM-004 active-planning -> active-implementation`.

Context: PR #80 merged Phase 003 completion truth into `main` at `1a8803d06f1ed693c55101fa7211e8a665d1a2cd`. Active work now has the accepted Phase 004 owner, handoff contract, allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition map, and stop conditions.

Options considered: leave Phase 004 in active planning only; start Phase 004 implementation without updating planning; authorize one bounded Phase 004 implementation PR.

Reason: The track orchestration workflow allows the next implementation phase only after the previous phase is reviewed, merged, and closeout truth is recorded. That condition is now satisfied for Phase 003. The accepted cutover plan and active-work record provide enough exact scope to authorize Phase 004 implementation without widening into Phase 005 or later.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E6` PR #80 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 003 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md`.

Follow-up: Fulfilled by the Phase 004 App Mounting API completion and Phase 005 planning decision below.

Reactivation condition: Reopen if Phase 004 implementation needs scope outside the accepted App Mounting API contract, if private App internals are required without accepted API adjustment, if typed screen/source/action contracts enter the PR, or if planning drifts from merged code again.

Supersedes: none.

Superseded by: Phase 004 App Mounting API completion and Phase 005 planning decision.

## Phase 004 App Mounting API completion and Phase 005 planning decision

Date: 2026-07-07

Decision: Mark `PT-UI-RUNTIME-PLATFORM-004 — App Mounting API` completed through merged PR #82 and open `PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` as active planning only.

State transition: `PT-UI-RUNTIME-PLATFORM-004 review -> completed`; `PT-UI-RUNTIME-PLATFORM-005 production-track -> active-planning`.

Context: PR #82 merged the bounded Phase 004 implementation into `main` at `9fb86f0d426385be7e425ff943c7a9d5450e1edb`. The track orchestration routine, workflow lifecycle, and phase completion drift check require completion truth before the next implementation phase starts.

Options considered: leave Phase 004 as active implementation after merge; start Phase 005 implementation immediately from the cutover plan; record Phase 004 completion truth and open Phase 005 as planning only.

Reason: Phase 004 delivered the authorized App Mounting API while preserving typed screen/source/action contracts and all later runtime scope for downstream phases. The correct next state is to record completion truth, then prepare Phase 005 separately. The phase completion drift check does not authorize Phase 005 implementation from the closeout patch.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, and `../../reports/closeouts/pt-ui-runtime-platform-004-closeout.md`.

Evidence: `E3` source/test inspection by path, `E5` local command validation on PR #82 head, `E6` PR #82 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment. PR #82 had no unresolved comments, reviews, review requests, or hosted checks when inspected before merge.

Follow-up: Fulfilled by the Phase 005 Typed Screen / Source / Action Contracts activation decision below.

Reactivation condition: Reopen if PR #82 completion evidence is found inaccurate, if Phase 004 introduced forbidden Phase 005-014 scope, if Phase 005 planning needs authority beyond the accepted cutover plan, or if planning drifts from merged code again.

Supersedes: Phase 004 App Mounting API activation decision.

Superseded by: Phase 005 Typed Screen / Source / Action Contracts activation decision.

## Phase 005 Typed Screen / Source / Action Contracts activation decision

Date: 2026-07-07

Decision: Authorize `PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` as exactly one bounded active-implementation phase after Phase 004 completion truth merged.

State transition: `PT-UI-RUNTIME-PLATFORM-005 active-planning -> active-implementation`.

Context: PR #83 merged Phase 004 completion truth into `main` at `8b6f3074b7e380c51fa4fea7923e4c9409dab24f`. Active work now has the accepted Phase 005 owner, handoff contract, allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition map, and stop conditions.

Options considered: leave Phase 005 in active planning only; start Phase 005 implementation without updating planning; authorize one bounded Phase 005 implementation PR.

Reason: The track orchestration workflow allows the next implementation phase only after the previous phase is reviewed, merged, and closeout truth is recorded. That condition is now satisfied for Phase 004. The accepted cutover plan and active-work record provide enough exact scope to authorize Phase 005 implementation without widening into Phase 006 or later.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E6` PR #83 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 004 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-004-closeout.md`.

Follow-up: Open exactly one bounded `PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` implementation PR from current `main`. Keep the PR draft until focused Phase 005 validation and required docs/diff/status commands are clean.

Reactivation condition: Reopen if Phase 005 implementation needs scope outside the accepted typed screen/source/action contract, if source/program facts are skipped, if generic controls mutate app state directly, if mounted session/runtime trace/render publication enters the PR, or if planning drifts from merged code again.

Supersedes: Phase 004 App Mounting API completion and Phase 005 planning decision.

Superseded by: Phase 005 Typed Screen / Source / Action Contracts completion and Phase 006 planning decision.

## Phase 005 Typed Screen / Source / Action Contracts completion and Phase 006 planning decision

Date: 2026-07-07

Decision: Mark `PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` completed through merged PR #85 and open `PT-UI-RUNTIME-PLATFORM-006 — Mounted Surface Session Runtime` as active planning only.

State transition: `PT-UI-RUNTIME-PLATFORM-005 review -> completed`; `PT-UI-RUNTIME-PLATFORM-006 production-track -> active-planning`.

Context: PR #85 merged the bounded Phase 005 implementation into `main` at `6226470defa7a72a567fc03c1bc3783e63e2c2c8`. The track orchestration routine, workflow lifecycle, and phase completion drift check require completion truth before the next implementation phase starts.

Options considered: leave Phase 005 as active implementation after merge; start Phase 006 implementation immediately from the cutover plan; record Phase 005 completion truth and open Phase 006 as planning only.

Reason: Phase 005 delivered the authorized Typed Screen / Source / Action Contracts while preserving mounted sessions and all later runtime scope for downstream phases. The correct next state is to record completion truth, then prepare Phase 006 separately. The phase completion drift check does not authorize Phase 006 implementation from the closeout patch.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, and `../../reports/closeouts/pt-ui-runtime-platform-005-closeout.md`.

Evidence: `E3` source/test inspection by path, `E5` local command validation on PR #85 head, `E6` PR #85 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment. PR #85 had no unresolved comments, reviews, review requests, or hosted checks when inspected before merge.

Follow-up: Open a separate Phase 006 activation decision after this closeout/planning truth merges. Do not start Phase 006 implementation until active implementation is separately authorized.

Reactivation condition: Reopen if PR #85 completion evidence is found inaccurate, if Phase 005 introduced forbidden Phase 006-014 scope, if Phase 006 planning needs authority beyond the accepted cutover plan, or if planning drifts from merged code again.

Supersedes: Phase 005 Typed Screen / Source / Action Contracts activation decision.

Superseded by: Phase 006 Mounted Surface Session Runtime activation decision.

## Phase 006 Mounted Surface Session Runtime activation decision

Date: 2026-07-07

Decision: Authorize `PT-UI-RUNTIME-PLATFORM-006 — Mounted Surface Session Runtime` as exactly one bounded active-implementation phase after Phase 005 completion truth merged.

State transition: `PT-UI-RUNTIME-PLATFORM-006 active-planning -> active-implementation`.

Context: PR #86 merged Phase 005 completion truth into `main` at `cd18029d7a8943af114b6eabe5f4ebc82d537249`. Active work now has the accepted Phase 006 owner, handoff contract, allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition map, and stop conditions.

Options considered: leave Phase 006 in active planning only; start Phase 006 implementation without updating planning; authorize one bounded Phase 006 implementation PR.

Reason: The track orchestration workflow allows the next implementation phase only after the previous phase is reviewed, merged, and closeout truth is recorded. That condition is now satisfied for Phase 005. The accepted cutover plan and active-work record provide enough exact scope to authorize Phase 006 implementation without widening into Phase 007 or later.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E6` PR #86 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 005 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-005-closeout.md`.

Follow-up: Open exactly one bounded `PT-UI-RUNTIME-PLATFORM-006 — Mounted Surface Session Runtime` implementation PR from current `main`. Keep the PR draft until focused Phase 006 validation and required docs/diff/status commands are clean.

Reactivation condition: Reopen if Phase 006 implementation needs scope outside the accepted mounted surface session runtime contract, if engine duplicates `ui_surface` semantics, if world-space UI/SDF/SpatialCanvas enters the PR, if host action dispatch/runtime trace/render publication enters the PR, or if planning drifts from merged code again.

Supersedes: Phase 005 Typed Screen / Source / Action Contracts completion and Phase 006 planning decision.

Superseded by: Phase 006 Mounted Surface Session Runtime completion and Phase 007 planning decision.

## Phase 006 Mounted Surface Session Runtime completion and Phase 007 planning decision

Date: 2026-07-07

Decision: Mark `PT-UI-RUNTIME-PLATFORM-006 — Mounted Surface Session Runtime` completed through merged PR #88 and open `PT-UI-RUNTIME-PLATFORM-007 — Host Action Dispatch and Runtime Trace` as active planning only.

State transition: `PT-UI-RUNTIME-PLATFORM-006 review -> completed`; `PT-UI-RUNTIME-PLATFORM-007 production-track -> active-planning`.

Context: PR #88 merged the bounded Phase 006 implementation into `main` at `82d6f00326cf2823eb91d3f655a730b962b355f6`. The track orchestration routine, workflow lifecycle, and phase completion drift check require completion truth before the next implementation phase starts.

Options considered: leave Phase 006 as active implementation after merge; start Phase 007 implementation immediately from the cutover plan; record Phase 006 completion truth and open Phase 007 as planning only.

Reason: Phase 006 delivered the authorized mounted surface/session runtime while preserving host action dispatch, runtime trace, and all later runtime scope for downstream phases. The correct next state is to record completion truth, then prepare Phase 007 separately. The phase completion drift check does not authorize Phase 007 implementation from the closeout patch.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, and `../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md`.

Evidence: `E3` source/test inspection by path, `E5` local command validation on PR #88 head, `E6` PR #88 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment. PR #88 had no unresolved comments, reviews, review requests, or hosted checks when inspected before merge.

Follow-up: Open a separate Phase 007 active-implementation decision after this closeout/planning truth merges. Do not start Phase 007 implementation until active implementation is separately authorized.

Reactivation condition: Reopen if PR #88 completion evidence is found inaccurate, if Phase 006 introduced forbidden Phase 007-014 scope, if Phase 007 planning needs authority beyond the accepted cutover plan, or if planning drifts from merged code again.

Supersedes: Phase 006 Mounted Surface Session Runtime activation decision.

Superseded by: Phase 007 Host Action Dispatch and Runtime Trace activation decision.

## Phase 007 Host Action Dispatch and Runtime Trace activation decision

Date: 2026-07-07

Decision: Authorize `PT-UI-RUNTIME-PLATFORM-007 — Host Action Dispatch and Runtime Trace` as exactly one bounded active-implementation phase after Phase 006 completion truth merged.

State transition: `PT-UI-RUNTIME-PLATFORM-007 active-planning -> active-implementation`.

Context: PR #89 merged Phase 006 completion truth into `main` at `0dc7b5337126697d4a445adfa687b30559c44d59`. Active work now has the accepted Phase 007 owner, handoff contract, allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition map, and stop conditions.

Options considered: leave Phase 007 in active planning only; start Phase 007 implementation without updating planning; authorize one bounded Phase 007 implementation PR.

Reason: The track orchestration workflow allows the next implementation phase only after the previous phase is reviewed, merged, and closeout truth is recorded. That condition is now satisfied for Phase 006. The accepted cutover plan and active-work record provide enough exact scope to authorize Phase 007 implementation without widening into Phase 008 or later.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E6` PR #89 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 006 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md`.

Follow-up: Open exactly one bounded `PT-UI-RUNTIME-PLATFORM-007 — Host Action Dispatch and Runtime Trace` implementation PR from current `main`. Keep the PR draft until focused Phase 007 validation and required docs/diff/status commands are clean.

Reactivation condition: Reopen if Phase 007 implementation needs scope outside the accepted host action dispatch/runtime trace contract, if invalid actions can partially mutate host state, if trace records become product-specific or engine-wide framework behavior, if runtime evaluation/render publication enters the PR, or if planning drifts from merged code again.

Supersedes: Phase 006 Mounted Surface Session Runtime completion and Phase 007 planning decision.

Superseded by: Phase 007 Host Action Dispatch and Runtime Trace completion and Phase 008 planning decision.

## Phase 007 Host Action Dispatch and Runtime Trace completion and Phase 008 planning decision

Date: 2026-07-07

Decision: Mark `PT-UI-RUNTIME-PLATFORM-007 — Host Action Dispatch and Runtime Trace` completed through merged PR #91 and open `PT-UI-RUNTIME-PLATFORM-008 — Runtime Evaluation, State Snapshot, and Invalidation` as active planning only.

State transition: `PT-UI-RUNTIME-PLATFORM-007 review -> completed`; `PT-UI-RUNTIME-PLATFORM-008 production-track -> active-planning`.

Context: PR #91 merged the bounded Phase 007 implementation into `main` at `5dd90a2caf1bb7e4d5710830499df1d122fe587f`. The track orchestration routine, workflow lifecycle, and phase completion drift check require completion truth before the next implementation phase starts.

Options considered: leave Phase 007 as active implementation after merge; start Phase 008 implementation immediately from the cutover plan; record Phase 007 completion truth and open Phase 008 as planning only.

Reason: Phase 007 delivered the authorized host action dispatch and generic UI-runtime trace path while preserving runtime evaluation, state snapshot, invalidation, and all later runtime scope for downstream phases. The correct next state is to record completion truth, then prepare Phase 008 separately. The phase completion drift check does not authorize Phase 008 implementation from the closeout patch.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E5` local command validation recorded for PR #91, `E6` PR #91 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 007 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-007-closeout.md`.

Follow-up: Open a separate Phase 008 active-implementation decision after this closeout/planning truth merges. Do not start Phase 008 implementation until active implementation is separately authorized.

Reactivation condition: Reopen if PR #91 completion evidence is found inaccurate, if Phase 007 introduced forbidden Phase 008-014 scope, if Phase 008 planning needs authority beyond the accepted cutover plan, or if planning drifts from merged code again.

Supersedes: Phase 007 Host Action Dispatch and Runtime Trace activation decision.

Superseded by: Phase 008 Runtime Evaluation, State Snapshot, and Invalidation activation decision.

## Phase 008 Runtime Evaluation, State Snapshot, and Invalidation activation decision

Date: 2026-07-07

Decision: Authorize `PT-UI-RUNTIME-PLATFORM-008 — Runtime Evaluation, State Snapshot, and Invalidation` as exactly one bounded active-implementation phase after Phase 007 completion truth merged.

State transition: `PT-UI-RUNTIME-PLATFORM-008 active-planning -> active-implementation`.

Context: PR #92 merged Phase 007 completion truth into `main` at `7f83c02075d976ad6eb19e82408aa596cce1024f`. Active work now has the accepted Phase 008 owner, handoff contract, allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition map, and stop conditions. Activation-time source inspection confirmed that `engine` already depends on `ui_runtime` and `ui_render_data`, and that the bounded implementation may add only `ui_artifacts`, `ui_binding`, `ui_evaluator`, `ui_runtime_view`, and `ui_state` if needed for the evaluator/runtime-view path.

Options considered: leave Phase 008 in active planning only; start Phase 008 implementation without updating planning; authorize one bounded Phase 008 implementation PR.

Reason: The track orchestration workflow allows the next implementation phase only after the previous phase is reviewed, merged, and closeout truth is recorded. That condition is now satisfied for Phase 007. The accepted cutover plan and source inspection provide enough exact scope to authorize Phase 008 implementation without widening into Phase 009 or later.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E6` PR #92 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 007 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-007-closeout.md`.

Follow-up: Open exactly one bounded `PT-UI-RUNTIME-PLATFORM-008 — Runtime Evaluation, State Snapshot, and Invalidation` implementation PR from current `main`. Keep the PR draft until focused Phase 008 validation and required docs/diff/status commands are clean.

Reactivation condition: Reopen if Phase 008 implementation needs scope outside the accepted runtime evaluation/snapshot/invalidation contract, if frame output skips source/program/evaluator evidence, if renderer primitives become UI source truth, if render publication enters the PR, or if planning drifts from merged code again.

Supersedes: Phase 007 Host Action Dispatch and Runtime Trace completion and Phase 008 planning decision.

Superseded by: Phase 008 Runtime Evaluation, State Snapshot, and Invalidation completion and Phase 009 planning decision.

## Phase 008 Runtime Evaluation, State Snapshot, and Invalidation completion and Phase 009 planning decision

Date: 2026-07-07

Decision: Mark `PT-UI-RUNTIME-PLATFORM-008 — Runtime Evaluation, State Snapshot, and Invalidation` completed through merged PR #94 and open `PT-UI-RUNTIME-PLATFORM-009 — SurfaceFrame Generic Producer Boundary` as active planning only.

State transition: `PT-UI-RUNTIME-PLATFORM-008 review -> completed`; `PT-UI-RUNTIME-PLATFORM-009 production-track -> active-planning`.

Context: PR #94 merged the bounded Phase 008 implementation into `main` at `be5b790e38b7f80ad17092fa0cb75e87eef4d849`. The track orchestration routine, workflow lifecycle, and phase completion drift check require completion truth before the next implementation phase starts.

Options considered: leave Phase 008 as active implementation after merge; start Phase 009 implementation immediately from the cutover plan; record Phase 008 completion truth and open Phase 009 as planning only.

Reason: Phase 008 delivered the authorized runtime evaluation, state snapshot, invalidation, diagnostics, reporting, trace, and focused engine tests while preserving SurfaceFrame generic producer boundary, UiPlugin render publication, scene/debug overlay migration, source reload/persistence, and Counter product scope for downstream phases. The correct next state is to record completion truth, then prepare Phase 009 separately. The phase completion drift check does not authorize Phase 009 implementation from the closeout patch.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E5` local command validation recorded for PR #94, `E6` PR #94 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 008 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-008-closeout.md`.

Follow-up: Open a separate Phase 009 active-implementation decision after this closeout/planning truth merges. Do not start Phase 009 implementation until active implementation is separately authorized.

Reactivation condition: Reopen if PR #94 completion evidence is found inaccurate, if Phase 008 introduced forbidden Phase 009-014 scope, if Phase 009 planning needs authority beyond the accepted cutover plan, or if planning drifts from merged code again.

Supersedes: Phase 008 Runtime Evaluation, State Snapshot, and Invalidation activation decision.

Superseded by: Phase 009 SurfaceFrame Generic Producer Boundary activation decision.

## Phase 009 SurfaceFrame Generic Producer Boundary activation decision

Date: 2026-07-07

Decision: Authorize `PT-UI-RUNTIME-PLATFORM-009 — SurfaceFrame Generic Producer Boundary` as exactly one bounded active-implementation phase after Phase 008 completion truth merged.

State transition: `PT-UI-RUNTIME-PLATFORM-009 active-planning -> active-implementation`.

Context: PR #95 merged Phase 008 completion truth into `main` at `f83d9eb6536749cd80d9df05b18a182380aac116`. Active work now has the accepted Phase 009 owner, handoff contract, activation-time migration map, allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition map, and stop conditions. Activation-time source inspection confirmed that the current producer-facing seam is UI-named through `UiFrameProducerId`, `UiFrameSubmission`, `UiFrameSubmissionRegistryResource`, and `PreparedUiFrameSubmission`, while the renderer already has generic `RenderFrameProducerId`, `RenderSurfaceId`, and prepared render-frame packets.

Options considered: leave Phase 009 in active planning only; start Phase 009 implementation without updating planning; authorize one bounded Phase 009 implementation PR.

Reason: The track orchestration workflow allows the next implementation phase only after the previous phase is reviewed, merged, and closeout truth is recorded. That condition is now satisfied for Phase 008. The accepted cutover plan and source inspection provide enough exact scope to authorize Phase 009 implementation without widening into UiPlugin render publication, scene/debug overlay migration, or later phases.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E6` PR #95 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 008 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-008-closeout.md`.

Follow-up: Open exactly one bounded `PT-UI-RUNTIME-PLATFORM-009 — SurfaceFrame Generic Producer Boundary` implementation PR from current `main`. Keep the PR draft until focused Phase 009 validation and required docs/diff/status commands are clean.

Reactivation condition: Reopen if Phase 009 implementation needs scope outside the accepted producer-generic SurfaceFrame boundary migration map, if UiPlugin render publication enters the PR, if scene/debug overlay migration or retirement enters the PR, if genericization creates a second runtime path, if compatibility shims become the durable public API instead of the generic seam, or if planning drifts from merged code again.

Supersedes: Phase 008 Runtime Evaluation, State Snapshot, and Invalidation completion and Phase 009 planning decision.

Superseded by: Phase 009 SurfaceFrame Generic Producer Boundary completion and Phase 010 planning decision.

## Phase 009 SurfaceFrame Generic Producer Boundary completion and Phase 010 planning decision

Date: 2026-07-07

Decision: Mark `PT-UI-RUNTIME-PLATFORM-009 — SurfaceFrame Generic Producer Boundary` completed through merged PR #97 and open `PT-UI-RUNTIME-PLATFORM-010 — UiPlugin Render Publication` as active planning only.

State transition: `PT-UI-RUNTIME-PLATFORM-009 review -> completed`; `PT-UI-RUNTIME-PLATFORM-010 production-track -> active-planning`.

Context: PR #97 merged Phase 009 into `main` at `50e2dbdf1f9c076f4a76a04543274801d1f1649b` after focused migration validation, engine validation, docs validation, diff hygiene, PR metadata inspection, and review-thread inspection. The Phase 009 closeout records that `RenderFrameProducerId`, `SurfaceFrameSubmission`, `SurfaceFrameSubmissionRegistryResource`, `PreparedSurfaceFrameSubmission`, and `SurfaceFrameSubmissionRenderOutputProof` now own the accepted producer-generic seam, while UiPlugin render publication remains unimplemented.

Options considered: leave Phase 009 active after merge; mark Phase 009 complete and immediately authorize Phase 010 implementation; mark Phase 009 complete and open Phase 010 active planning only.

Reason: The track orchestration workflow requires completion truth after each implementation PR before the next implementation phase starts. Phase 010 needs its own source inspection, exact allowed/forbidden file list, focused validation envelope, evidence requirements, principle review, and stop conditions before implementation authorization. Opening active planning only preserves the one-phase-per-PR boundary.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 009, `E6` PR #97 merge metadata and no-checks metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment in `../../reports/closeouts/pt-ui-runtime-platform-009-closeout.md`.

Follow-up: Open a separate `PT-UI-RUNTIME-PLATFORM-010 — UiPlugin Render Publication` active-implementation authorization PR from current `main` if authority and source inspection still agree. Do not start Phase 010 implementation until that activation PR is merged.

Reactivation condition: Reopen if PR #97 completion evidence is found inaccurate, if Phase 009 introduced forbidden Phase 010-014 scope, if Phase 010 planning needs authority beyond the accepted cutover plan, or if planning drifts from merged code again.

Supersedes: Phase 009 SurfaceFrame Generic Producer Boundary activation decision.

Superseded by: Phase 010 UiPlugin Render Publication activation decision.

## Phase 010 UiPlugin Render Publication activation decision

Date: 2026-07-07

Decision: Authorize `PT-UI-RUNTIME-PLATFORM-010 — UiPlugin Render Publication` as exactly one bounded active-implementation phase after Phase 009 completion truth merged.

State transition: `PT-UI-RUNTIME-PLATFORM-010 active-planning -> active-implementation`.

Context: PR #98 merged Phase 009 completion truth into `main` at `1b29eb58cdbea4d3c351403702373d013772d541`. Active work now has the accepted Phase 010 owner, handoff contract, activation-time implementation map, allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition map, and stop conditions. Activation-time source inspection confirmed the current UiPlugin runtime evaluation path, frame payload facts, trace resource, generic `SurfaceFrameSubmissionRegistryResource`, `RenderFrameProducerId`, `RenderSurfaceId`, and focused UI/render test coverage that Phase 010 must extend.

Options considered: leave Phase 010 in active planning only; start Phase 010 implementation without updating planning; authorize one bounded Phase 010 implementation PR.

Reason: The track orchestration workflow allows the next implementation phase only after the previous phase is reviewed, merged, and closeout truth is recorded. That condition is now satisfied for Phase 009. The accepted cutover plan, architecture trace requirements, and source inspection provide enough exact scope to authorize Phase 010 implementation without widening into scene/debug overlay migration, Counter app packaging, source reload/persistence, or later phases.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: `E3` source/design/planning inspection by path, `E6` PR #98 merge metadata, `E8` accepted architecture/workflow/planning authority, and Phase 009 closeout evidence in `../../reports/closeouts/pt-ui-runtime-platform-009-closeout.md`.

Follow-up: Open exactly one bounded `PT-UI-RUNTIME-PLATFORM-010 — UiPlugin Render Publication` implementation PR from current `main`. Keep the PR draft until focused Phase 010 validation and required docs/diff/status commands are clean.

Reactivation condition: Reopen if Phase 010 implementation needs scope outside the accepted UiPlugin render publication map, if RenderPlugin becomes UI runtime owner, if scene/debug overlay migration or retirement enters the PR, if Counter product packaging enters the PR, if source/program/action semantics change, if publication cannot map to an explicit render surface, or if planning drifts from merged code again.

Supersedes: Phase 009 SurfaceFrame Generic Producer Boundary completion and Phase 010 planning decision.

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
