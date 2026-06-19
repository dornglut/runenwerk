---
title: WR-180 UI Composition Cutover Governance And Visual Direction Gate Plan
description: Decision-complete checkpoint-1 contract for accepted architecture, ordered production authority, and the mandatory three-option visual selection before implementation.
status: active
owner: ui
layer: workspace
canonical: false
last_reviewed: 2026-06-19
related_adrs:
  - ../../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_designs:
  - ../../../design/accepted/app-neutral-ui-composition-design.md
  - ../../../design/accepted/adaptive-ui-composition-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-180 UI Composition Cutover Governance And Visual Direction Gate Plan

## Authority

This plan executes `PM-UI-COMPOSITION-001` and `WR-180` only. The user has
approved the architecture, clean-cutover sequence, and implementation-readiness
amendments.

This checkpoint authorizes documentation governance and visual ideation. It
does not authorize Rust, runtime, chrome, docking, editor behavior, Draw
behavior, crate creation, or legacy-code mutation.

## Required Outputs

1. Accepted core composition design.
2. Accepted dependent adaptive composition design.
3. Accepted ADR 0013 superseding ADRs 0006 and 0012.
4. Applied `WR-180` through `WR-189` roadmap sequence.
5. Active `PT-UI-COMPOSITION-CUTOVER` and reviewed execution manifest.
6. Exactly three independent 1440x1024 visual directions grounded in actual
   checked-in Runenwerk captures and current theme tokens.
7. One recorded user selection and a checkpoint closeout.

## Visual Brief

Design a modern IDE/browser hybrid backed by the future adaptive region graph.
Preserve Runenwerk's compact black/dark-gray, zero-radius tool aesthetic.

Every option must show:

- browser-like tab movement;
- explicit center and edge docking targets;
- floating/detached root affordance;
- real-window detach intent;
- policy-driven compact drawers/overflow;
- visible focus;
- touch-sized resize affordances;
- realistic editor content at 1440x1024.

The options must vary information hierarchy, layout strategy, and docking
feedback, not brand identity. The user must choose one before any product UI
implementation starts.

## Sequence

1. Inspect repository governance, current code truth, existing captures, and
   theme sources.
2. Replace the compatibility-first intake with the approved clean-cutover
   contracts.
3. Apply roadmap authority and create the ordered production track/manifest.
4. Render and validate roadmap, production, docs, planning, and PUML sources.
5. Inspect the actual visual references directly.
6. Generate exactly three independent options through the Product Design
   ideation workflow.
7. Stop and ask the user to select option 1, 2, or 3.
8. Record the selected option and only then close PM-001/WR-180.

## Stop Conditions

- Stop before image generation if no actual checked-in Runenwerk capture can be
  inspected and attached.
- Stop if governance validation fails or production/roadmap authority drifts.
- Stop after presenting three options until the user selects one.
- Stop before any runtime/chrome/docking implementation.
- Do not mark checkpoint 1 complete while visual selection is absent.

## Validation

```text
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task docs:validate
task planning:validate
task puml:validate
```

## Closeout

Closeout belongs at
`docs-site/src/content/docs/reports/closeouts/pm-ui-composition-001-governance-and-visual-direction-gate/closeout.md`.
It must link all three visual artifacts, record the selected direction, confirm
that no product code changed, list validation evidence, and name WR-181 as the
next legal checkpoint.
