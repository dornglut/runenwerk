---
title: UI Composition Visual Direction Decision Analysis
description: Critical comparison of the three PM-UI-COMPOSITION-001 docking directions against architecture, accessibility, interaction, and implementation criteria.
status: active
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

# UI Composition Visual Direction Decision Analysis

## Decision Criteria

The visual target must support the architecture rather than force architecture
around chrome. The comparison therefore prioritizes:

1. explicit mapping to typed center/edge/detach proposals;
2. keyboard, controller, touch, and screen-reader discoverability;
3. low accidental-drop risk and reversible interaction;
4. minimal persistent chrome and preserved primary viewport area;
5. clean separation between adaptive mechanism and editor presentation;
6. measurable hit-testing and preview behavior without graph cloning.

## Option 1: Edge Rails

Strengths:

- lowest learning cost for conventional IDE users;
- large edge targets can meet touch sizing cleanly;
- destination preview and primary viewport remain easy to understand;
- implementation maps directly to region bounds and five target roles.

Risks:

- full-edge rails obscure more destination content during drag;
- a global New Window control risks coupling detach behavior to permanent shell
  chrome rather than the active drag session;
- target glyphs are less self-explanatory than the compass arrows;
- repeated rail affordances can become visually noisy with nested regions.

This is the safest conservative direction, but it does not express stack
insertion as clearly as the other options.

## Option 2: Region Compass

Strengths:

- all legal target roles are visible in one contextual control;
- arrows map cleanly to semantic keyboard/controller focus and commit actions;
- the overlay exists only during an interaction session, keeping normal chrome
  minimal;
- detach is explicit and separate from ordinary split targets;
- dashed destination previews make rollback and expected outcome clear;
- the mechanism remains projection-owned and does not require persistent editor
  structural chrome.

Risks:

- the compass can obscure the center of content if it is oversized;
- focus order, spoken labels, target contrast, and reduced-motion transitions
  must be specified carefully;
- large dashed previews should appear only for the focused candidate, not all
  candidates simultaneously, to avoid visual overload.

This direction has the strongest architecture-to-interaction correspondence and
the cleanest semantic-input model.

## Option 3: Structural Lanes

Strengths:

- browser-like tab insertion and region seams communicate structure directly;
- stack reorder versus split placement can be distinguished precisely;
- seam handles can become good touch resize affordances;
- compact side drawers demonstrate responsive priorities clearly.

Risks:

- persistent lanes and side tools consume the most viewport area;
- the model couples interaction more tightly to editor tab/seam presentation;
- native-window detach requires longer pointer travel to a remote portal;
- more simultaneously active lanes increase hit-test ambiguity and accidental
  drop risk in nested layouts;
- keyboard/controller navigation through many lane targets is more complex.

This is the most expressive direct-manipulation direction, but it has the
highest chrome and interaction-state complexity.

## Recommendation

Select **Option 2: Region Compass** as the base target.

It best preserves KISS at the architecture boundary: one contextual adaptive
session projects a finite target set, semantic input can navigate the same set,
the editor supplies visuals without becoming structural authority, and normal
chrome remains quiet. Its risks are bounded accessibility and sizing work rather
than additional ownership or persistent-state complexity.

If a hybrid is preferred, the only recommended blend is Option 2's contextual
compass with Option 3's tab-strip insertion marker and Option 1's larger touch
edge targets. That blend must be generated and selected as a new visual target
before implementation.

## Gate State

Recommendation is not selection. `selection.md` remains absent and runtime work
remains blocked until the user explicitly chooses an option or requests a hybrid
revision.
