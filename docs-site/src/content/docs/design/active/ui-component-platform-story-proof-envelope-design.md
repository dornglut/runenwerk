---
title: UI Component Platform Story Proof Envelope Design
description: Phase 3 design for how ControlPackages declare component story requirements and consume ui_story V2 proof reports without owning the story runner.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-25
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./ui-component-platform-control-kernel-design.md
  - ./ui-component-platform-authoring-kit-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# UI Component Platform Story Proof Envelope Design

## Status

This is the Phase 3 planning and acceptance design for `feature/ui-component-platform-003-story-proof-envelope`.

It follows Phase 1 `ControlPackage` / `ControlKernel` contracts and Phase 2 authoring ergonomics. It defines how reusable controls name required story proof and how component maturity consumes `ui_story` V2 reports. It does not authorize story runner behavior, Gallery previews, Designer UX, Workbench behavior, runtime widget behavior, runtime mount eligibility, text editing, canvas surfaces, transitions, or product host mutation.

## Existing Authority

`ui_story` is already the proof substrate. It owns manifests, workflow graph execution, evidence, reports, diagnostics, and mount verdicts. The active UI platform roadmap says story proof comes before expanded rendering or product mounting claims, and reusable component maturity starts after story-gated rendering proof rather than waiting for the entire story platform closeout.

The story-driven golden workflow says `UiStory` is the canonical developer-facing proof envelope and `UiStoryWorkflowReportV2` is the inspection object consumed by mount decisions, CLI summaries, and later app integration. Phase 3 must consume that authority instead of inventing a parallel control-specific proof runner.

## Problem

`ControlPackage` descriptors currently name stories and evidence requirements, but the contract is not yet strong enough to answer these questions for a reusable control family:

- Which stories form the required matrix for a control kind?
- Which stories are normal, edge, failure, accessibility, interaction, layout, text, render, and budget proof?
- Which story outcomes satisfy descriptor-only maturity?
- Which story outcomes are required before future mount eligibility can even be considered?
- How does a failed component story point back to the package, control kind, state, schema, route, token, accessibility, render, or budget requirement that caused it?
- How does Gallery, Workbench, UI Designer, or a docs page inspect component proof without owning control semantics?

Without a component-level story proof envelope, later phases could accidentally treat a visible preview, a hand-authored fixture, or a single happy-path story as reusable component maturity.

## Decision

Add a component story proof layer to the UI Component Platform.

The component layer owns **story requirements and proof consumption**, not story execution.

Correct ownership split:

```text
ui_controls
  owns ControlPackage, control kind story requirements, story matrix metadata,
  and control-package maturity interpretation.

ui_story
  owns manifests, workflow graph, evidence records, runner inputs/outputs,
  workflow reports, diagnostics, expected-failure matching, and mount verdicts.

ui_artifacts
  owns exported read-only package/proof snapshots for downstream inspection.

Gallery / Workbench / UI Designer
  consume story proof and package snapshots; they do not own reusable control semantics.
```

## Proposed Contract Shape

The future implementation should prefer one focused module first:

```text
domain/ui/ui_controls/src/story_proof.rs
```

Split later only if stable responsibilities justify it.

Candidate public concepts:

```text
ControlStoryMatrixDescriptor
ControlStoryMatrixEntry
ControlStoryProofRequirement
ControlStoryProofProfile
ControlStoryProofEnvelope
ControlStoryProofVerdict
ControlStoryProofDiagnostic
ControlStoryProofSummary
```

The final names may differ after inspecting nearby conventions, but the responsibilities should remain:

- bind a `ControlKindId` to required `ControlStoryId`s;
- categorize stories as normal, edge, failure, accessibility, interaction, layout, text, render, budget, and mount-readiness proof;
- record whether a story is expected to pass or expected to fail;
- require stable references to `ui_story` V2 story IDs or workflow report identities;
- summarize proof outcome without copying the whole `UiStoryWorkflowReportV2` into `ui_controls`;
- keep component maturity descriptor-only unless story/render/budget evidence requirements are actually satisfied;
- expose diagnostics that identify package, control kind, story, proof category, and first blocking reason;
- remain inspectable by `ui_artifacts` without giving artifacts mutation authority.

## Required Proof Categories

A mature reusable control kind must be able to declare these categories, even if a Phase 3 implementation only proves a minimal subset first:

```text
normal
edge
failure
accessibility
interaction
layout
text
render
budget
mount-readiness
```

For Phase 3 implementation, the minimum acceptable subset is:

```text
normal
failure
accessibility
render
budget
```

The minimum subset is intentionally small. It proves the envelope without starting interaction, layout, text, canvas, or runtime-mount phases early.

## Report Consumption Rules

