---
title: UI Composition Visual Direction Selection
description: Binding selection and implementation constraints for the Region Compass composition interaction direction.
status: accepted
owner: ui
layer: report
canonical: false
last_reviewed: 2026-06-19
related_designs:
  - ../../../design/accepted/adaptive-ui-composition-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# UI Composition Visual Direction Selection

## Decision

The user selected **Option 2: Region Compass** on 2026-06-19.

![Selected Region Compass direction](option-2-region-compass.png)

This selection is binding for the later editor docking implementation. It does
not authorize runtime, chrome, docking, or consumer-migration work ahead of the
governed checkpoint sequence.

## Interaction Contract

- The compass is a contextual interaction overlay, never permanent shell
  chrome or structural authority.
- The active destination exposes a finite center, left, right, top, and bottom
  target set derived from adaptive composition proposals.
- All legal targets may be visible, but only the focused candidate displays its
  outcome preview.
- Detach or new-window placement is a separate explicit portal and is never
  encoded as an ordinary split target.
- Pointer, touch, keyboard, and controller input navigate and commit the same
  semantic target set.
- Escape, cancel, focus loss, rejected commit, or windowing veto rolls the
  interaction back without structural mutation.
- A successful commit becomes structural only through a validated
  `ui_composition` transaction at the checkpoint where that authority exists.

## Accessibility Contract

- Every target has a stable inspection and screen-reader label that names the
  action and destination.
- Keyboard and controller focus is visible at high contrast and follows a
  deterministic spatial order.
- Touch targets meet the repository's later acceptance gate without changing
  target semantics.
- Text scaling must not hide target labels or the detach action.
- Reduced-motion mode removes nonessential movement while preserving preview
  and commit feedback.
- Color is never the only signal for focus, validity, rejection, or commit.

## Visual Contract

- Preserve the existing Runenwerk token language: black and near-black
  surfaces, light and muted text, electric-blue accent, one-pixel borders,
  compact spacing, zero-radius geometry, and compact tool typography.
- Keep the compass small enough to preserve destination context and derive its
  placement from the active region rather than fixed screen coordinates.
- Use the selected artifact as direction evidence, not as pixel-perfect runtime
  geometry. Runtime geometry must satisfy the adaptive, accessibility, and
  performance contracts before visual matching.

## Supersession Rule

Any future hybrid, material redesign, or change to the target vocabulary must
produce a new grounded visual option and receive explicit user selection before
superseding this decision.
