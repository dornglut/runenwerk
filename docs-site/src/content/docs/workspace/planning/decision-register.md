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
