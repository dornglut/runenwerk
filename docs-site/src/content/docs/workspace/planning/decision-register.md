---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-06
related_docs:
  - ../workflow-lifecycle.md
  - ./ecs-backed-counter-ui-story-proof-planning.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
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

Evidence: PR #44 merged into `main` at merge commit `6f2d3827f315191d7aeaf68a64f523627197cad8`; local validation passed on 2026-07-02; package-backed overlay declarations, runtime overlay proof, proof-frame projection, static mount proof, and no-bypass evidence are present.

Follow-up: Keep Phase 13 as completed dependency.

## Phase 14 text editing planning decision

Date: 2026-07-02

Decision: Start `PT-UI-COMPONENT-PLATFORM-014` as Text Editing / Editable Text Behavior design/planning intake.

State transition: `production-track -> active-planning`

Evidence: Phase 13 completed overlay/layering, and Phase 14 planning scope required package-backed editable text behavior without moving product/editor/game ownership into generic UI.

Follow-up: Review and refine the Text Editing design before implementation.

## Phase 14 implementation and review readiness decision

Date: 2026-07-02

Decision: Promote `PT-UI-COMPONENT-PLATFORM-014` from planning to local implementation, then move the branch to review after package-backed implementation evidence was added locally.

State transition: `active-planning -> active-implementation -> review`

Evidence: Local implementation evidence covered editable-text vocabulary, descriptor wiring, package validation, InspectorField lowering, catalog and inspection projection, normalized edit/composition/selection facts, runtime proof-frame evidence, static mount validation, no-bypass evidence, and focused tests. Local validation passed on 2026-07-02 with the recorded Phase 14 cargo/docs/diff gate.

Follow-up: After acceptance or merge, record Phase 14 completion truth and open Phase 15.

## Phase 14 completion decision

Date: 2026-07-02

Decision: Mark `PT-UI-COMPONENT-PLATFORM-014` Text Editing / Editable Text Behavior completed through merged PR #46.

State transition: `review -> completed`

Evidence: PR #46 merged into `main` at merge commit `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`. Local Phase 14 validation passed on 2026-07-02 before merge.

Follow-up: Keep Phase 14 as completed dependency and use it as the preceding substrate for Generic Text planning.

## Phase 15 generic text planning-start decision

Date: 2026-07-02

Decision: Start `PT-UI-COMPONENT-PLATFORM-015` as Generic Text design/planning intake.

State transition: `production-track -> active-planning`

Evidence: Existing `ui_text` owned renderer-independent text contracts, and the Phase 15 scope was bounded to generic text display/layout, package/catalog/inspection projection, visual proof, and static mount proof. Product/editor/game mutation, dynamic plugin framework, `foundation/meta`, shared plugin primitives, UI Designer, UI Gallery product surface, Workbench/provider redesign, authored UI editing, and compatibility-only aliases/shims remained out of scope.

Follow-up: Review and refine the Generic Text design before implementation.

## Phase 15 generic text implementation closeout decision

Date: 2026-07-02

Decision: Move `PT-UI-COMPONENT-PLATFORM-015` Generic Text out of active planning after local implementation closeout evidence passed on PR #48.

State transition: `active-planning -> active-implementation -> review`

Evidence: PR #48 branch `ui/generic-text-phase-15` at implementation commit `32e402b108d1e72d7cc5b4113af29d8d29626680` implemented the renderer-neutral Generic Text substrate across `ui_text`, `ui_render_data`, `ui_controls`, `ui_runtime`, and `ui_static_mount`. Local validation passed on 2026-07-02 with the focused package, workspace, docs, and diff gate.

Follow-up: Complete PR #48 merge, then record the merge commit.

## Phase 15 generic text baseline completion decision

Date: 2026-07-02

Decision: Mark `PT-UI-COMPONENT-PLATFORM-015` Generic Text baseline completed through merged PR #48.

State transition: `review -> completed`

