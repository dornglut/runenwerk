---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-05
related_docs:
  - ../workflow-lifecycle.md
  - ./ecs-backed-counter-ui-story-proof-planning.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../design/active/runenwerk-typed-app-composition-plugin-framework-design.md
---

# Decision Register

Use this file to explain planning priority changes and lifecycle state transitions.

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

Follow-up: Review and accept, revise, or reject the Phase 14 design before implementation.

## Phase 14 implementation and review readiness decision

Date: 2026-07-02

Decision: Promote `PT-UI-COMPONENT-PLATFORM-014` from planning to local implementation, then move the branch to review after package-backed implementation evidence was added locally.

State transition: `active-planning -> active-implementation -> review`

Evidence: The local branch implemented editable-text vocabulary, descriptor wiring, package validation, InspectorField text-editing lowering, catalog projection, inspection projection, normalized text edit/composition/selection facts, `ui_runtime::text_editing` replay/report/proof-frame evidence, static mount validation, no-bypass evidence, and focused tests. Local validation passed on 2026-07-02 with the recorded Phase 14 cargo/docs/diff gate.

Follow-up: After acceptance or merge, record Phase 14 completion truth and open Phase 15.

## Phase 14 completion decision

Date: 2026-07-02

Decision: Mark `PT-UI-COMPONENT-PLATFORM-014` Text Editing / Editable Text Behavior completed through merged PR #46.

State transition: `review -> completed`

Evidence: PR #46 merged into `main` at merge commit `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`. Post-merge inspection showed `main` identical to that merge commit. Main contains package-backed editable-text declarations, InspectorField text-editing lowering, package descriptor wiring, package validation, catalog projection, inspection projection, normalized text edit/composition/selection facts, `ui_runtime::text_editing` replay/report/value/caret/selection/composition/suppression/no-bypass proof, proof-frame projection, static mount validation, focused tests, and final proof-frame cleanup. Local Phase 14 validation passed on 2026-07-02 before merge.

Follow-up: Keep Phase 14 as completed dependency and use it as the preceding substrate for Generic Text planning.

## Phase 15 generic text planning-start decision

Date: 2026-07-02

Decision: Start `PT-UI-COMPONENT-PLATFORM-015` as Generic Text design/planning intake.

State transition: `production-track -> active-planning`

Evidence: Existing `ui_text` owns renderer-independent text contracts. Phase 15 planning scope is text runs, inline spans, wrapping, alignment, truncation/ellipsis, line metrics, glyph/run evidence, package/catalog/inspection projection, visual proof, and static mount proof. Text editing, rich text editor behavior, code editor behavior, product document buffers, undo/redo, clipboard, app-specific text rendering policy, dynamic plugin framework, `foundation/meta`, shared plugin primitives, UI Designer, UI Gallery product surface, Workbench/provider redesign, product/editor/game mutation, authored UI editing, compatibility-only aliases/shims, and phase-shaped public API names remain out of scope.

Follow-up: Review and refine the Generic Text design. Do not implement until active planning is promoted with exact scope, owner files, validation, evidence, and stop conditions.

## Phase 15 generic text implementation closeout decision

Date: 2026-07-02

Decision: Move `PT-UI-COMPONENT-PLATFORM-015` Generic Text out of active planning after local implementation closeout evidence passed on PR #48.

State transition: `active-planning -> active-implementation -> review`

Evidence: PR #48 branch `ui/generic-text-phase-15` at implementation commit `32e402b108d1e72d7cc5b4113af29d8d29626680` implemented the renderer-neutral Generic Text substrate across `ui_text`, `ui_render_data`, `ui_controls`, `ui_runtime`, and `ui_static_mount`. Evidence covers text block/run/span/source-range/style/layout evidence, Generic Text package descriptors and validation reasons, catalog projection, separate `TextDisplay` inspection projection, runtime proof report/frame, static mount proof, no-bypass boundary assertions, migration away from the old compatibility path, and renderer-neutral frame/extract adaptation without renderer backend ownership.

