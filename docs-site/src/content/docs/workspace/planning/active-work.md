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
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-001`

Title: `Live UiPlugin Runtime and Generic Surface-Frame Rendering`

State: PR #74 is an open draft docs-only intake/design-gate hardening PR.

Lifecycle state: `active-planning` design-gate complete / implementation-planning required. Not `active-implementation`.

Owner: UI runtime/platform planning owns the gate record. A future engine `UiPlugin` implementation owner is proposed, but no implementation owner/files are authorized by this active-work entry.

Authority files:

```text
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
```

Evidence classes: `E2` connector metadata/file inspection, `E3` source/test inspection by path, and `E8` accepted architecture/workflow/planning authority. No `E5` local command validation is available from this connector-only session.

Complete investigation gate: complete for PR #74 design-gate hardening. See the investigation report.

Complete design gate: complete for opening a separate implementation-planning PR only. Runtime implementation remains blocked.

Implementation contract: not authorized. A separate implementation-planning PR must record exact owner modules, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, acceptance criteria, and stop conditions before Rust code starts.

Allowed files/crates for current focus: docs-only investigation/design/planning records in PR #74.

Non-owned files/crates for current focus:

```text
runtime Rust implementation
engine UiPlugin code
public AppUiExt code
app.mount_ui implementation
UiScreen / IntoUi implementation
UiActionHandler implementation
render adapter code
SurfaceFrame type migration code
SDF/world-space/SpatialCanvas implementation
foundation/meta
domain/app_program
generic plugin framework
```

Known blockers: runtime implementation is blocked until the next implementation-planning PR is opened and accepted with an exact implementation contract and validation envelope.

Next action: review PR #74 as docs-only design-gate hardening. After review, open a separate implementation-planning PR for the first runtime slice; do not start implementation from PR #74.

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
Known blockers:
Next action:
```