Evidence: PR #48 merged into `main` at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`. The validated implementation commit passed the local Phase 15 package, workspace, docs, and diff gate.

Follow-up: Keep Phase 15 as completed dependency and perform the hardening/future-proofing review before opening Phase 16 implementation.

## Phase 15 generic text hardening completion decision

Date: 2026-07-02

Decision: Mark the Phase 15 Generic Text hardening pass completed through merged PR #49 without starting Phase 16 implementation.

State transition: `completed -> completed-hardening`

Evidence: PR #49 merged into `main` at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9` and completed the recorded Generic Text hardening pass. Final Phase 15 validation passed on 2026-07-02 with the full package, workspace, docs, and diff gate.

Follow-up: Use PR #48 plus PR #49 as the authoritative Phase 15 completion evidence. Open Phase 16 Surface2D as planning/design hardening only.

## Phase 16 Surface2D planning-start decision

Date: 2026-07-02

Decision: Start `PT-UI-COMPONENT-PLATFORM-016` as Surface2D design/planning intake after Phase 15 baseline and hardening evidence were recorded.

State transition: `production-track -> active-planning`

Evidence: Phase 15 Generic Text completed through PR #48 and PR #49. Surface2D planning scope covered renderer-neutral 2D surface identity, content/viewport bounds, transforms, pan/zoom/fit, selection rectangle, hover coordinate, pointer capture, gesture cancel/commit, overlay/diagnostic layers, grid/background vocabulary, large-content bounds, LOD readiness, and budget evidence.

Follow-up: Harden the Surface2D design before implementation.

## Phase 16 Surface2D completion decision

Date: 2026-07-03

Decision: Mark `PT-UI-COMPONENT-PLATFORM-016` Surface2D completed through docs-hardening PR #62 and implementation PR #61.

State transition: `review -> completed`

Context: PR #62 merged generic workflow, principle, decomposition, and merge-readiness hardening before Phase 16 completion. PR #61 then squash-merged the Surface2D implementation into `main`.

Options considered: mark Phase 16 completed; leave Phase 16 in review; reopen implementation for the leftover `surface2d-phase-16` branch.

Reason: `main` contains the delivered Surface2D package/catalog/inspection contract, runtime proof report/frame, and static mount proof. Post-merge validation from `main` passed, and no Phase 16 product blocker remains.

Affected planning files: `active-work.md`, `completed-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `ui-component-platform-surface2d-design.md`, `phase-16-surface2d-source-investigation.md`, and `phase-16-surface2d-closeout.md`.

Evidence: PR #62 merged at `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`. PR #61 squash-merged at `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. Post-merge validation from `main` passed with the recorded focused Surface2D package/runtime/static-mount commands, `cargo test --workspace`, docs validation, and diff check.

Follow-up: Keep Phase 16 as a completed dependency and keep Phase 17 SpatialCanvas as future planning until separately authorized.

## Phase 16 leftover branch decision

Date: 2026-07-03

Decision: Keep remote branch `surface2d-phase-16` for manual review instead of deleting it during closeout.

State transition: none

Context: Cleanup inspection found `surface2d-phase-16` still had commits not patch-equivalent to `origin/main`.

Options considered: delete as obsolete; apply all changes as follow-up; keep for manual review.

Reason: The leftover branch modified only the Surface2D design document and mixed potentially useful future-use-case pressure with stale pre-implementation assumptions. It was not safe to fold into closeout truth.

Affected planning files: `phase-16-surface2d-closeout.md`, `active-work.md`, and `production-tracks.md`.

Evidence: Branch comparison showed three extra commits and one large design-document diff.

Follow-up: Review `surface2d-phase-16` manually outside the Phase 16 closeout. If useful material remains, extract a focused docs-only follow-up from `main`.

## UI framework app integration direction decision

Date: 2026-07-05

