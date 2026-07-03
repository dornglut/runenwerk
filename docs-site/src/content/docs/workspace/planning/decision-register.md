---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-spatial-canvas-design.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../reports/investigations/phase-17-spatialcanvas-source-investigation.md
  - ../../reports/investigations/surface2d-future-pressure-branch-review.md
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

## Phase 16 future-pressure extraction decision

Date: 2026-07-03

Decision: Treat the Surface2D future-pressure extraction as completed through merged PR #64 and remove the retained stale branch from active planning blockers.

State transition: none

Context: PR #64 merged `docs-site/src/content/docs/reports/investigations/surface2d-future-pressure-branch-review.md` at merge commit `05c51375986cf08e360884ebf44702ec62662c1e`. The report preserved future Surface2D/Canvas2D/canvas-family pressure as reference material without changing completed Phase 16 truth or starting Phase 17 work.

Options considered: leave the retained-branch warning in active planning; update planning to reflect the extraction and current branch state; reopen Phase 16.

Reason: Current repository state contains the extracted report, GitHub reports no open PRs, and `git branch --all --verbose --no-abbrev` during Phase 17 intake no longer lists `origin/surface2d-phase-16`. The retained branch is no longer an active planning blocker. Phase 16 remains completed and is not reopened.

Affected planning files: `active-work.md`, `production-tracks.md`, and this decision register.

Evidence: PR #64 metadata, current local branch inspection, and `surface2d-future-pressure-branch-review.md`.

Follow-up: Use the extracted future-pressure report only as design-pressure reference for later canvas work.

Supersedes: The branch-retention follow-up in `Phase 16 leftover branch decision`.

## Phase 17 SpatialCanvas planning-start decision

Date: 2026-07-03

Decision: Open `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas as active planning/design intake only.

State transition: `production-track -> active-planning`

Context: Phase 16 Surface2D is completed through PR #61 after PR #62 workflow hardening. PR #63 closed Phase 16 planning truth, and PR #64 extracted stale Surface2D future-pressure reference material. The next named UI Component Platform milestone is SpatialCanvas, but implementation is not authorized.

Options considered: start implementation immediately; defer Phase 17; open planning/design intake only.

Reason: SpatialCanvas is reusable platform work that may affect public API, renderer-neutral contracts, input behavior, inspection/catalog projection, retained UI boundaries, and product/editor/game separation. Source investigation found that `ui_controls`, `ui_runtime`, and `ui_static_mount` are the likely first owners, while `ui_tree`, `ui_render_data`, `ui_render_primitives`, `ui_input`, `ui_surface`, `ui_composition`, `ui_graph_editor`, renderer backends, product/editor/game owners, new crates, plugin framework work, and `foundation/meta` remain non-owners unless later evidence and design explicitly promote them.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `ui-component-platform-spatial-canvas-design.md`, and `phase-17-spatialcanvas-source-investigation.md`.

Evidence: `docs-site/src/content/docs/reports/investigations/phase-17-spatialcanvas-source-investigation.md`, `docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md`, completed Surface2D closeout evidence, UI domain ownership docs, and current source inspection.

Follow-up: Review and accept, revise, defer, or reject the SpatialCanvas design. Do not implement until active planning explicitly promotes Phase 17 to `active-implementation` with exact owner files, complete contract, validation envelope, evidence expectation, module decomposition, principle compliance, and stop conditions.

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
