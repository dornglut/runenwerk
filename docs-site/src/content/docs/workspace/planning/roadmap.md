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

Dependency level: prerequisite for component platform work

Evidence:

```text
User reported Phase 1 complete on 2026-06-25.
```

Next action:

```text
Keep as completed dependency.
```

### PT-UI-COMPONENT-PLATFORM-002

ID: `PT-UI-COMPONENT-PLATFORM-002`

Title: UI Component Platform Authoring Kit

State: completed by user validation report

Owner: ui

Dependency level: follows Phase 1

Evidence:

```text
User reported Phase 2 validation green on 2026-06-25.
```

Next action:

```text
Keep as completed dependency.
```

### PT-UI-COMPONENT-PLATFORM-003

ID: `PT-UI-COMPONENT-PLATFORM-003`

Title: UI Component Platform Story Proof Envelope

State: completed by user validation report

Owner: ui

Dependency level: follows Phase 2

Evidence:

```text
User reported Phase 3 validation green on 2026-06-26.
```

Next action:

```text
Keep as completed dependency.
```

### PT-UI-COMPONENT-PLATFORM-004

ID: `PT-UI-COMPONENT-PLATFORM-004`

Title: UI Component Platform Catalog / Discovery / Inspection

State: completed by user validation report

Owner: ui

Dependency level: follows Phase 3

Evidence:

```text
User reported Phase 4 validation green on 2026-06-26.
```

Next action:

```text
Keep as completed dependency.
```

### PT-UI-COMPONENT-PLATFORM-005

ID: `PT-UI-COMPONENT-PLATFORM-005`

Title: UI Component Platform Input / Gesture / Device

State: completed by user validation report

Owner: ui

Dependency level: follows Phase 4 Catalog / Discovery / Inspection

Write scope:

```text
domain/ui/ui_controls/src/input.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/catalog/mod.rs
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/catalog/index.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/src/catalog/query.rs
domain/ui/ui_controls/tests/control_input_contract.rs
domain/ui/ui_controls/tests/control_input_catalog_contract.rs
docs-site/src/content/docs/design/active/ui-component-platform-input-gesture-device-design.md
```

Validation expectation:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo test -p ui_controls control_input
cargo test -p ui_controls control_catalog
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_controls control_story_proof
cargo test -p ui_controls control_authoring
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
git diff --check
```

Evidence:

```text
User reported Phase 5 validation green on 2026-06-26 after catalog split cleanup.
```

Next action:

```text
Proceed to PT-UI-COMPONENT-PLATFORM-006 State Binding / Host Intent design/planning.
```

Notes:

```text
Phase 5 remains declarative. It does not add product-owned behavior, renderer semantics, or ECS semantics.
```

### PT-UI-COMPONENT-PLATFORM-006-PLANNING

ID: `PT-UI-COMPONENT-PLATFORM-006-PLANNING`

Title: UI Component Platform State Binding / Host Intent design intake

State: active planning

Owner: ui

Dependency level: follows Phase 5 Input / Gesture / Device

Write scope:

```text
docs-site/src/content/docs/design/active/ui-component-platform-state-binding-host-intent-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/completed-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Validation expectation:

```text
Manual planning consistency review.
No Rust implementation in the planning pass.
```

Evidence:

```text
Phase 6 was promoted after Phase 5 validation passed green by user report.
```

Next action:

```text
Review and accept the Phase 6 State Binding / Host Intent design before implementation.
```

Notes:

```text
This planning entry does not authorize state binding code, host intent code, app/editor/game mutation, runtime widget behavior, runtime mount eligibility, text editing implementation, canvas surfaces, renderer behavior, or ECS behavior.
```

## Rules

- Markdown must be enough to understand the current state.
- Legacy structured files may remain as optional mirrors.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