Decision: Replace the active next-work focus from `PT-UI-APP-PROGRAM-001` manual Typed App Program headless counter implementation to `PT-UI-FRAMEWORK-APP-INTEGRATION-001` UI Framework App Integration Direction Review.

State transition: `PT-UI-APP-PROGRAM-001 active-planning/implementation-spike -> superseded`; `PT-UI-FRAMEWORK-APP-INTEGRATION-001 -> active-planning`.

Context: Review of the UI architecture, UI story workflow, runtime rendering pipeline, UI Component Platform roadmap, Interaction V2 ADR, Typed App Program design, PR #69, and Phase 17 SpatialCanvas intake showed that the repository has strong UI substrate but lacks a canonical app-facing framework path. PR #69 followed its prior planning contract, but that contract started from a manual proof-IR layer rather than the real app-authoring question.

Options considered: merge PR #69 as the app framework foundation; continue raw ECS UI; make external templates the only immediate path; promote SpatialCanvas; select ECS/App/Plugin-hosted UI-definition-backed app integration.

Reason: The best long-term direction is App/ECS-hosted app authoring that lowers UI through `ui_definition`, FormedInteractionModel, `UiProgram`, runtime artifacts, and `UiStory` reports while keeping app mutation in host/app owners.

Affected planning files: `active-work.md`, `decision-register.md`, `typed-app-program-ui-proof-001-planning.md`, and `ui-framework-app-integration-direction-review.md`.

Evidence: Current UI architecture and UI Story workflow authority supported the source-to-mount pipeline; PR #69 was open/draft as a manual `app_program` proof crate; PR #65 was open/draft as SpatialCanvas planning-only.

Follow-up: Review and accept, revise, or reject `ui-framework-app-integration-direction-review.md`.

Reactivation condition: Reactivate `PT-UI-APP-PROGRAM-001` only if a later accepted design decides that a dedicated `app_program` crate is required as proof/report vocabulary and is not the public UI framework authoring path.

Supersedes: `PT-UI-APP-PROGRAM-001` as the prior app-framework foundation candidate.

Superseded by: `PT-UI-FRAMEWORK-APP-INTEGRATION-001`.

## ECS-backed Counter UI Story Proof implementation-planning decision

Date: 2026-07-05

Decision: Convert `PT-UI-FRAMEWORK-APP-INTEGRATION-002` from broad proof-planning intake into a conditional implementation contract for a small UI-owned `ui_app_integration` crate and an ECS-backed Counter UiStory-compatible proof.

State transition: `active-planning intake -> active-planning implementation contract`; implementation branch authorized only after the docs PR was reviewed/merged and the implementation stayed within the accepted contract.

Context: Source inspection showed the workspace already contained the relevant UI crates through `ui_story`; `ui_definition` owned authored template/node/slot structures; `ui_program` owned route/event packet contracts; `ui_program_lowering` owned the current source-to-UiProgram formation entrypoint; `ui_controls` owned package registry snapshots; `ui_story` was a V2 proof contract surface; `ui_testing` already depended on the proof/evaluation stack; and `engine::App` exposed resource/system methods while keeping world internals private.

Options considered: pure fixture/story-only proof; immediate public AppUiExt API; small UI-owned integration crate with internal/proof-local APIs first; revive `app_program`; implement SpatialCanvas first.

Reason: The best long-term path was internal proof first, then public API later: create a small UI-owned integration bridge, prove the Counter loop through internal/proof-local APIs, use `ui_story`/`ui_testing` only as proof harnesses, and defer public AppUiExt-style engine ergonomics until dependency direction and owner boundaries were validated.

Affected planning files: `active-work.md`, `decision-register.md`, and `ecs-backed-counter-ui-story-proof-planning.md`.

Evidence: `Cargo.toml` had no `ui_app_integration` member yet; the inspected UI crates provided the required source/program/route/control/story/proof contracts; `engine::App` world state was private, so the first public App extension was deferred.