Validation: `cargo test -p ui_text`, `cargo test -p ui_render_data`, `cargo test -p ui_controls`, `cargo test -p ui_runtime`, `cargo test -p ui_static_mount`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check` passed locally on 2026-07-02.

Follow-up: Complete PR #48 merge, then record the merge commit.

## Phase 15 generic text baseline completion decision

Date: 2026-07-02

Decision: Mark `PT-UI-COMPONENT-PLATFORM-015` Generic Text baseline completed through merged PR #48.

State transition: `review -> completed`

Evidence: PR #48 merged into `main` at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`. The validated implementation commit `32e402b108d1e72d7cc5b4113af29d8d29626680` passed the local Phase 15 gate: `cargo test -p ui_text`, `cargo test -p ui_render_data`, `cargo test -p ui_controls`, `cargo test -p ui_runtime`, `cargo test -p ui_static_mount`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`.

Follow-up: Keep Phase 15 as completed dependency and perform the hardening/future-proofing review before opening Phase 16 implementation.

## Phase 15 generic text hardening completion decision

Date: 2026-07-02

Decision: Mark the Phase 15 Generic Text hardening pass completed through merged PR #49 without starting Phase 16 implementation.

State transition: `completed -> completed-hardening`

Evidence: PR #49 merged into `main` at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9`. It corrected resolved source-run and cluster evidence, added height overflow evidence, added stable-ID text constructors, added generic text layout shape helpers, replaced the remaining button default role-specific text policy, segmented visual runs by homogeneous evidence, exposed text direction policy through Generic Text inspection, renamed runtime text helpers to `text_emission`, and split large runtime output emission files. Final Phase 15 validation passed on 2026-07-02 with the full package, workspace, docs, and diff gate.

Follow-up: Use PR #48 plus PR #49 as the authoritative Phase 15 completion evidence. Open Phase 16 Surface2D as planning/design hardening only.

## Phase 16 Surface2D planning-start decision

Date: 2026-07-02

Decision: Start `PT-UI-COMPONENT-PLATFORM-016` as Surface2D design/planning intake after Phase 15 baseline and hardening evidence were recorded.

State transition: `production-track -> active-planning`

Evidence: Phase 15 Generic Text completed through PR #48 at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff` and PR #49 at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9`. The existing Surface2D design scopes generic renderer-neutral 2D surface identity, content/viewport bounds, world/screen transforms, pan, zoom, fit, selection rectangle, hover coordinate, pointer capture, gesture cancel/commit, overlay/diagnostic layers, grid/background vocabulary, large-content bounds, LOD readiness, and budget evidence.

Follow-up: Harden the Surface2D design before implementation. Planning must settle exact owner files, minimum scope, validation envelope, no product/editor/game mutation rule, accessibility/input acceptance, performance/budget evidence, stop conditions, and the relationship to existing `ui_surface` vocabulary. Typed App Composition remains proposed architecture reference only, not implementation authority.

## Phase 16 Surface2D completion decision

Date: 2026-07-03

Decision: Mark `PT-UI-COMPONENT-PLATFORM-016` Surface2D completed through docs-hardening PR #62 and implementation PR #61.

State transition: `review -> completed`

Context: PR #62 merged the generic workflow, principle, decomposition, and merge-readiness hardening before Phase 16 completion. PR #61 then squash-merged the Surface2D implementation into `main`.

Options considered: mark Phase 16 completed; leave Phase 16 in review; reopen implementation for the leftover `surface2d-phase-16` branch.

Reason: `main` contains the delivered Surface2D package/catalog/inspection contract, runtime proof report/frame, and static mount proof. Post-merge validation from `main` passed, and no Phase 16 product blocker remains. The leftover branch was not safe to fold into closeout because it changes only the Surface2D design document and mixes potentially useful future-use-case pressure with stale pre-merge implementation assumptions.

