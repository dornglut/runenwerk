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

### PT-UI-COMPONENT-PLATFORM-009A

ID: `PT-UI-COMPONENT-PLATFORM-009A`

Title: UI Component Platform Ownership Realignment

State: active planning

Owner: ui

Dependency level: follows Phase 8 Accessibility / Focus / Inspection

Write scope:

```text
docs-site/src/content/docs/design/active/ui-component-platform-ownership-realignment-design.md
docs-site/src/content/docs/design/active/ui-component-platform-layout-container-virtualization-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Validation expectation:

```text
Manual planning consistency review.
No Rust migration in this pass.
```

Evidence: Manual ownership audit found the completed Phase 5-8 code stayed declarative but put some generic vocabulary in ui_controls instead of owner crates.

Next action: Review and accept the ownership realignment and corrected Phase 9 design.

### PT-UI-COMPONENT-PLATFORM-009B

ID: `PT-UI-COMPONENT-PLATFORM-009B`

Title: UI Component Platform Layout Foundation

State: future

Owner: ui_layout

Dependency level: follows accepted 009A

Next action: Add generic layout/container/scroll/virtualization vocabulary to ui_layout after 009A acceptance.

### PT-UI-COMPONENT-PLATFORM-009C

ID: `PT-UI-COMPONENT-PLATFORM-009C`

Title: UI Component Platform Control Layout Bridge

State: future

Owner: ui_controls

Dependency level: follows 009B

Next action: Add control-facing layout requirements and catalog inspection summaries that reference ui_layout types.

## Rules

- Markdown must be enough to understand the current state.
- Legacy structured files may remain as optional mirrors.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
