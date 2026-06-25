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

State: completed by user report; merge/CI evidence may live with the PR or local validation transcript

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
User reported Phase 1 complete on 2026-06-25. Prior focused validation output was reviewed in conversation. The Phase 2 branch is intentionally based on the completed Phase 1 branch until Phase 1 reaches main.
```

Next action:

```text
Keep as Phase 2 dependency. Rebase Phase 2 onto main after Phase 1 lands.
```

Notes:

```text
Do not reopen Phase 1 for Phase 2 authoring concerns unless validation shows actual drift.
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
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
docs-site/src/content/docs/workspace/planning/completed-work.md
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
Proceed to PT-UI-COMPONENT-PLATFORM-003 Story Proof Envelope design/planning. Do not implement story runner behavior before Phase 3 design acceptance.
```

Notes:

```text
Phase 2 does not implement runtime widget behavior, story runner behavior, Gallery previews, Designer UX, Workbench behavior, canvas surfaces, text editing, transitions, or runtime mount eligibility.
```

### PT-UI-COMPONENT-PLATFORM-003-PLANNING

ID: `PT-UI-COMPONENT-PLATFORM-003-PLANNING`

Title: UI Component Platform Story Proof Envelope design intake

State: active planning

Owner: ui

Dependency level: follows Phase 2 authoring kit and consumes existing `ui_story` V2 proof contracts

Write scope:

```text
docs-site/src/content/docs/design/active/ui-component-platform-story-proof-envelope-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Validation expectation:

```text
Manual planning consistency review.
No Rust implementation in the planning pass.
Implementation validation commands are defined in the design and deferred to the later implementation pass.
```

Evidence:

```text
Story Proof Envelope design updated with ownership split, proposed contract shape, proof categories, report-consumption rules, non-goals, boundary rules, acceptance criteria, test plan, and implementation gate.
```

Next action:

```text
Review and accept the Phase 3 design. After acceptance, run a bounded implementation pass on the same branch or a follow-up branch.
```

Notes:

```text
This planning entry does not authorize story runner behavior, Gallery execution, CLI execution, Designer UX, Workbench behavior, runtime widget behavior, runtime mount eligibility, text editing, canvas surfaces, transitions, renderer-owned UI semantics, or ECS-owned UI semantics.
```

## Rules

- Markdown must be enough to understand the current state.
- Legacy structured files may remain as optional mirrors.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
