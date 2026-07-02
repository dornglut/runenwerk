---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-01
related_docs:
  - ../workflow-lifecycle.md
---

# Decision Register

Use this file to explain planning priority changes and lifecycle state transitions.

## Initial cutover decision

Date: 2026-06-25

Decision: Runenwerk workspace planning becomes Markdown-first for scriptless workflow.

Reason: GitHub connector and context-tool workflows cannot rely on full repo export, command execution, generated prompts, or rendered planning views.

Affected files: `planning/README.md`, `active-work.md`, `roadmap.md`, `deferred-work.md`, `completed-work.md`, `production-tracks.md`.

Follow-up: Copy detailed legacy rows into Markdown planning records as they are touched.

## Phase 2 authoring-kit planning decision

Date: 2026-06-25

Decision: Start Phase 2 as an authoring-kit design/planning intake before implementation.

Reason: The authoring kit needed accepted owner, scope, non-goals, validation, and stop conditions before code.

Follow-up: Completed by user validation report.

## Phase 3 story-proof-envelope planning decision

Date: 2026-06-25

Decision: Start Phase 3 as a Story Proof Envelope design/planning intake before implementation.

Reason: Story proof had to consume existing `ui_story` V2 authority instead of creating a parallel control-specific proof model.

Follow-up: Completed by user validation report.

## Phase 4 catalog planning decision

Date: 2026-06-26

Decision: Start Phase 4 as a Catalog / Discovery / Inspection design intake before implementation.

Reason: Catalog facts had to remain derived read-only projections from ControlPackage descriptors and proof summaries.

Follow-up: Completed by user validation report.

## Phase 5 input planning decision

Date: 2026-06-26

Decision: Start Phase 5 as an Input / Gesture / Device design intake before implementation.

Reason: Input, gesture, and device facts had to remain declarative package facts while runtime input collection, routing, and mutation stayed outside the component platform.

Follow-up: Completed by user validation report.

## Phase 6 state planning decision

Date: 2026-06-26

Decision: Start Phase 6 as a State Binding / Host Intent design intake before implementation.

Reason: Reusable controls may describe state buckets, binding requirements, and host intent proposals. Actual app/editor/game mutation, route authorization, persistence, and domain-specific rules remain host-owned.

Follow-up: Completed by user validation report.

## Phase 7 theme planning decision

Date: 2026-06-26

Decision: Start Phase 7 as a Theme / State / Style design intake before implementation.

Reason: Theme, visual state, and style facts need reusable declarations before controls can share consistent presentation semantics without moving renderer or product styling ownership into ui_controls.

Follow-up: Completed by user validation report.

## Phase 8 accessibility planning decision

Date: 2026-06-26

Decision: Start Phase 8 as an Accessibility / Focus / Inspection design intake before implementation.

Reason: Accessibility roles, focus semantics, keyboard navigation, and inspection metadata need reusable declarations before layout, render, base controls, or interaction phases consume them.

Follow-up: Completed by user validation report.

## Phase 9A ownership realignment decision

Date: 2026-06-26

Decision: Insert an ownership realignment pass before Phase 9 implementation.

Reason: Completed Phases 5-8 stayed declarative but duplicated some generic UI vocabulary in ui_controls instead of anchoring that vocabulary in owner crates.

Follow-up: Accept the realignment design, then implement layout foundation in ui_layout before adding the ui_controls bridge.

## Phase 9 closeout decision

Date: 2026-06-26

Decision: Mark `PT-UI-COMPONENT-PLATFORM-009` Layout / Container / Virtualization complete.

Context: PR #29 merged the corrected Phase 9 work into `main`. PR #30 is closed unmerged and superseded.

Options considered: Keep Phase 9 pending local validation; close Phase 9 based on user validation report; reopen the stale pre-realignment branch.

Reason: User reported the Phase 9 validation gate green after the merged 009A/009B/009C work. The remote repository shows the owner-first implementation on `main`, and PR #30 explicitly records the stale branch as superseded.

Affected planning files: `completed-work.md`, `roadmap.md`, `production-tracks.md`, `active-work.md`.

