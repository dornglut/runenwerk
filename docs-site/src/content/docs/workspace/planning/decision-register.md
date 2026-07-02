---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-02
related_docs:
  - ../workflow-lifecycle.md
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
