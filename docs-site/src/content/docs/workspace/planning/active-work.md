---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-06
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
  - ../../architecture/ui-framework-architecture.md
  - ./ecs-backed-counter-ui-story-proof-planning.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../../design/active/ui-runtime-rendering-pipeline-roadmap.md
  - ../../design/active/runenwerk-ui-platform-capability-roadmap.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

Architecture reference: the canonical top-down architecture is
`docs-site/src/content/docs/architecture/ui-framework-architecture.md`; the
current focus is reviewing and hardening the draft docs-only intake in PR #74
for Live `UiPlugin` runtime and generic surface-frame rendering. This focus is
planning/intake review only. It must not start runtime implementation.

ID: `PT-UI-RUNTIME-PLATFORM-001`

Title: `Live UiPlugin Runtime and Generic Surface-Frame Rendering Intake`

State: draft docs-only intake open in PR #74. Review and harden the intake
before any implementation authorization.

Lifecycle state: `active-planning` intake review. Not `active-implementation`.

Owner: UI runtime/platform planning owns the intake. Runtime, engine plugin,
surface-frame rendering, host adapter, SDF/world-space, and public plugin API
owners remain unimplemented until the intake is accepted and a complete
implementation contract is recorded.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md`, `docs-site/src/content/docs/architecture/ui-framework-architecture.md`, completed `PT-UI-FRAMEWORK-APP-INTEGRATION-002` closeout truth, and draft PR #74 intake evidence.

Evidence classes: `E8` accepted architecture/planning authority, `E3` source
inspection of the completed `ui_app_integration` proof boundary, and GitHub
metadata for draft PR #74. Implementation evidence for
`PT-UI-RUNTIME-PLATFORM-001` is not yet available and must not be inferred.

Complete investigation gate: required for runtime/platform implementation
authorization. The PR #74 intake may collect and harden investigation evidence,
but implementation remains forbidden until the complete gate is accepted.

Complete design gate: not satisfied for runtime implementation. The intake must
prove the target owner map, public/private API boundary, module decomposition,
validation envelope, evidence expectation, and stop conditions before any
runtime work starts.

Implementation contract: not authorized. PR #74 is a docs-only intake and must
not be promoted to active implementation by this active-work entry.

Allowed files/crates: docs-only intake review files in PR #74. No runtime,
engine, UI crate, render adapter, SDF, SpatialCanvas, `foundation/meta`, or
generic plugin framework implementation files are authorized by this focus.

Non-owned files/crates: runtime implementation, engine plugin APIs,
surface-frame render adapters, SDF/world-space implementation, SpatialCanvas
implementation, public `AppUiExt`, generic `UiPlugin` execution framework,
`foundation/meta`, `domain/app_program`, product/editor/game mutation, renderer
backend ownership, and any shortcut that bypasses `ui_definition`, `UiProgram`,
or story-compatible proof reports.

Principle compliance matrix: pending for PR #74 intake review. The intake must
preserve KISS/DRY/YAGNI/SOLID/Separation of Concerns/Avoid Premature
Optimization/Law of Demeter before implementation can be authorized.

Module decomposition map: not accepted for runtime implementation yet. PR #74
must make owner modules and split triggers explicit before implementation.

Maintainability review status: pending intake review. Implementation remains
blocked until decomposition, public API shape, validation, and stop conditions
are accepted.

Feature support matrix: pending intake review. Live `UiPlugin` runtime,
generic surface-frame rendering, host policy, render target ownership, route
and mutation boundaries, and proof/mount eligibility must be classified before
implementation.

Future-use-case pressure matrix: pending intake review. Public plugin runtime,
editor/game hosts, SDF/world-space targets, retained/runtime frame output, and
future App/Plugin ergonomics must be positioned without moving semantics into
the wrong owner.

Hierarchy/composition matrix: pending intake review. Runtime surface hierarchy,
host/plugin hierarchy, render-frame hierarchy, and product/semantic hierarchy
must remain separate.

Ergonomics/usability: public runtime/plugin ergonomics remain design pressure,
not implementation authorization.

Validation expectation: docs-only intake review should pass
`python tools/docs/validate_docs.py` and `git diff --check`. Runtime
implementation validation must be defined by the future accepted contract.

Known blockers: runtime implementation is blocked until the PR #74 intake is
reviewed/hardened and accepted with complete investigation/design/planning
evidence. The completed PR #72 proof is now historical evidence, not an active
blocker.

Next action: review and harden PR #74 / `PT-UI-RUNTIME-PLATFORM-001` intake
only. Do not start Live `UiPlugin` runtime implementation.

Evidence: `PT-UI-FRAMEWORK-APP-INTEGRATION-002` is completed through PR #72 at
merge commit `e093eb1affdc465b96430200960f8e3cdca0d26b` and closeout report
`../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md`.
GitHub reports PR #74 as an open draft docs-only intake,
`Docs: open Live UiPlugin runtime intake`, from
`docs/live-uiplugin-runtime-intake` to `main`.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, stop conditions, principle compliance status, and module decomposition status are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Evidence classes:
Complete investigation gate:
Complete design gate:
Implementation contract:
Allowed files/crates:
Non-owned files/crates:
Principle compliance matrix:
Module decomposition map:
Maintainability review status:
Feature support matrix:
Future-use-case pressure matrix:
Hierarchy/composition matrix:
Ergonomics/usability:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