Affected planning files: `active-work.md`, `completed-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `ui-component-platform-surface2d-design.md`, `phase-16-surface2d-source-investigation.md`, and `phase-16-surface2d-closeout.md`.

Evidence: PR #62 merged at `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`. PR #61 squash-merged at `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. Post-merge validation from `main` passed with `cargo test -p ui_controls surface2d`, `cargo test -p ui_controls control_package`, `cargo test -p ui_runtime surface2d`, `cargo test -p ui_static_mount surface2d`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`.

Follow-up: Keep Phase 16 as a completed dependency and perform the next production-track planning intake. The next named future milestone is `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas, but this decision does not authorize Phase 17 implementation.

## Phase 16 leftover branch decision

Date: 2026-07-03

Decision: Keep remote branch `surface2d-phase-16` for manual review instead of deleting it during closeout.

State transition: none

Context: Cleanup inspection found `surface2d-phase-16` still has commits not patch-equivalent to `origin/main`.

Options considered: delete as obsolete; apply all changes as follow-up; keep for manual review.

Reason: `git cherry -v origin/main origin/surface2d-phase-16` reported three `+` commits. `git diff --stat origin/main...origin/surface2d-phase-16` showed one large design-document diff, `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`, with 689 changed lines. The diff contains some potentially useful future-use-case pressure, but it also contains stale pre-implementation assumptions that conflict with the merged Phase 16 implementation, including active-planning language, older implementation owner assumptions, and monolithic file naming.

Affected planning files: `phase-16-surface2d-closeout.md`, `active-work.md`, and `production-tracks.md`.

Evidence: `git log --oneline --decorate origin/main..origin/surface2d-phase-16` showed `7bc6abf1 Add Surface2D future use-case matrix`, `6f571270 Harden Surface2D genericity and hierarchy planning`, and `b5a0cc52 Refine Surface2D planning gate`. `git diff --name-status origin/main...origin/surface2d-phase-16` showed only the Surface2D design document modified.

Follow-up: Review `surface2d-phase-16` manually outside the Phase 16 closeout. If useful material remains, extract a focused docs-only follow-up from `main` without mixing it into completed closeout truth.

## UI framework app integration direction decision

Date: 2026-07-05

Decision: Replace the active next-work focus from `PT-UI-APP-PROGRAM-001` manual Typed App Program headless counter implementation to `PT-UI-FRAMEWORK-APP-INTEGRATION-001` UI Framework App Integration Direction Review.

State transition: `PT-UI-APP-PROGRAM-001 active-planning/implementation-spike -> superseded`; `PT-UI-FRAMEWORK-APP-INTEGRATION-001 -> active-planning`.

Context: Review of the UI architecture, UI story workflow, runtime rendering pipeline, UI Component Platform roadmap, Interaction V2 ADR, Typed App Program design, PR #69, and Phase 17 SpatialCanvas intake showed that the repository has strong UI substrate but lacks a canonical app-facing framework path. PR #69 follows its prior planning contract, but that contract starts from a manual proof-IR layer rather than the real app-authoring question.

Options considered: merge PR #69 as the app framework foundation; continue raw ECS UI; make external templates the only immediate path; promote SpatialCanvas; select ECS/App/Plugin-hosted UI-definition-backed app integration.

Reason: The best long-term direction is App/ECS-hosted app authoring that lowers UI through `ui_definition`, FormedInteractionModel, `UiProgram`, runtime artifacts, and `UiStory` reports while keeping app mutation in host/app owners. Raw ECS UI would make ECS own UI semantics. The current `app_program` proof would make manual snapshots, route maps, reducers, and projection reports feel like the public framework. SpatialCanvas is a future reusable surface, not the app integration layer.

Affected planning files: `active-work.md`, `decision-register.md`, `typed-app-program-ui-proof-001-planning.md`, and new design file `ui-framework-app-integration-direction-review.md`.