Evidence: 009A ownership realignment design, 009B `ui_layout` layout foundation, 009C `ui_controls` layout bridge over `ui_layout`, read-only catalog inspection bridge, focused tests, and user validation report.

Follow-up: Open Phase 10 Render Surface / Output planning. Do not revive PR #30 or `feature/ui-component-platform-009-layout`.

## Phase 10 render surface / output planning decision

Date: 2026-06-26

Decision: Start `PT-UI-COMPONENT-PLATFORM-010-PLANNING` as an owner-first Render Surface / Output design intake before implementation.

Context: Phase 10 needs reusable render/output evidence without repeating the Phase 9 ownership mistake.

Options considered: Put generic render/output vocabulary in `ui_controls`; use `ui_render_data` as renderer-facing output owner with `ui_runtime` and engine render as adjacent execution owners; defer Phase 10 entirely.

Reason: Repository authority and code inspection place renderer-facing `UiFrame`, surface, layer, primitive, product surface, and viewport embed contracts in `ui_render_data`; retained output generation in `ui_runtime`; and backend rendering execution in `engine/src/plugins/render`. `ui_controls` should only expose per-control render evidence requirements and summaries that reference owner contracts.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `ui-component-platform-render-surface-output-design.md`.

Evidence: `DOMAIN_MAP.md`, crate inventory, UI architecture docs, `ui_render_data` primitive exports, `ui_runtime::build_ui_frame`, `ui_controls` existing render evidence requirement fields, and Phase 9 ownership realignment rule.

Follow-up: Accept the Phase 10 design, then implement owner-first slices. No Rust implementation is authorized by this planning pass.

## Phase 10 closeout decision

Date: 2026-06-26

Decision: Mark `PT-UI-COMPONENT-PLATFORM-010` Render Surface / Output complete.

Context: PR #34 merged the full owner-first Phase 10 implementation into `main`.

Options considered: Keep P10 pending local validation; close P10 from PR #34 plus user validation report; split P10 into further render/runtime/backend subtasks.

Reason: User reported the full P10 validation gate green after PR #34 merged. The merged implementation covers `ui_render_data`, `ui_controls`, `ui_runtime`, and engine render proof while preserving owner boundaries.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`.

Evidence: PR #34, renderer-neutral output evidence, control render bridge, runtime output evidence generation, engine render submission proof, and user validation report.

Follow-up: Open Phase 11 Base Control Packages design intake.

## Phase 11 base control packages planning decision

Date: 2026-06-26

Decision: Start `PT-UI-COMPONENT-PLATFORM-011-PLANNING` as a Base Control Packages design intake before implementation.

Context: The component platform now has descriptor, proof, catalog, layout, accessibility, and render/output evidence layers. The base control package needs hardening before Gallery, Workbench, or UI Designer should rely on it as reusable product-facing inventory.

Options considered: Start full interaction immediately; harden base control packages first; skip to overlay/text/canvas phases.

Reason: Full interaction needs credible base control descriptors to operate on. Phase 11 should make Label, Button, InspectorField, ColorPicker, ActionPrompt, ListView, TreeView, and TableView package-quality without taking over Phase 12 interaction behavior.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `ui-component-platform-base-control-packages-design.md`.

Evidence: Completed Phases 1-10 and the existing base control modules in `ui_controls`.

Follow-up: Review the Phase 11 design, then implement package hardening. Full interaction remains Phase 12.

## Phase 11 closeout decision

Date: 2026-06-28

Decision: Mark `PT-UI-COMPONENT-PLATFORM-011` Base Control Packages complete.

State transition: `review -> completed`

Context: PR #37 merged the Phase 11 implementation into `main` after the base-control refactor changed the old explicit inventory into a UI-local contribution, preset, and lowering proof.

Options considered: Leave Phase 11 in review until another local validation pass; close Phase 11 based on merged PR #37 plus reported green validation; reopen Phase 11 for shared plugin-framework extraction.

Reason: PR #37 is merged and records the intended Phase 11 scope. The implementation stayed in `domain/ui/ui_controls`, covered the eight target base controls, and did not introduce shared plugin infrastructure, `foundation/meta`, generic plugin primitives, or full runtime interaction.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, `ui-component-platform-base-control-packages-design.md`, and the Phase 11 closeout report.

