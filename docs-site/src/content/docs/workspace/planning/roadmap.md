---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-01
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-implementation-scope.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
---

# Roadmap

This is the Markdown-first roadmap record for scriptless workflow.

Use this file for human-readable WR planning state from this cutover onward.

## Current entries

### PT-UI-COMPONENT-PLATFORM-001

ID: `PT-UI-COMPONENT-PLATFORM-001`

Title: UI Component Platform ControlPackage / ControlKernel contract

State: completed by user report

Owner: ui

Evidence: User reported Phase 1 complete on 2026-06-25.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-002

ID: `PT-UI-COMPONENT-PLATFORM-002`

Title: UI Component Platform Authoring Kit

State: completed by user validation report

Owner: ui

Evidence: User reported Phase 2 validation green on 2026-06-25.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-003

ID: `PT-UI-COMPONENT-PLATFORM-003`

Title: UI Component Platform Story Proof Envelope

State: completed by user validation report

Owner: ui

Evidence: User reported Phase 3 validation green on 2026-06-26.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-004

ID: `PT-UI-COMPONENT-PLATFORM-004`

Title: UI Component Platform Catalog / Discovery / Inspection

State: completed by user validation report

Owner: ui

Evidence: User reported Phase 4 validation green on 2026-06-26.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-005

ID: `PT-UI-COMPONENT-PLATFORM-005`

Title: UI Component Platform Input / Gesture / Device

State: completed by user validation report

Owner: ui

Evidence: User reported Phase 5 validation green on 2026-06-26.

Next action: Keep as completed dependency; later migrate generic vocabulary to owner crates.

### PT-UI-COMPONENT-PLATFORM-006

ID: `PT-UI-COMPONENT-PLATFORM-006`

Title: UI Component Platform State Binding / Host Intent

State: completed by user validation report

Owner: ui

Evidence: User reported Phase 6 validation green on 2026-06-26.

Next action: Keep as completed dependency; later migrate generic vocabulary to owner crates.

### PT-UI-COMPONENT-PLATFORM-007

ID: `PT-UI-COMPONENT-PLATFORM-007`

Title: UI Component Platform Theme / State / Style

State: completed by user validation report

Owner: ui

Evidence: User reported Phase 7 validation green on 2026-06-26.

Next action: Keep as completed dependency; later migrate generic vocabulary to owner crates.

### PT-UI-COMPONENT-PLATFORM-008

ID: `PT-UI-COMPONENT-PLATFORM-008`

Title: UI Component Platform Accessibility / Focus / Inspection

State: completed by user validation report

Owner: ui

Evidence: User reported Phase 8 validation green and merged on 2026-06-26.

Next action: Keep as completed dependency; later migrate generic vocabulary to owner crates.

### PT-UI-COMPONENT-PLATFORM-009

ID: `PT-UI-COMPONENT-PLATFORM-009`

Title: UI Component Platform Layout / Container / Virtualization

State: completed by user validation report

Owner: `ui_layout` for generic layout vocabulary; `ui_controls` for the control-facing bridge.

Evidence: PR #29 merged the corrected Phase 9 work and the user reported the validation gate green.

Next action: Keep as completed dependency; do not use PR #30 or `feature/ui-component-platform-009-layout`.

### PT-UI-COMPONENT-PLATFORM-009A

ID: `PT-UI-COMPONENT-PLATFORM-009A`

Title: UI Component Platform Ownership Realignment

State: completed by user validation report

Owner: ui

Evidence: Ownership realignment design exists and PR #29 merged the accepted owner-first correction.

Next action: Keep as completed planning dependency for later owner-crate vocabulary migrations.

### PT-UI-COMPONENT-PLATFORM-009B

ID: `PT-UI-COMPONENT-PLATFORM-009B`

Title: UI Component Platform Layout Foundation

State: completed by user validation report

Owner: ui_layout

Dependency level: follows accepted 009A

Evidence: Generic layout vocabulary exists in `ui_layout`, is exported publicly, and has focused layout contract tests.

Next action: Keep as completed dependency for 009C and Phase 10.

### PT-UI-COMPONENT-PLATFORM-009C

ID: `PT-UI-COMPONENT-PLATFORM-009C`

Title: UI Component Platform Control Layout Bridge

State: completed by user validation report

Owner: ui_controls

Dependency level: follows 009B implementation

