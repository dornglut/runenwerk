---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
---

# Roadmap

This is the Markdown-first roadmap record for scriptless workflow.

Use this file for human-readable WR planning state from this cutover onward.

## Entry shape

```text
ID:
Title:
State:
Owner:
Dependency level:
Write scope:
Validation expectation:
Evidence:
Next action:
Notes:
```

## Current entries

### PT-UI-COMPONENT-PLATFORM-001

ID: `PT-UI-COMPONENT-PLATFORM-001`

Title: UI Component Platform ControlPackage / ControlKernel contract

State: completed by user report

Owner: ui

Dependency level: prerequisite for component authoring-kit work

Write scope:

```text
domain/ui/ui_controls/
domain/ui/ui_artifacts/
docs-site/src/content/docs/design/active/ui-component-platform-control-kernel-design.md
```

Validation expectation:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
git diff --check
```

Evidence:

```text
User reported Phase 1 complete on 2026-06-25. Prior focused validation output was reviewed in conversation.
```

Next action:

```text
Keep as Phase 2 dependency.
```

Notes:

```text
Do not reopen Phase 1 for Phase 2+ concerns unless validation shows actual drift.
```

### PT-UI-COMPONENT-PLATFORM-002

ID: `PT-UI-COMPONENT-PLATFORM-002`

Title: UI Component Platform Authoring Kit

State: completed by user validation report

Owner: ui

Dependency level: follows Phase 1 ControlPackage / ControlKernel contract

Write scope:

```text
domain/ui/ui_controls/src/authoring/mod.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/tests/control_authoring_contract.rs
docs-site/src/content/docs/design/active/ui-component-platform-authoring-kit-design.md
```

Validation expectation:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo test -p ui_controls control_authoring
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_controls control_kernel
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
git diff --check
```

Evidence:

```text
User reported all Phase 2 validation commands green on 2026-06-25. The implementation adds a bounded authoring API that builds ordinary Phase 1 descriptors and proves valid output, explicit non-mount eligibility, and fail-closed invalid output through focused tests.
```

Next action:

```text
Keep as Phase 3 dependency.
```

Notes:

```text
Phase 2 does not implement runtime widget behavior, story runner behavior, Gallery previews, Designer UX, Workbench behavior, canvas surfaces, text editing, transitions, or runtime mount eligibility.
```

### PT-UI-COMPONENT-PLATFORM-003

ID: `PT-UI-COMPONENT-PLATFORM-003`

Title: UI Component Platform Story Proof Envelope

State: completed by user validation report

Owner: ui

Dependency level: follows Phase 2 authoring kit and consumes existing `ui_story` V2 proof contracts

Write scope:

```text
domain/ui/ui_controls/src/story_proof/mod.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/tests/control_story_proof_contract.rs
docs-site/src/content/docs/design/active/ui-component-platform-story-proof-envelope-design.md
```

Validation expectation:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo test -p ui_controls control_story_proof
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_controls control_authoring
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
cargo test -p ui_story workflow
git diff --check
```

Evidence:

```text
User reported all Phase 3 validation commands green on 2026-06-26. The implementation adds bounded story-proof requirements and summaries under ui_controls without executing stories or moving ui_story report/evidence ownership.
```

Next action:

```text
Proceed to PT-UI-COMPONENT-PLATFORM-004 Catalog / Discovery / Inspection design/planning.
```

Notes:

```text
Phase 3 does not implement catalog/discovery/inspection, story runner behavior, Gallery execution, CLI execution, runtime widget behavior, runtime mount eligibility, Designer UX, Workbench behavior, text editing, canvas surfaces, transitions, renderer-owned UI semantics, or ECS-owned UI semantics.
```

### PT-UI-COMPONENT-PLATFORM-004-PLANNING

ID: `PT-UI-COMPONENT-PLATFORM-004-PLANNING`

Title: UI Component Platform Catalog / Discovery / Inspection design intake

State: ready for planning

Owner: ui

Dependency level: follows Phase 3 Story Proof Envelope

Write scope:

```text
docs-site/src/content/docs/design/active/ui-component-platform-catalog-discovery-inspection-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Validation expectation:

```text
Manual planning consistency review.
No Rust implementation in the planning pass.
```

Evidence:

```text
Phase 4 was promoted after Phase 3 validation passed green by user report.
```

Next action:

```text
Write and review the Phase 4 Catalog / Discovery / Inspection design before implementation.
```

Notes:

```text
This planning entry does not authorize catalog UI, Gallery previews, Designer UX, Workbench behavior, runtime widget behavior, runtime mount eligibility, text editing, canvas surfaces, transitions, renderer-owned UI semantics, or ECS-owned UI semantics.
```

## Rules

- Markdown must be enough to understand the current state.
- Legacy structured files may remain as optional mirrors.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