Evidence: PR #37, `BaseControlsPlugin`, `UiControls`, `ControlContribution`, `ControlDef`, control presets, field groups, theme groups, `ControlCompiler`, `ControlCatalog`, `ControlInspection`, per-control `control_contribution()` modules, and reported green Phase 11 validation.

Follow-up: Open `PT-UI-COMPONENT-PLATFORM-012-PLANNING` as Generic Interaction design intake. Preserve Phase 12 as planning only until owner boundaries and validation gates are accepted.

## Phase 12 generic interaction planning decision

Date: 2026-06-28

Decision: Start `PT-UI-COMPONENT-PLATFORM-012-PLANNING` as a Generic Interaction design intake before implementation.

State transition: `production-track -> active-planning`

Context: Phase 11 provides descriptor-backed, catalog-visible, package-quality base controls. Reusable interaction behavior is still out of scope and must be designed without moving host policy or product state changes into `ui_controls`.

Options considered: Treat the Phase 5 input design as sufficient; treat the editor Interaction V2 design as sufficient; create a component-platform-specific Phase 12 design intake that references both but owns the cross-crate boundary for reusable controls.

Reason: The Phase 5 input design covers declarative input/gesture/device facts, and the editor Interaction V2 design covers retained editor/runtime interaction formation. Phase 12 needs an explicit component-platform design that connects `ui_controls`, `ui_input`, `ui_runtime`, and host/app/editor/game ownership for reusable controls.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, and `ui-component-platform-generic-interaction-design.md`.

Evidence: Phase 11 closeout, Phase 5 input/gesture/device design, editor Interaction V2 design, and the current UI Component Platform production track.

Follow-up: Review the Phase 12 design intake. Do not authorize implementation until the later PR has exact owner files, non-goals, validation gate, evidence expectation, and stop conditions.

## Phase 12 generic interaction review evidence decision

Date: 2026-06-29

Decision: Keep `PT-UI-COMPONENT-PLATFORM-012` Generic Interaction in review while PR #43 cleanup, validation, and merge are pending.

State transition: `active-implementation -> review`

Context: PR #43 on branch `codex/phase-12-generic-interaction` implemented package-backed reusable interaction descriptors, catalog/inspection visibility, normalized input facts, descriptor-driven replay/report, renderer-neutral visible proof, and boundary assertions without app/editor/game command behavior or product mutation.

Options considered: Keep Phase 12 in review until PR #43 is cleaned up, validated, and merged; complete Phase 12 using the renderer-neutral `InteractionVisualProof`/`InteractionProofFrame` path plus deterministic replay/report tests before merge; reopen Phase 12 to add overlay/layering or text editing.

Reason: The accepted design requires visible proof, descriptor-backed replay, no-bypass evidence, and explicit deferral of overlays/layering and full text editing. PR #43 provides a renderer-neutral main/inspector/report proof model formed from compiled base-control package descriptors, but completion cannot be claimed before the branch is cleaned up, validated, reviewed, and merged. Requiring a product-facing gallery page would expand Phase 12 into unrelated gallery/framework work.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, and `ui-component-platform-generic-interaction-design.md`.

Evidence: `ControlPackageDescriptor::interaction_descriptors`, `ControlPackageAuthoringBuilder::with_interaction_descriptor`, `ControlCatalogIndex::from_packages`, base-control interaction lowering, `NormalizedInputFact`, `MountedInteractionFixture`, `InteractionFormationReport`, `InteractionVisualProof`, `InteractionProofFrame`, and `interaction_replay_report` tests on PR #43.

Follow-up: Finish PR #43 cleanup and validation, then review/merge PR #43 before marking Phase 12 complete. Keep Phase 13 overlays/layering and later full text editing deferred until separate active planning records authorize them.

## Phase 12A executable interaction story planning decision

Date: 2026-06-29

Decision: Start `PT-UI-COMPONENT-PLATFORM-012A-PLANNING` as an Executable Interaction Story design intake before implementation.

State transition: `production-track -> active-planning`

Context: Phase 12 provides contract, replay, report, renderer-neutral visible proof, and static frame evidence. That lower-tier evidence is useful, but it does not yet prove live gallery/proof-host interaction where actual pointer/key/focus/text-intent input updates reusable interaction state through the same normalized input and runtime formation path.