Evidence: `ui_controls` layout descriptors reference `ui_layout` vocabulary directly, expose read-only catalog inspection facts, and have focused control layout and catalog bridge tests.

Next action: Keep as completed dependency; do not add generic layout vocabulary to `ui_controls`.

### PT-UI-COMPONENT-PLATFORM-010

ID: `PT-UI-COMPONENT-PLATFORM-010`

Title: UI Component Platform Render Surface / Output

State: completed by user validation report

Owner: `ui_render_data` for renderer-neutral output evidence contracts; `ui_controls` for the control-facing bridge; `ui_runtime` for emitted-frame evidence generation; engine render for submission proof without UI semantics.

Evidence: PR #34 merged the full P10 owner-first implementation into `main`. User reported the full validation gate green on 2026-06-26.

Next action: Keep as completed dependency for Phase 11 and Phase 12.

### PT-UI-COMPONENT-PLATFORM-011

ID: `PT-UI-COMPONENT-PLATFORM-011`

Title: UI Component Platform Base Control Packages

State: completed through PR #37 and user validation report

Lifecycle state: `completed`

Owner: `ui_controls` for base control package descriptors, UI-local contribution declarations, preset/lowering code, catalog projection, and read-only inspection facts. Owner crates such as `ui_layout`, `ui_render_data`, `ui_input`, `ui_theme`, and accessibility/focus contracts remain the source of generic vocabulary.

Evidence: PR #37 merged Phase 11 into `main` on 2026-06-28. The proof covers Label, Button, InspectorField, ColorPicker, ActionPrompt, ListView, TreeView, and TableView through `BaseControlsPlugin`, `UiControls`, `ControlContribution`, `ControlDef`, control presets, field groups, theme groups, `ControlCompiler`, catalog projection, and inspection projection. Reported local validation passed for the Phase 11 cargo/docs/diff gate.

Known gaps: Full runtime interaction remains Phase 12. Overlays/popups/layering remain Phase 13. Text editing remains later. No shared plugin framework extraction, no `foundation/meta`, and no generic plugin primitives are authorized.

Next action: Keep Phase 11 as completed dependency and proceed to `PT-UI-COMPONENT-PLATFORM-012-PLANNING`.

### PT-UI-COMPONENT-PLATFORM-012

ID: `PT-UI-COMPONENT-PLATFORM-012`

Title: UI Component Platform Generic Interaction

State: completed through merged PR #43 and user validation report

Lifecycle state: `completed`

Owner: `ui_controls` for reusable control interaction declarations and descriptor/catalog/inspection facts; `ui_input` for normalized input packets, device/gesture facts, pointer/key/focus data, and runtime input sample formation; `ui_runtime` for resolving normalized input facts against runtime UI structure, producing reusable interaction facts/events, and projecting semantic proof to `UiFrame`; `ui_static_mount` for renderer-neutral static mount validation; hosts/apps/editor/game for OS/window input collection, routing policy, command handling, product state changes, game/world input policy, and product-specific behavior.

Authority: `ui-component-platform-generic-interaction-design.md`, `ui-component-platform-input-gesture-device-design.md`, `editor-ui-runtime-v2-and-interaction-formation-design.md`, the Phase 11 closeout report, and the UI Component Platform production track.

Evidence: PR #43 merged into `main` on 2026-06-30 at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f`. User start condition reports PR #43 was validated and merged. The merged implementation provides package-backed `ControlInteractionDescriptor` records, catalog/inspection interaction summaries, normalized pointer/keyboard/focus/semantic/text-intent facts, descriptor-driven mounted replay/report, the renderer-neutral visible proof model in `ui_runtime` through `InteractionVisualProof`, `InteractionVisualMainView`, `InteractionInspectorView`, `InteractionReportView`, `InteractionVisibleState`, and `InteractionProofFrame`, and static mount proof through `InteractionProofRenderFrame`/`UiFrame`/`UiStaticMountReport::from_frame`. The proof covers Button hover/pressed/focus-visible/activation/disabled suppression, List/Tree/Table intent markers, InspectorField text-intent probe behavior, read-only text-intent probe behavior, no-target/disabled/focus-negative cases, deterministic frame primitive ordering, and zero host-command/product-mutation/overlay/text-edit boundary assertions.

Known gaps: Product-facing UI Gallery exposure is separate future work under `PT-UI-GALLERY-001`. Phase 13 overlay/popup/layering, later full text editing, app/editor/game command handling, backend renderer behavior, broad shared plugin framework extraction, `foundation/meta`, and generic plugin primitives remain out of scope.

