---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-29
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-design.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-012A-PLANNING`

Title: UI Component Platform Executable Interaction Story

State: accepted direction / implementation planning

Lifecycle state: `active-planning`

Owner: Planning spans `ui_story` for executable story identity/evidence envelope authority, `ui_runtime` for interaction story session execution mechanics and replay/live application, `ui_input` for normalized pointer/keyboard/focus/text-intent samples, `ui_controls` for reusable interaction descriptors and read-only catalog/inspection declarations, `ui_static_mount` for static `UiFrame` validation, and the existing gallery/proof host layer for live input collection and proof presentation. Product/editor/app layers remain later consumers only.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/documentation-structure.md`, `docs-site/src/content/docs/workspace/authority-model.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/planning/README.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md`, and `docs-site/src/content/docs/design/active/ui-component-platform-story-proof-envelope-design.md`.

Write scope: Documentation/planning only until exact implementation scope is accepted. Do not implement runtime sessions, gallery live hosts, host input adapters, product behavior, overlays, text editing, shared plugin framework extraction, generic plugin primitives, or `foundation/meta` until the active-work entry is promoted to `active-implementation` with exact owner files/crates, validation envelope, evidence expectation, and stop conditions.

Validation expectation: This planning patch should be readable from Markdown and should validate with `python3 tools/docs/validate_docs.py` and `git diff --check` when a local checkout is available. Command execution is not required to understand or review the accepted direction.

Known blockers: The Tier 5 design is accepted, but implementation is not scoped. Exact owner files/crates, host adapter location, runtime session API scope, focused test names, evidence artifacts, and manual live validation steps still need inspection before implementation can start.

Next action: Inspect the actual `ui_story`, `ui_runtime`, `ui_input`, `ui_static_mount`, and gallery/proof-host files to create an exact implementation scope for `PT-UI-COMPONENT-PLATFORM-012A`. Do not write code until that implementation scope is recorded and accepted.

Evidence: User accepted `ui-component-platform-executable-interaction-story-design.md` on 2026-06-29. PR #43 already provides the lower-tier assets this design should reuse: package-backed interaction descriptors, catalog/inspection projection, normalized input facts, descriptor-driven replay/report, `InteractionVisualProof`, `InteractionProofRenderFrame`, and `UiStaticMountReport::from_frame`. The accepted Tier 5 direction requires replay mode, live proof-host mode, shared normalized input path, semantic replay/live parity, static frame artifact, and no-bypass boundary assertions.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, and stop conditions are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Write scope:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