Options considered: Keep static proof as the final interaction standard; add an ad-hoc live Button demo; define a Tier 5 executable story standard with deterministic replay, live proof-host execution, semantic replay/live parity, static frame evidence, and no-bypass assertions.

Reason: Static proof alone is too weak for future reusable interaction claims, and a live demo alone can bypass descriptors, catalog facts, normalized input, and runtime interaction formation. The stronger long-term standard is one executable story that can run in replay mode and live mode, with both modes differing only by input source after normalization.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `design/active/README.md`, and `ui-component-platform-executable-interaction-story-design.md`.

Evidence: PR #43 lower-tier interaction assets, Phase 3 story-proof ownership boundary, the Phase 12 design and review evidence, and the new executable interaction story design intake.

Follow-up: Review and accept, revise, or reject the Tier 5 design. Do not authorize code until owner files/crates, host adapter location, runtime session API scope, validation envelope, evidence artifacts, manual live validation, and stop conditions are accepted.

## Phase 12A executable interaction story acceptance decision

Date: 2026-06-29

Decision: Accept `ui-component-platform-executable-interaction-story-design.md` as the Tier 5 executable interaction story direction.

State transition: `proposed-design -> accepted-direction`

Context: User reviewed and accepted the Tier 5 design intake. The design records owner boundaries, vocabulary, non-goals, tradeoffs, acceptance criteria, implementation gate, stop conditions, and relationship to current work.

Options considered: Revise the design before acceptance; accept the Tier 5 direction and keep implementation blocked until exact scope is inspected; start implementation immediately.

Reason: The design provides the required long-term standard without authorizing speculative implementation. It keeps the lower-tier Phase 12 static/replay proof as reusable evidence, but requires a future executable story with replay/live semantic parity before live reusable interaction proof can be claimed.

