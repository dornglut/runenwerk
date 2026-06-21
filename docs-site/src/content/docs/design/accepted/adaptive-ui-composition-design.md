---
title: Adaptive UI Composition And Docking Design
description: Accepted dependent design for transient adaptive projection, responsive reflow, proposals, previews, semantic input, docking, cross-window movement, accessibility, and measurable performance.
status: accepted
owner: ui
layer: domain/app
canonical: true
last_reviewed: 2026-06-19
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_designs:
  - ./app-neutral-ui-composition-design.md
  - ./editor-native-multi-window-presentation-design.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# Adaptive UI Composition And Docking Design

## Status And Dependency

This accepted design depends on the core composition contract. Runtime chrome,
docking, preview, drawer, and motion work cannot start until checkpoint 1 has
produced exactly three grounded visual directions and the user has selected one.

The visual source is the checked-in Runenwerk editor captures and current theme
tokens. The target remains compact black/dark-gray, zero-radius, and tool-dense,
with improved hierarchy, focus, previews, motion, and accessibility.

## Ownership

`domain/ui/ui_adaptive_composition` owns derived mechanism only:

- `AdaptiveProjectionState` and adaptive region graph;
- viewport reflow policy and minimums;
- docking/snap targets and previews;
- drag and resize sessions;
- transient edit classification;
- promotion deltas;
- semantic-input proposals.

It never commits structural state, runs app commands, owns native windows, or
persists projections. It emits proposals to host policy, and accepted proposals
become `ui_composition` transactions.

## Adaptive Edit Classification

- `TransientAdaptive`: drawer state, hover, preview, temporary projected bounds,
  compact substitutions, and pointer position.
- `StructuralTransaction`: explicitly committed canonical movement, splitting,
  ordering, resizing, mounting, root, or target change.
- `PromotionCandidate`: projected arrangement explicitly requested as a new
  saved layout.

Responsive reflow is always transient until explicit promotion. Canonical state
automatically restores when constraints disappear.

## Interaction Model

The selected UX must implement a modern IDE/browser hybrid backed by the
adaptive region graph:

- browser-like tab movement;
- explicit center and edge drop targets;
- floating roots;
- real native-window detach/reattach;
- policy-driven compact drawers and overflow;
- immediate pointer tracking and transactional commit/rollback.

Pointer, touch, keyboard, and controller input map to semantic actions. The
adaptive runtime never imports raw gamepad semantics.

Required semantic actions include focus direction, activate, cancel, tab cycle,
enter move mode, enter resize mode, commit, and rollback.

Touch uses long-press on an explicit drag region and visible touch-sized resize
handles. Escape/cancel rolls every active interaction back without structural
mutation.

## App Proofs

The editor proves the complete docking model, including actual cross-window
movement and target-close rehome/veto behavior.

Runenwerk Draw proves adaptive composition: the primary canvas remains usable,
while low-priority regions collapse into drawers or overflow and restore when
space returns.

Browser, terminal, dashboard, mobile, and game examples remain headless fixtures
only.

## Accessibility Acceptance

Runtime acceptance requires:

- keyboard-only docking and resizing;
- deterministic focus order and visible focus;
- high-contrast compatibility;
- text scaling without clipping or inaccessible controls;
- reduced-motion zero-duration behavior;
- touch target sizing;
- controller parity;
- Escape/cancel behavior;
- screen-reader or inspection labels for tabs, panels, targets, previews,
  drawers, active state, and unavailable content where supported.

Motion is short and tokenized. Pointer tracking remains immediate. Reduced
motion disables transition duration rather than changing structure.

## Performance Contract

Deterministic benchmark layouts:

- normal: 128 regions, 64 mounted units, 4 targets;
- large: 2,048 regions, 1,024 mounted units, 16 targets;
- transaction case: 64 commands against the large layout.

Optimized reference-desktop runs use warm-up and at least 30 measured samples.
Required p95 budgets:

| Path | Budget |
|---|---:|
| Region hit testing | 0.25 ms |
| Adaptive proposal generation | 0.75 ms |
| Preview projection | 1.00 ms |
| Complete drag-frame update | 2.00 ms |
| 64-command transaction validation | 1.50 ms |
| Committed 64-command mutation | 1.50 ms |
| Large canonical serialization | 20 ms |
| Large validation/deserialization | 20 ms |

The drag path must perform zero full `CompositionState` or region-graph clones
per pointer move. Per-frame allocation must not scale linearly with total graph
size; unchanged storage is shared and committed mutations rebuild only affected
paths or indexes.

CI records trends. Absolute acceptance uses the recorded reference desktop.

## Visual Direction Gate

The Product Design ideation checkpoint must generate exactly three independent
1440x1024 directions using actual checked-in Runenwerk captures as image
references. Each must show tab dragging, center/edge docking, cross-window
detach affordance, compact drawers, focus, and touch resize affordances.

No runtime/chrome/docking implementation starts until one direction is selected.

