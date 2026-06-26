---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-26
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

Evidence: PR #29 merged the corrected owner-first Phase 9 work. 009A recorded the ownership realignment rule, 009B added generic layout/container/scroll/content/identity/virtualization vocabulary in `ui_layout`, 009C added the `ui_controls` control layout bridge over `ui_layout`, catalog inspection exposes read-only layout summaries, focused tests exist, and the user reported the validation gate green on 2026-06-26.

Next action: Keep as completed dependency for Phase 10; do not use PR #30 or `feature/ui-component-platform-009-layout`.

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

Evidence: Generic layout/container/scroll/content/identity/virtualization vocabulary exists in `ui_layout`, is exported publicly, and has focused layout contract tests.

Next action: Keep as completed dependency for 009C and Phase 10.

### PT-UI-COMPONENT-PLATFORM-009C

ID: `PT-UI-COMPONENT-PLATFORM-009C`

Title: UI Component Platform Control Layout Bridge

State: completed by user validation report

Owner: ui_controls

Dependency level: follows 009B implementation

Evidence: `ui_controls` layout descriptors reference `ui_layout` vocabulary directly, expose read-only catalog inspection facts, and have focused control layout and catalog bridge tests.

Next action: Keep as completed dependency; do not add generic layout vocabulary to `ui_controls`.

### PT-UI-COMPONENT-PLATFORM-010-PLANNING

ID: `PT-UI-COMPONENT-PLATFORM-010-PLANNING`

Title: UI Component Platform Render Surface / Output design intake

State: active planning

Owner: `ui_render_data` for renderer-facing output contracts; `ui_runtime` and `engine/src/plugins/render` are adjacent execution owners. `ui_controls` is bridge-only after owner contracts exist.

Evidence: Ownership investigation found renderer-facing `UiFrame`, surface, layer, and primitive contracts in `ui_render_data`; retained tree to frame output generation in `ui_runtime`; and backend render execution in `engine/src/plugins/render`.

Next action: Accept the owner-first Phase 10 design before any Rust implementation.

## Rules

- Markdown must be enough to understand the current state.
- Legacy structured files may remain as optional mirrors.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