Evidence: Current UI domain roadmap records `ui_definition`, `ui_program`, `ui_program_lowering`, `ui_compiler`, `ui_artifacts`, `ui_binding`, `ui_story`, runtime views, render primitives, and gallery proof as active current truth. The UI Story workflow records the full source-to-mount eligibility pipeline. The UI Runtime Rendering Pipeline forbids visible UI success without the upstream reports. The UI Component Platform roadmap rejects ECS-owned UI semantics and app-only mutation shortcuts. PR #69 is open/draft and implements a manual `app_program` proof crate. PR #65 is open/draft as SpatialCanvas planning-only.

Follow-up: Review and accept, revise, or reject `ui-framework-app-integration-direction-review.md`. If accepted, close or keep PR #69 only as a superseded spike and write a new implementation-planning contract for `ECS-backed Counter UI Story Proof`.

Reactivation condition: Reactivate `PT-UI-APP-PROGRAM-001` only if a later accepted design decides that a dedicated `app_program` crate is required as proof/report vocabulary and is not the public UI framework authoring path.

Supersedes: `PT-UI-APP-PROGRAM-001` as the next active implementation foundation.

Superseded by: `PT-UI-FRAMEWORK-APP-INTEGRATION-001`.

## ECS-backed Counter UI Story Proof implementation-planning decision

Date: 2026-07-05

Decision: Convert `PT-UI-FRAMEWORK-APP-INTEGRATION-002` from broad proof-planning intake into a conditional implementation contract for a small UI-owned `ui_app_integration` crate and an ECS-backed Counter UiStory-compatible proof.

State transition: `active-planning intake -> active-planning implementation contract`; implementation branch authorized only after this docs PR is reviewed/merged and the implementation stays within the accepted contract.

Context: Source inspection showed the root workspace already contains the relevant UI crates through `ui_story`; `ui_definition` owns authored template/node/slot structures; `ui_program` owns route/event packet contracts; `ui_program_lowering` owns the current source-to-UiProgram formation entrypoint; `ui_controls` owns package registry snapshots; `ui_story` is a V2 proof contract surface; `ui_testing` already depends on the proof/evaluation stack; `ecs` exposes resource/world/system primitives; and `engine::App` exposes resource/system methods while keeping world internals private.

Options considered: pure fixture/story-only proof; immediate public AppUiExt API; small UI-owned integration crate with internal/proof-local APIs first; revive `app_program`; implement SpatialCanvas first.

Reason: The best long-term path is `C-internal first, then B-public later`: create a small UI-owned integration bridge now, prove the Counter loop through internal/proof-local APIs, use `ui_story`/`ui_testing` only as proof harnesses, and defer public AppUiExt-style engine ergonomics until dependency direction and owner boundaries are validated. This avoids both fixture-only proof drift and premature public API freeze.

Affected planning files: `active-work.md`, `decision-register.md`, and `ecs-backed-counter-ui-story-proof-planning.md`.

Evidence: `Cargo.toml` has no `ui_app_integration` member yet; `ui_definition` has `AuthoredUiTemplate`, `UiNodeDefinition`, and route/value slot refs; `ui_program` has `UiEventPacket`, `RouteId`, `RouteSchemaVersion`, and `RouteCapability`; `ui_program_lowering` has `form_ui_program_report_from_node_with_registry_snapshot`; `ui_controls` has `runenwerk_control_package` and `ControlPackageRegistrySnapshot`; `ui_testing` already depends on the full proof/evaluation stack; `engine::App` world state is private, so the first public App extension should be deferred.

Follow-up: After this docs PR is accepted, open an implementation branch that changes only the allowed files: root `Cargo.toml`, new `domain/ui/ui_app_integration` crate files, and focused counter proof tests. Do not add public engine::App extension methods in the first implementation.

Reactivation condition: Reopen the broader planning question only if implementing the exact contract requires engine core dependency inversion, public App extension API, `ui_definition`/`ui_program` dependency on ECS, `app_program` resurrection, or bypassing `ui_definition`, `UiProgram`, or story-compatible reports.

Supersedes: `PT-UI-FRAMEWORK-APP-INTEGRATION-002` broad A/B/C intake state.

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