Follow-up: After the docs PR was accepted, open an implementation branch that changes only the allowed files: root `Cargo.toml`, new `domain/ui/ui_app_integration` crate files, and focused counter proof tests.

Reactivation condition: Reopen the broader planning question only if implementing the exact contract requires engine core dependency inversion, public App extension API, `ui_definition`/`ui_program` dependency on ECS, `app_program` resurrection, or bypassing `ui_definition`, `UiProgram`, or story-compatible reports.

Supersedes: `PT-UI-FRAMEWORK-APP-INTEGRATION-002` broad A/B/C intake state.

Superseded by: none.

## ECS-backed Counter UI Story Proof closeout decision

Date: 2026-07-06

Decision: Mark `PT-UI-FRAMEWORK-APP-INTEGRATION-002` ECS-backed Counter UI Story Proof completed through merged PR #72 and the closeout report.

State transition: `review -> completed`

Context: PR #72 merged into `main`. Post-merge inspection found the implementation proof in `domain/ui/ui_app_integration` and planning truth still showed PR #72 closeout as the active blocker.

Options considered: keep PR #72 in review until a later runtime intake; mark the proof completed and start runtime implementation; mark the proof completed and move the active focus to PR #74 intake review only.

Reason: The delivered PR #72 scope matched the accepted planning contract for the bounded proof and preserved all recorded non-goals: no public `AppUiExt`, engine `UiPlugin`, render adapter, SDF/SpatialCanvas/world-space implementation, `foundation/meta`, `domain/app_program`, generic plugin framework, callback/direct mutation bypass, or ECS-owned durable UI semantic model.

