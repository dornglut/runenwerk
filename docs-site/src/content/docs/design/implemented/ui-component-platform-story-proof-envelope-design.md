---
title: UI Component Platform Story Proof Envelope Design
description: Implemented Runenwerk-local control story requirement, proof profile, verdict, diagnostic, and summary contract over the ui_story evidence owner.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ./ui-component-platform-control-kernel-design.md
  - ./ui-component-platform-authoring-kit-design.md
  - ./ui-component-platform-executable-interaction-story-design.md
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Story Proof Envelope Design

## Status

Implemented. `ui_controls` now represents reusable control story requirements and proof interpretation without taking ownership of story execution.

## Implemented contract

`domain/ui/ui_controls/src/story_proof/mod.rs` provides:

- `ControlStoryProofCategory`;
- `ControlStoryProofExpectedOutcome`;
- `ControlStoryProofProfile`;
- `ControlStoryProofRequirement`;
- `ControlStoryMatrixDescriptor`;
- `ControlStoryProofVerdict`;
- `ControlStoryProofDiagnostic`;
- `ControlStoryProofSummary`.

Proof categories cover normal, edge, failure, accessibility, interaction, layout, text, render, budget, and mount-readiness evidence. Profiles define required category sets while preserving conservative descriptor-only behavior.

Matrix validation resolves control/story identity against package contents, rejects duplicate/unresolved requirements, and checks required profile categories. Proof summaries preserve satisfied/unsatisfied/not-evaluated state plus the first unsatisfied requirement or blocking diagnostic.

## Ownership

`ui_controls` owns component proof requirements and compact maturity interpretation. `ui_story` remains the story workflow/evidence/report execution owner. `ui_artifacts` may export read-only proof facts. Product tools consume the evidence without becoming control-semantic owners.

## Intentional landed differences

The proposal named `ControlStoryMatrixEntry` as a separate concept. The implementation normalizes it as a type alias of `ControlStoryProofRequirement`; this avoids duplicate structures while retaining matrix-entry semantics.

The current proof summary is deliberately compact rather than copying the entire `ui_story` workflow report into `ui_controls`.

## Boundary rules

- `ui_controls` does not execute stories.
- It does not duplicate the `ui_story` workflow graph/report model.
- Expected-failure requirements remain explicit first-class proof policy.
- Satisfied proof requirements do not implicitly execute product behavior.
- Runtime mount eligibility remains separately explicit and evidence-gated.

## Evidence

Current story-proof source plus focused package/story-proof tests exercise duplicate/missing requirements, required categories, expected failures, summaries, and fail-closed package correlation.