A component story proof envelope may consume `ui_story` V2 reports only as evidence.

It must not:

- run stories itself;
- reinterpret workflow graph ownership;
- infer success from visible output;
- treat skipped or blocked evidence as passed;
- mutate app/editor/game state;
- make a control runtime-mount eligible;
- move Gallery, Workbench, or Designer semantics into `ui_controls`.

A report satisfies a proof requirement only when:

- the report story identity matches the required story identity;
- the report outcome matches the expected pass/failure policy;
- no blocking diagnostic remains for required proof nodes;
- required evidence categories are present;
- expected-failure stories match their diagnostic expectations;
- the first blocking diagnostic is preserved when proof fails.

## Descriptor-Only Versus Mount-Ready

Phase 3 does **not** make controls runtime-mount eligible.

Allowed output:

```text
Control kind has declared story requirements.
Story proof summary can say requirements are satisfied or unsatisfied.
Catalog/artifact consumers can inspect the proof summary.
Mount eligibility remains explicit and conservative.
```

Forbidden output:

```text
Control kind becomes runtime mount eligible.
Gallery preview is treated as production proof.
Renderer output becomes UI source truth.
Story report ownership moves into ui_controls.
```

## Candidate Implementation Scope

The first implementation pass may touch:

```text
domain/ui/ui_controls/src/story_proof.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/package/metadata.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/tests/control_story_proof_contract.rs
domain/ui/ui_artifacts/src/control_packages.rs
domain/ui/ui_artifacts/tests/control_package_artifact.rs
```

Use `ui_artifacts` only if exported read-only snapshots need story proof data. Do not add app/editor/gallery code in Phase 3.

## Non-Goals

Do not implement:

- `ui_story` runner behavior;
- new story manifest parsing;
- Gallery story execution;
- CLI story execution;
- Designer UX;
- Workbench behavior;
- runtime widget behavior;
- runtime mount eligibility;
- app/editor/game mutation;
- Surface2D;
- SpatialCanvas;
- NodeCanvas;
- PortGraphCanvas;
- ProgressionTreeView;
- TrackSurface;
- Timeline;
- CurveEditor;
- transitions;
- text editing;
- renderer-owned UI semantics;
- ECS-owned UI semantics.

## Boundary Rules

- `ui_controls` may define control story requirements and proof summaries.
- `ui_controls` must not execute stories.
- `ui_controls` must not own `ui_story` workflow graphs.
- `ui_controls` must not duplicate `UiStoryEvidence`, `UiStoryWorkflowReportV2`, or `UiStoryOutcomeV2` as a parallel model.
- `ui_controls` may store stable references and minimal summaries needed for component maturity.
- `ui_artifacts` may export read-only story proof facts.
- Gallery, Workbench, and UI Designer remain consumers.
- Runtime mount eligibility remains future-gated.

## Acceptance Criteria

Phase 3 is implementation-complete only when:

- component story proof requirements are represented in `ui_controls`;
- each `ControlKindDescriptor` can reference a story matrix or proof requirements without breaking existing packages;
- validation catches missing, duplicate, or unresolved story proof requirements;
- focused tests prove valid proof requirements pass and invalid requirements fail closed;
- expected-failure stories are represented as first-class requirements;
- proof summaries preserve the first blocking diagnostic or first unsatisfied requirement;
- artifact export remains read-only if story proof data is exported;
- no story runner, Gallery, Designer, Workbench, runtime mount, canvas, text, transition, renderer, or ECS behavior is implemented.

## Test Plan

Required focused tests for the future implementation pass:

```text
cargo test -p ui_controls control_story_proof
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_controls control_authoring
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
cargo test -p ui_story workflow
```

Required static checks:

```text
cargo fmt --all --check
cargo check -p ui_controls
git diff --check
```

Recommended test cases:

- valid control story matrix validates;
- duplicate story requirement is rejected;
- required story ID with no matching descriptor is rejected;
- missing minimum proof category is rejected for a mature profile;
- expected-failure story requirement is represented and does not count as a normal pass;
- proof summary reports first unsatisfied requirement;
- artifact snapshot preserves story proof metadata read-only;
- runtime mount eligibility remains not eligible.

## Phase 3 Implementation Gate

Before writing Rust code, confirm:

- this design is accepted;
- Phase 2 remains green on the branch base;
- `ui_story` V2 report/evidence names are still current;
- planning records name Phase 3 implementation as active;
- no new stop condition has been triggered.

## Handoff

Start implementation only after this design and planning state are accepted. The first implementation pass should add the smallest control story proof contract and focused tests. Do not implement story execution, Gallery/CLI execution, product mount eligibility, text, canvas, transition, Designer, Workbench, renderer, or ECS behavior in Phase 3.