Affected planning files: `ui-component-platform-executable-interaction-story-design.md`, `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: User acceptance on 2026-06-29, the accepted design document, PR #43 lower-tier interaction assets, and the updated active-work planning entry.

Follow-up: Inspect actual `ui_story`, `ui_runtime`, `ui_input`, `ui_static_mount`, and gallery/proof-host files. Then create the exact active-implementation scope with owner files/crates, host adapter location, runtime session API scope, validation envelope, evidence artifacts, manual live validation, and stop conditions.

## Phase 12A executable interaction story implementation-scope decision

Date: 2026-06-29

Decision: Promote `PT-UI-COMPONENT-PLATFORM-012A` to active implementation using `ui-component-platform-executable-interaction-story-implementation-scope.md`.

State transition: `active-planning -> active-implementation`

Context: The accepted Tier 5 direction has now been inspected against the available owner crates. `ui_story` has a V2 proof-envelope workflow model, `ui_runtime` has descriptor-backed replay/proof assets, `ui_input` has normalized input facts and high-level input events, `ui_static_mount` has direct `UiFrame` validation, and `runenwerk_editor` has shell input seams but no dedicated `ui_gallery` crate or module on this branch.

Options considered: Keep implementation blocked until a product-facing gallery module exists; implement broad editor shell/product gallery integration now; implement the narrow Tier 5 proof-host slice with exact files, shared replay/live runtime path, semantic parity, static mount evidence, and stop conditions.

Reason: The narrow proof-host slice is the correct next step. It proves executable interaction without overreaching into product-facing shell surfaces, overlays, text editing, command dispatch, generic plugin infrastructure, or `foundation/meta`. Product-facing editor/window/gallery exposure can follow after this proof-host evidence exists.

Affected planning files: `ui-component-platform-executable-interaction-story-implementation-scope.md`, `active-work.md`, `roadmap.md`, `production-tracks.md`, and `decision-register.md`.

Evidence: Inspection of `ui_story` V2 workflow files, `ui_runtime` generic interaction fixture/replay/proof files, `ui_input` fact/event files, `ui_static_mount::UiStaticMountReport::from_frame`, `runenwerk_editor` dependencies and shell input seams, and the implementation scope document.

Follow-up: Implement only the scoped files. Stop and record a scope revision if implementation requires editor shell surface registry changes, product command execution, product mutation, overlays, full text editing, a parallel story runner, a generic plugin framework, `foundation/meta`, or generic plugin primitives.

## Phase 12 / 12A review correction and UI Gallery split decision

Date: 2026-06-30

Decision: Correct PR #43 planning truth: `PT-UI-COMPONENT-PLATFORM-012` and `PT-UI-COMPONENT-PLATFORM-012A` are in review and pending cleanup, validation, and merge. Remove `PT-UI-COMPONENT-PLATFORM-012B` from active planning and split UI Gallery product exposure into separate future `PT-UI-GALLERY-001`.

State transition: `completed` / `active-implementation` -> `review`

Context: PR #43 contains useful Phase 12 lower-tier generic interaction evidence and Phase 12A executable interaction story evidence, but the branch also introduced flawed 012B/UI Lab proof-surface work that reused provider id `11`, created invalid Workbench provider assignment behavior, and overstated UI Gallery/product exposure. Completion language was premature because PR #43 had not merged.

Options considered: Repair the flawed UI Lab surface attempt inside PR #43; keep 012B active and add more provider-registry work; remove 012B from PR #43, keep 012/012A in review, and plan UI Gallery separately.

Reason: The long-term boundary keeps reusable interaction semantics and executable story proof in owner crates and narrow proof-host evidence. UI Gallery exposure is product/platform adoption work, not required to validate PR #43's reusable interaction proof. Keeping the flawed 012B surface would blur provider identity, Workbench composition, and gallery claims.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, and the PR #43 body.

Evidence: User cleanup instruction on 2026-06-30, restored workbench coverage, duplicate provider assignment failure from the 012B UI Lab surface path, and the accepted executable interaction story design's stop conditions around product-facing gallery and shell-surface expansion.

Follow-up: Update PR #43 body to remove 012B/UI Lab surface claims, validate the focused Phase 12/12A gate, and merge only after the branch contains no 012B work or planning claims.

Reactivation condition: Create `PT-UI-GALLERY-001` only through a separate active planning entry with owner, scope, validation envelope, evidence expectation, and stop conditions.

## Phase 12 / 12A completion and Phase 13 activation decision

Date: 2026-06-30

Decision: Mark `PT-UI-COMPONENT-PLATFORM-012` and `PT-UI-COMPONENT-PLATFORM-012A` completed from merged PR #43 evidence and start `PT-UI-COMPONENT-PLATFORM-013` as overlay/popup/layering design intake.

State transition: `review -> completed`; `production-track -> active-planning`

Context: PR #43 is closed and merged into `main` at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f`. User start condition reports PR #43 validated and merged. The PR body records Phase 12 generic interaction evidence and Phase 12A executable interaction story evidence while excluding overlays, popups, dropdowns, tooltips, modals, layering, product command execution, product/editor/game mutation, full text editing, dynamic plugin loading, and UI Gallery exposure.

Options considered: Leave Phase 12/12A in review because the PR body still contains stale review wording; mark completion using PR metadata plus user validation evidence and correct planning truth; start Phase 13 implementation immediately.

Reason: Planning truth must follow merged repository evidence. The stale PR-body review sentence is superseded by the actual closed/merged PR metadata and the user-reported validation start condition. Phase 13 can begin as design intake only because overlays/layering were explicitly excluded from PR #43 and need their own owner-first contract before code.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`, `ui-component-platform-generic-interaction-design.md`, `ui-component-platform-executable-interaction-story-implementation-scope.md`, and `ui-component-platform-overlay-popup-layering-design.md`.

Evidence: PR #43 merged metadata, merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f`, user validation/merge start condition, PR #43 proof-path list, durable base-controls naming cleanup, and current planning docs.

Follow-up: Keep Phase 13 in active planning until `ui-component-platform-overlay-popup-layering-design.md` is accepted and a Rust implementation scope records exact owner crates/files, validation commands, proof scenarios, negative scenarios, evidence contracts, no-bypass assertions, and stop conditions.

## Phase 13 overlay / popup / layering design intake decision

Date: 2026-06-30

Decision: Use `ui-component-platform-overlay-popup-layering-design.md` as the active Phase 13 design document for reusable overlay, popup, dropdown, tooltip, focus-containing, and layering semantics.