Affected planning files: `active-work.md`, `completed-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, and closeout report `../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md`.

Evidence: GitHub reported PR #72 merged on 2026-07-06 at merge commit `e093eb1affdc465b96430200960f8e3cdca0d26b`. Source inspection covered the new crate, source modules, tests, and docs index link. Evidence classes: `E3`, `E5`, `E8`, and GitHub merge metadata.

Follow-up: Review and harden draft PR #74 / `PT-UI-RUNTIME-PLATFORM-001` intake. Do not start Live `UiPlugin` runtime implementation until complete investigation/design/planning evidence accepts an exact implementation contract.

Reactivation condition: Reopen `PT-UI-FRAMEWORK-APP-INTEGRATION-002` only if a future inspection finds the merged proof violates the recorded stop conditions.

Supersedes: the former pre-PR #75 closeout-pending state for PR #72.

Superseded by: none.

## Live UiPlugin runtime and surface-frame rendering intake decision

Date: 2026-07-06

Decision: Create/review `PT-UI-RUNTIME-PLATFORM-001` as the proposed-design / active-planning intake for the live public UI runtime platform instead of continuing with a narrow public `AppUiExt` ergonomics slice or a broad `domain/ui/ui_runtime_platform` crate.

State transition: `idea/investigating -> proposed-design / active-planning intake review`; implementation remains not authorized.

Context: PR #75 recorded `PT-UI-FRAMEWORK-APP-INTEGRATION-002` completion truth through PR #72 and removed the prior closeout blocker. Review of the UI framework architecture spine, workflow gates, completed app-integration proof, existing UI domain crates, engine App/Plugin ownership, and render submission boundaries showed that the next real platform question is broader than `AppUiExt` sugar but should still avoid a generic god crate.

Options considered: continue `PT-UI-FRAMEWORK-APP-INTEGRATION-003` as public AppUiExt ergonomics; implement a render adapter first; add a broad `domain/ui/ui_runtime_platform` crate; grow `ui_app_integration` into the final framework; create an engine-owned `UiPlugin` runtime layer that reuses `domain/ui` contracts and cleans the render boundary.

Reason: A public AppUiExt-only slice would freeze ergonomics before the runtime path is proven. A render-adapter-only slice would not solve mounted sessions, typed actions, or host mutation. A broad `domain/ui/ui_runtime_platform` crate would duplicate ownership. Growing `ui_app_integration` would violate its proof-local ECS-specific role. The best long-term path is an engine-owned `UiPlugin` runtime layer using existing domain UI contracts and a producer-agnostic render surface-frame boundary.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, and design file `../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md`.

Evidence: Workflow authority requires complete investigation and complete design gates for platform/public API/domain-boundary work. PR #72 closeout records `ui_app_integration` as proof-local and ECS-specific, with no public `AppUiExt`, no engine `UiPlugin`, and no render adapter implementation. The UI architecture spine records that render targets consume derived output only.

Follow-up: Review and harden PR #74 / `PT-UI-RUNTIME-PLATFORM-001` intake only. Do not open runtime implementation until complete investigation gate evidence, complete design gate evidence, and an exact active-planning implementation contract are recorded.

Reactivation condition: Reactivate a narrower AppUiExt-only slice only if the complete design gate proves the live runtime, host action, surface/session, and render publication contracts are already accepted and do not need this platform track.

Supersedes: the future roadmap placeholder `PT-UI-FRAMEWORK-APP-INTEGRATION-003 - Public AppUiExt Ergonomics` as the next app-framework design direction.

Superseded by: none.

## Live UiPlugin runtime design-gate hardening decision

Date: 2026-07-06

Decision: Mark `PT-UI-RUNTIME-PLATFORM-001` complete investigation/design gate hardening complete for PR #74, while keeping runtime implementation blocked until a separate implementation-planning PR records the exact implementation contract.

State transition: `active-planning intake review -> active-planning design-gate complete / implementation-planning required`.

Context: PR #74 was a coherent docs-only intake but still lacked the explicit investigation/design-gate evidence required by workspace workflow: authority/source matrix, current-state inventory, gap matrix, alternatives matrix, dependency-direction review, render-boundary review, ergonomics review, module decomposition map, allowed/forbidden files/crates, validation envelope, evidence expectation, principle compliance, feature/future/hierarchy matrices, implementation sequencing, acceptance criteria, and stop conditions.

Options considered:

```text
A. Grow ui_app_integration into final framework
B. Add broad domain/ui/ui_runtime_platform crate
C. Add engine-owned UiPlugin runtime layer and reuse domain UI crates
D. Keep RenderPlugin UI-specific
E. Genericize render toward surface-frame submissions
F. Force users to write host adapters
G. Typed UiScreen + UiActionHandler default path
H. Keep AppUiExt-only slice separate
I. Absorb AppUiExt into broader runtime platform track
```

Reason: The accepted direction is option C plus staged option E and option G/I ergonomics. `ui_app_integration` remains proof-local. A broad `domain/ui/ui_runtime_platform` crate would duplicate existing domain owners. RenderPlugin should consume producer submissions and not own UI semantics. The common app-author path remains `app.mount_ui(CounterScreen)` with typed `UiScreen`, typed `IntoUi`, and typed `UiActionHandler` / `TryUiActionHandler`; manual host adapters, route maps, event packets, and render registries remain forbidden as normal authoring paths.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, design file `../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md`, and investigation report `../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md`.

Evidence: `E2` connector metadata/file inspection, `E3` source/test inspection by path, and `E8` accepted architecture/workflow/planning authority. Local command validation was unavailable in the connector-only session; no cargo/docs validation was run. Broad GitHub code search timed out for some symbol queries, so the investigation records repo-wide grep as unavailable and relies on direct path inspection.

Follow-up: Review PR #74 as docs-only design-gate hardening. After review, open a separate implementation-planning PR for the first runtime slice. That PR must record exact owner modules, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, acceptance criteria, and stop conditions before Rust code starts.

Reactivation condition: Reopen design-gate hardening only if review finds missing gate evidence, contradictory source authority, invalid dependency direction, unbounded render migration, public API ambiguity, or a stop-condition breach.

Supersedes: the earlier PR #74 intake state where complete investigation/design gate evidence was missing.

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