Next action: Keep Phase 12 as completed dependency.

### PT-UI-COMPONENT-PLATFORM-012A

ID: `PT-UI-COMPONENT-PLATFORM-012A`

Title: UI Component Platform Executable Interaction Story

State: completed through merged PR #43 and user validation report

Lifecycle state: `completed`

Owner: `ui_story` for workflow profile/evidence envelope authority, `ui_runtime` for interaction story session execution mechanics and replay/live application, `ui_input` for normalized input facts and minimal conversion helpers, `ui_controls` for reusable interaction descriptors and read-only declarations, `ui_static_mount` for static frame validation, and `runenwerk_editor` for the narrow proof-host adapter from `UiInputEvent` to runtime session evidence.

Authority: `ui-component-platform-executable-interaction-story-design.md`, `ui-component-platform-executable-interaction-story-implementation-scope.md`, `ui-component-platform-generic-interaction-design.md`, `ui-component-platform-story-proof-envelope-design.md`, `ui-component-platform-input-gesture-device-design.md`, `editor-ui-runtime-v2-and-interaction-formation-design.md`, and the UI Component Platform production track.

Evidence: PR #43 merged into `main` on 2026-06-30 at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f`. User start condition reports PR #43 was validated and merged. The merged implementation provides an executable generic interaction story, deterministic replay, live proof-host event application through the same runtime session path, semantic replay/live parity, static frame validation, and no-bypass counters. The durable proof-host evidence uses base-controls names, including `BaseControlsInteractionProofHost`, and does not claim product-facing Gallery exposure.

Known gaps: Product-facing editor window/gallery display is not part of PR #43 and remains separate future work under `PT-UI-GALLERY-001`. UI Lab / 012B surface work was removed from the PR scope and is not valid evidence for UI Gallery exposure.

Next action: Keep Phase 12A as completed dependency. Do not start product-facing UI Gallery exposure, text editing, product-command adoption, shared plugin framework extraction, generic plugin primitives, or `foundation/meta` from this milestone.

### PT-UI-COMPONENT-PLATFORM-013

ID: `PT-UI-COMPONENT-PLATFORM-013`

Title: Overlay / Popup / Layering full implementation

State: review on PR #44 after local validation gate passed

Lifecycle state: `review`

Owner: `ui_controls` owns reusable overlay declarations, ergonomic builders, base-control lowering, package descriptors, package validation, catalog projection, and inspection projection. `ui_input` owns normalized input facts only. `ui_runtime` owns package-backed overlay intent/session/stack/placement/layer/focus/dismissal/suppression/replay/report/proof-frame/no-bypass evidence under `ui_runtime::overlay`. `ui_static_mount` owns static frame validation. Product/editor/game behavior remains outside generic UI.

Authority: `ui-component-platform-overlay-popup-layering-design.md`, completed Phase 12/12A docs, `editor-ui-runtime-v2-and-interaction-formation-design.md`, `editor-self-authoring-and-final-ui-design.md`, and the UI Component Platform production track.

Evidence: PR #44 now contains implementation evidence for package-backed overlay declarations, base-control lowering, main-path package validation, catalog projection, inspection projection, normalized input fact consumption, runtime package-backed replay/report/stack/placement/focus/dismissal/suppression proof, proof-frame projection, static mount proof, and no-bypass evidence. Local validation passed on 2026-07-02 with the full Phase 13 cargo/docs/diff gate.

Known gaps: Phase 13 is not recorded in completed work until PR #44 is accepted or merged. UI Gallery, UI Designer, authored UI editing, product command execution, product/editor/game mutation, full text editing, dynamic plugin framework, `foundation/meta`, shared plugin primitives, Workbench/provider redesign, backend renderer behavior, and world-space overlays remain out of scope.

Next action: Review PR #44 and merge only after explicit instruction. After PR acceptance or merge, record Phase 13 completion truth before opening the next implementation phase.

## Rules

- Markdown must be enough to understand the current state.
- Legacy structured files may remain as optional mirrors.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
- Use `../workflow-lifecycle.md` before changing lifecycle state.
- Accepted direction does not authorize implementation. Roadmap entries that enter implementation must have exact owner, scope, validation envelope, evidence expectation, and stop conditions.

## Entry shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority:
Evidence:
Known gaps:
Next action:
```

Existing entries may be migrated as they are touched.