State transition: `active-planning -> proposed-design`

Context: The existing overlay design was stale activation-vocabulary material. The component platform now has base controls, generic interaction facts, executable interaction stories, and static frame proof. The older editor Interaction V2 design also records popup-stack lessons for anchoring, focus return, outside dismissal, Escape dismissal, scroll ownership, layer order, and viewport arbitration, but those lessons must be extracted without moving editor shell behavior or commands into generic UI.

Options considered: Implement editor popup behavior directly; turn product UI Gallery or UI Designer into the Phase 13 target; create a reusable package-backed overlay/layering design that consumes Phase 12/12A and keeps product behavior outside generic UI.

Reason: Overlay, popup, dropdown, tooltip, menu, and focus-containing behavior are reusable UI platform concerns, but app-specific command execution, product mutation, authored UI editing, and Workbench/provider behavior are not. A design-first owner map prevents a repeat of the removed 012B/UI Lab overreach.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, and `ui-component-platform-overlay-popup-layering-design.md`.

Evidence: Completed Phase 12/12A PR #43 proof, accepted executable interaction story standard, Interaction V2 popup-stack contract, and implemented self-authoring UI design goals that defer product/editor-specific command and authoring behavior.

Follow-up: Review and accept, revise, or reject the Phase 13 design. Do not authorize implementation until the implementation-scope section is accepted.

## Phase 13 implementation activation decision

Date: 2026-07-01

Decision: Continue PR #44 as the full `PT-UI-COMPONENT-PLATFORM-013` implementation PR instead of merging it as a design-only PR.

State transition: `active-planning -> active-implementation`

Context: User correction clarified that one PR should close the whole phase. The accepted overlay/popup/layering design records owner boundaries, validation gate, evidence contract, no-bypass assertions, and stop conditions. The implementation scope must be corrected to place overlay runtime modules under a runtime overlay module, not under `ui_runtime::input`.

Options considered: Merge PR #44 as design intake and open a later implementation PR; continue PR #44 and implement Phase 13 before merge; defer Phase 13 implementation.

Reason: Continuing PR #44 avoids a half-finished phase while preserving the accepted owner-first design. Implementation must still keep input fact ownership separate from overlay runtime orchestration.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, and `ui-component-platform-overlay-popup-layering-design.md`.

Evidence: PR #44 contains implementation-in-progress evidence for `ui_controls` overlay declarations, `ui_runtime` overlay replay/report/session/stack proof, base-controls overlay fixtures/scripts, focused tests, workflow consumption, and static mount proof. Local command validation remains required before completion.

Follow-up: Move overlay runtime implementation out of `ui_runtime::input`, add the report-to-proof-frame path, run the full Phase 13 validation gate from a local checkout, fix compile/test/docs issues, then record completion truth only after PR acceptance or merge.

## Phase 13 review readiness decision

Date: 2026-07-02

Decision: Move `PT-UI-COMPONENT-PLATFORM-013` to review on PR #44 after full local validation passed.

State transition: `active-implementation -> review`

Context: PR #44 now implements the full overlay / popup / layering chain required by the canonical design: package-backed overlay declarations, base-control lowering, main-path package validation, catalog projection, inspection projection, normalized input fact consumption, runtime package-backed replay/report/proof, static mount proof, and no-bypass evidence.

Options considered: Keep Phase 13 in active implementation despite green validation; mark Phase 13 completed before PR acceptance; move the phase to review and keep completion recording blocked until PR #44 is accepted or merged.

Reason: Local validation is green, so implementation is no longer blocked. Completion should still wait for PR acceptance or merge so planning truth follows repository history and does not start the next implementation phase early.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, and the PR #44 body.

Evidence: The full Phase 13 gate passed locally on 2026-07-02: `cargo fmt --all --check`, focused `cargo check`, package/catalog/inspection overlay tests, `ui_input` input tests, runtime overlay layering tests, runtime package-backed tests, static mount tests, docs validation, and `git diff --check`.

Follow-up: Review PR #44 and merge only after explicit instruction. After PR acceptance or merge, record Phase 13 completion truth before opening the next implementation phase.

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
