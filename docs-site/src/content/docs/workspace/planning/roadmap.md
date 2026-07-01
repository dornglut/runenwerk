---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-01
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
---

# Roadmap

## Current entries

### PT-UI-COMPONENT-PLATFORM-013

ID: `PT-UI-COMPONENT-PLATFORM-013`

Title: Overlay / Popup / Layering substrate implementation

State: active implementation on PR #44

Lifecycle state: `active-implementation`

Owner: `ui_controls` for reusable overlay declarations and builders; `ui_input` for facts; `ui_runtime` for overlay replay/report/session/stack proof; `ui_static_mount` for static frame validation.

Evidence: PR #44 contains active implementation evidence. Local command validation is still required before completion.

Known gaps: Do not mark complete until the Phase 13 local validation gate runs and issues are fixed.

Next action: Run local validation, fix issues, then record closeout before merge.
